use actix_web::{web, HttpResponse};
use diesel::prelude::*;
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
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::DatabaseError(_) => HttpResponse::InternalServerError().json("Database error"),
            ApiError::PoolError(_) => HttpResponse::InternalServerError().json("Connection pool error"),
            ApiError::BlockingError(_) => HttpResponse::InternalServerError().json("Blocking operation error"),
        }
    }
}

pub async fn create_product(
    pool: web::Data<DbPool>,
    new_product: web::Json<NewProduct>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;

    web::block(move || {
        diesel::insert_into(products)
            .values(&new_product.into_inner())
            .execute(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Created().finish())
}

pub async fn get_products(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;

    let results = web::block(move || products.load::<Product>(&mut conn))
        .await??;

    Ok(HttpResponse::Ok().json(results))
}

pub async fn get_product(
    pool: web::Data<DbPool>,
    product_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;

    let product = web::block(move || 
        products.filter(id.eq(product_id.into_inner())).first::<Product>(&mut conn)
    )
    .await??;

    Ok(HttpResponse::Ok().json(product))
}

pub async fn update_product(
    pool: web::Data<DbPool>,
    product_id: web::Path<i32>,
    updated_product: web::Json<NewProduct>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;

    web::block(move || {
        diesel::update(products.filter(id.eq(product_id.into_inner())))
            .set(&updated_product.into_inner())
            .execute(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().finish())
}

pub async fn delete_product(
    pool: web::Data<DbPool>,
    product_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get()?;

    web::block(move || 
        diesel::delete(products.filter(id.eq(product_id.into_inner())))
            .execute(&mut conn)
    )
    .await??;

    Ok(HttpResponse::Ok().finish())
}