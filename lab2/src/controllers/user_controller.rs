use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::models::User;
use crate::schema::users;
use crate::db::DbConnection;
use bcrypt::{hash, DEFAULT_COST};
use serde::Deserialize;
use serde_json::json;
use r2d2::PooledConnection;
use diesel::r2d2::ConnectionManager;
use log::{info, error};
#[derive(Deserialize, Debug)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub role_id: Option<i32>,
    pub apartment: Option<String>,
    pub block_id: Option<i32>,
    pub photo: Option<Vec<u8>>,
    pub password: String,
}

pub async fn create_user(new_user: web::Json<NewUser>, db: web::Data<DbConnection>) -> impl Responder {
    info!("Creating user: {:?}", new_user); // Log the new user data

    let mut conn: PooledConnection<ConnectionManager<SqliteConnection>> = db.get().expect("Failed to get DB connection");

    let hashed_password = hash(&new_user.password, DEFAULT_COST).expect("Failed to hash password");

    let user = User {
        id: 0, // Use 0 for auto-increment
        first_name: new_user.first_name.clone(),
        last_name: new_user.last_name.clone(),
        username: new_user.username.clone(),
        role_id: new_user.role_id,
        apartment: new_user.apartment.clone(),
        block_id: new_user.block_id,
        photo: new_user.photo.clone(),
        password: hashed_password,
    };

    let result = web::block(move || {
        // Move conn into the closure
        diesel::insert_into(users::table)
            .values(&user)
            .execute(&mut conn)
    }).await;

    match result {
        Ok(_) => {
            info!("User created successfully.");
            HttpResponse::Created().json(json!({"message": "User created successfully"}))
        },
        Err(err) => {
            error!("Failed to create user: {:?}", err); // Log the error
            HttpResponse::InternalServerError().finish()
        },
    }
}
