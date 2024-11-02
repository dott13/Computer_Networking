use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::models::User;
use crate::schema::users;
use crate::db::DbConnection;
use crate::utils::pagination::PaginatedResponse;
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use diesel::{Queryable, Selectable};
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

#[derive(Deserialize, Debug)]
pub struct UserQueryParams {
    page: Option<i64>,
    per_page: Option<i64>,
    block_id: Option<i32>,
}

// Add the `QueryableByName` derive
#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = users)]
pub struct UserResponse {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub first_name: String,
    #[diesel(sql_type = Text)]
    pub last_name: String,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Nullable<Integer>)]
    pub role_id: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub apartment: Option<String>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub block_id: Option<i32>,
    #[diesel(sql_type = Nullable<Binary>)]
    pub photo: Option<Vec<u8>>,
}

pub async fn get_users(
    db: web::Data<DbConnection>,
    query: web::Query<UserQueryParams>,
) -> impl Responder {
    info!("Fetching users with query params: {:?}", query);

    let mut conn: PooledConnection<ConnectionManager<SqliteConnection>> = match db.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let page_number = query.page.unwrap_or(1);
    let items_per_page = query.per_page.unwrap_or(10);
    let offset = (page_number - 1) * items_per_page;
    let block_filter = query.block_id;

    let result = web::block(move || {
        // Build base query and use `UserResponse::as_select()` to ensure compatibility
        let mut query = users::table
            .select(UserResponse::as_select())  // Automatically matches fields
            .into_boxed();
        
        // Apply filter if block_id is provided
        if let Some(block_id_val) = block_filter {
            query = query.filter(users::block_id.eq(block_id_val));
        }

        // Get total count with consistent filtering
        let total_query = users::table.select(diesel::dsl::count_star()).into_boxed();
        let total_query = if let Some(block_id_val) = block_filter {
            total_query.filter(users::block_id.eq(block_id_val))
        } else {
            total_query
        };
        
        let total_items: i64 = total_query.first(&mut conn)?;

        // Then get paginated results
        let results = query
            .offset(offset)
            .limit(items_per_page)
            .load::<UserResponse>(&mut conn)?;

        Ok::<(Vec<UserResponse>, i64), diesel::result::Error>((results, total_items))
    })
    .await;

    match result {
        Ok(Ok((results, total_items))) => {
            let total_pages = (total_items as f64 / items_per_page as f64).ceil() as i64;

            let response = PaginatedResponse {
                data: results,
                total: total_items,
                page: page_number,
                per_page: items_per_page,
                total_pages,
            };

            info!("Successfully fetched users. Total: {}", total_items);
            HttpResponse::Ok().json(response)
        }
        Ok(Err(e)) => {
            error!("Database error while fetching users: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
        Err(e) => {
            error!("Error while processing request: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}