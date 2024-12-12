use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use thiserror::Error;
use actix_web::error::BlockingError;
use serde::{Deserialize, Serialize};
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
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::DatabaseError(_) => HttpResponse::InternalServerError().json("Database error"),
            ApiError::PoolError(_) => HttpResponse::InternalServerError().json("Connection pool error"),
            ApiError::BlockingError(_) => HttpResponse::InternalServerError().json("Blocking operation error"),
            ApiError::NotFound => HttpResponse::NotFound().json("Product not found"),
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
    new_product: web::Json<NewProduct>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;
    let inserted_product = web::block(move || {
        // For SQLite, we'll insert and then manually fetch the last inserted product
        diesel::insert_into(products)
            .values(&new_product.into_inner())
            .execute(&mut conn)?;
        
        // Fetch the last inserted product
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