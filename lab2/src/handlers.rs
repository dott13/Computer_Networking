use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use diesel::prelude::*;
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use actix_web::error::BlockingError;
use crate::db::DbPool;
use crate::models::{NewProduct, Product};
use crate::schema::products::dsl::*;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Connection pool error")]
    PoolError(#[from] r2d2::Error),
    #[error("Blocking error")]
    BlockingError(#[from] BlockingError),
    #[error("Product not found")]
    NotFound,
    #[error("Multipart error")]
    MultipartError(#[from] actix_multipart::MultipartError),
    #[error("UTF-8 conversion error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Float parse error")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::DatabaseError(_) => HttpResponse::InternalServerError().json("Database error"),
            ApiError::PoolError(_) => HttpResponse::InternalServerError().json("Connection pool error"),
            ApiError::BlockingError(_) => HttpResponse::InternalServerError().json("Blocking operation error"),
            ApiError::NotFound => HttpResponse::NotFound().json("Product not found"),
            ApiError::MultipartError(_) => HttpResponse::BadRequest().json("Multipart error"),
            ApiError::Utf8Error(_) => HttpResponse::BadRequest().json("Invalid UTF-8 data"),
            ApiError::ParseFloatError(_) => HttpResponse::BadRequest().json("Invalid float value"),
        }
    }
}

#[derive(Deserialize)]
pub struct PaginationParams {
    offset: Option<i64>,
    limit: Option<i64>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    data: Vec<T>,
    total_count: i64,
    offset: i64,
    limit: i64,
}

pub async fn create_product(
    pool: web::Data<DbPool>,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let mut product_name: Option<String> = None;
    let mut product_price: Option<f64> = None;
    let mut product_description: Option<String> = None;
    let mut product_image_data: Option<Vec<u8>> = None;

    // Process the multipart payload
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition().unwrap();

        if let Some(field_name) = content_disposition.get_name() {
            match field_name {
                "name" => {
                    let mut value = Vec::new();
                    while let Some(chunk) = field.next().await {
                        value.extend_from_slice(&chunk?);
                    }
                    product_name = Some(String::from_utf8(value)?);
                }
                "price" => {
                    let mut value = Vec::new();
                    while let Some(chunk) = field.next().await {
                        value.extend_from_slice(&chunk?);
                    }
                    product_price = Some(String::from_utf8(value)?.parse()?);
                }
                "description" => {
                    let mut value = Vec::new();
                    while let Some(chunk) = field.next().await {
                        value.extend_from_slice(&chunk?);
                    }
                    product_description = Some(String::from_utf8(value)?);
                }
                "image" => {
                    let mut image_data = Vec::new();
                    while let Some(chunk) = field.next().await {
                        image_data.extend_from_slice(&chunk?);
                    }
                    product_image_data = Some(image_data);
                }
                _ => {}
            }
        }
    }

    // Validate required fields
    let product_name = product_name.ok_or(ApiError::NotFound)?;
    let product_price = product_price.ok_or(ApiError::NotFound)?;

    // Insert into the database
    let new_product = NewProduct {
        name: product_name,
        price: product_price,
        description: product_description,
        image: product_image_data,
    };

    let mut conn = pool.get()?;
    let inserted_product = web::block(move || {
        diesel::insert_into(products)
            .values(&new_product)
            .execute(&mut conn)?;
        products.order(id.desc()).first::<Product>(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Created().json(inserted_product))
}

pub async fn get_products(
    pool: web::Data<DbPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
    
    // Default values for pagination
    let page_offset = pagination.offset.unwrap_or(0);
    let page_limit = pagination.limit.unwrap_or(10).min(100); // Limit max to 100 items per page

    // Get paginated results
    let result = web::block(move || -> Result<(Vec<Product>, i64), diesel::result::Error> {
        // First, get the total count of products
        let count = products.count().get_result::<i64>(&mut conn)?;
        
        // Then get the paginated results
        let results = products
            .offset(page_offset)
            .limit(page_limit)
            .load::<Product>(&mut conn)?;
        
        Ok((results, count))
    })
    .await??;

    // Create paginated response
    let response = PaginatedResponse {
        data: result.0,
        total_count: result.1,
        offset: page_offset,
        limit: page_limit,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_product(
    pool: web::Data<DbPool>,
    product_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
    let product = web::block(move ||
        products.filter(id.eq(product_id.into_inner())).first::<Product>(&mut conn)
    )
    .await.map_err(|_| ApiError::NotFound)??;
    
    Ok(HttpResponse::Ok().json(product))
}

pub async fn update_product(
    pool: web::Data<DbPool>,
    product_id: web::Path<i32>,
    updated_product: web::Json<NewProduct>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
    let product_id_inner = product_id.into_inner();
    let updated = web::block(move || {
        // For SQLite, we'll update and then manually fetch the updated product
        diesel::update(products.filter(id.eq(product_id_inner)))
            .set(&updated_product.into_inner())
            .execute(&mut conn)?;
        
        // Fetch the updated product
        products.filter(id.eq(product_id_inner)).first::<Product>(&mut conn)
    })
    .await.map_err(|_| ApiError::NotFound)??;
    
    Ok(HttpResponse::Ok().json(updated))
}

pub async fn delete_product(
    pool: web::Data<DbPool>,
    product_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
    let product_id_inner = product_id.into_inner();
    let deleted_count = web::block(move ||
        diesel::delete(products.filter(id.eq(product_id_inner)))
            .execute(&mut conn)
    )
    .await??;
    
    if deleted_count == 0 {
        return Err(ApiError::NotFound);
    }
    
    Ok(HttpResponse::Ok().json(format!("Product {} deleted", product_id_inner)))
}