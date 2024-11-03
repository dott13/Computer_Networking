use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use diesel::result::{Error as DieselError, DatabaseErrorKind};
use jsonwebtoken::{encode, Header, EncodingKey};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::schema::users;
use crate::models::User;
use crate::db::DbConnection;
use crate::utils::pagination::PaginatedResponse;
use bcrypt::{hash, DEFAULT_COST, verify};
use actix_multipart::Multipart;
use futures::stream::StreamExt; // For the `next` method
use serde::{Deserialize, Serialize};
use diesel::{Queryable, Selectable};
use serde_json::json;
use r2d2::PooledConnection;
use diesel::r2d2::ConnectionManager;
use log::{info, error};

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub role_id: Option<i32>,
    pub apartment: Option<String>,
    pub block_id: Option<i32>,
    pub password: String,
    pub photo: Option<Vec<u8>>,
}

pub async fn create_user(
    mut payload: Multipart,
    db: web::Data<DbConnection>,
) -> impl Responder {
    let mut new_user = NewUser {
        first_name: String::new(),
        last_name: String::new(),
        username: String::new(),
        role_id: None,
        apartment: None,
        block_id: None,
        password: String::new(),
        photo: None,
    };

    // Parse the multipart form data
    while let Some(field_result) = payload.next().await {
        match field_result {
            Ok(mut field) => {
                let field_name = field.name();
                match field_name {
                    Some("first_name") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        new_user.first_name = String::from_utf8(bytes.to_vec()).unwrap();
                    }
                    Some("last_name") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        new_user.last_name = String::from_utf8(bytes.to_vec()).unwrap();
                    }
                    Some("username") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        new_user.username = String::from_utf8(bytes.to_vec()).unwrap();
                    }
                    Some("role_id") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        new_user.role_id = Some(String::from_utf8(bytes.to_vec()).unwrap().parse().unwrap());
                    }
                    Some("apartment") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        new_user.apartment = Some(String::from_utf8(bytes.to_vec()).unwrap());
                    }
                    Some("block_id") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        new_user.block_id = Some(String::from_utf8(bytes.to_vec()).unwrap().parse().unwrap());
                    }
                    Some("password") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        new_user.password = String::from_utf8(bytes.to_vec()).unwrap();
                    }
                    Some("photo") => {
                        // Collect the photo data
                        let mut data = Vec::new();
                        while let Some(chunk) = field.next().await {
                            let chunk = chunk.unwrap();
                            data.extend_from_slice(&chunk);
                        }
                        new_user.photo = Some(data);
                    }
                    _ => {}
                }
            },
            Err(err) => {
                error!("Error processing field: {:?}", err);
                return HttpResponse::BadRequest().finish();
            }
        }
    }

    info!("Creating user: {:?}", new_user);

    // Get a database connection
    let mut conn: PooledConnection<ConnectionManager<SqliteConnection>> = db.get().expect("Failed to get DB connection");
    info!("Successfully connected to the database");
    
    // Hash the password before insertion
    let hashed_password = hash(&new_user.password, DEFAULT_COST).expect("Failed to hash password");

    // Create a NewUser instance for insertion
    let new_user_insert = NewUser {
        first_name: new_user.first_name.clone(),
        last_name: new_user.last_name.clone(),
        username: new_user.username.clone(),
        role_id: new_user.role_id,
        apartment: new_user.apartment.clone(),
        block_id: new_user.block_id,
        password: hashed_password, // Use the hashed password
        photo: new_user.photo.clone(),
    };

    // Insert into the database directly
    let result = diesel::insert_into(users::table)
        .values(&new_user_insert) // Use NewUser struct here
        .execute(&mut conn);

    match result {
        Ok(_) => {
            info!("User created successfully.");
            HttpResponse::Created().json(json!({"message": "User created successfully"}))
        },
        Err(err) => {
            match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    error!("Failed to create user: Username already exists.");
                    HttpResponse::Conflict().json(json!({"error": "Username already exists."}))
                },
                _ => {
                    error!("Failed to create user due to a database error: {:?}", err);
                    HttpResponse::InternalServerError().finish()
                },
            }
        },
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,
    role: i32, 
    exp: usize,       // Expiration time as UNIX timestamp
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    db: web::Data<DbConnection>,
    login_data: web::Json<LoginRequest>
) -> impl Responder {
    let mut conn = db.get().expect("Failed to get DB connection");

    let result: Result<User, diesel::result::Error> = users::table
        .filter(users::username.eq(&login_data.username))
        .first(&mut conn);

    match result {
        Ok(user) => {
            // Verify password
            if verify(&login_data.password, &user.password).unwrap_or(false) {
                // Generate JWT token
                let expiration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as usize
                    + 7 * 24 * 60 * 60; // Token valid for 7 days

                let claims = Claims {
                    sub: user.id,
                    role: user.role_id.unwrap_or_default(),
                    exp: expiration,
                };

                let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(secret_key.as_ref()),
                )
                .expect("Failed to encode token");
                info!("User entered the system successfully. {}", token);
                HttpResponse::Ok().json(json!({"token": token}))
            } else {
                HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}))
            }
        }
        Err(_) => HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"})),
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

//Get users with pagination and filtering
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

// Get a single user by ID
pub async fn get_user(
    db: web::Data<DbConnection>,
    user_id: web::Path<i32>,
) -> impl Responder {
    info!("Fetching user with ID: {}", user_id);

    let mut conn: PooledConnection<ConnectionManager<SqliteConnection>> = match db.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let result = web::block(move || {
        users::table
            .filter(users::id.eq(*user_id))
            .select(UserResponse::as_select())
            .first::<UserResponse>(&mut conn)
    })
    .await;

    match result {
        Ok(Ok(user)) => HttpResponse::Ok().json(user),
        Ok(Err(diesel::NotFound)) => HttpResponse::NotFound().json(json!({"error": "User not found"})),
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

#[derive(Insertable, Deserialize, Debug, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub role_id: Option<i32>,
    pub apartment: Option<String>,
    pub block_id: Option<i32>,
    pub password: Option<String>,
    pub photo: Option<Vec<u8>>,
}

//Update a user by his id
pub async fn update_user(
    user_id: web::Path<i32>,
    mut payload: Multipart,
    db: web::Data<DbConnection>,
) -> impl Responder {
    let mut updated_user = UpdateUser {
        first_name: None,
        last_name: None,
        username: None,
        role_id: None,
        apartment: None,
        block_id: None,
        password: None,
        photo: None,
    };

    // Parse the multipart form data
    while let Some(field_result) = payload.next().await {
        match field_result {
            Ok(mut field) => {
                let field_name = field.name();
                match field_name {
                    Some("first_name") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        updated_user.first_name = Some(String::from_utf8(bytes.to_vec()).unwrap());
                    }
                    Some("last_name") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        updated_user.last_name = Some(String::from_utf8(bytes.to_vec()).unwrap());
                    }
                    Some("username") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        updated_user.username = Some(String::from_utf8(bytes.to_vec()).unwrap());
                    }
                    Some("role_id") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        updated_user.role_id = Some(String::from_utf8(bytes.to_vec()).unwrap().parse().unwrap());
                    }
                    Some("apartment") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        updated_user.apartment = Some(String::from_utf8(bytes.to_vec()).unwrap());
                    }
                    Some("block_id") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        updated_user.block_id = Some(String::from_utf8(bytes.to_vec()).unwrap().parse().unwrap());
                    }
                    Some("password") => {
                        let bytes = field.bytes(1024).await.unwrap().unwrap();
                        updated_user.password = Some(String::from_utf8(bytes.to_vec()).unwrap());
                    }
                    Some("photo") => {
                        let mut data = Vec::new();
                        while let Some(chunk) = field.next().await {
                            data.extend_from_slice(&chunk.unwrap());
                        }
                        updated_user.photo = Some(data);
                    }
                    _ => {}
                }
            },
            Err(err) => {
                error!("Error processing field: {:?}", err);
                return HttpResponse::BadRequest().finish();
            }
        }
    }

    // Hash password if it's provided in the update
    if let Some(ref pwd) = updated_user.password {
        updated_user.password = Some(hash(pwd, DEFAULT_COST).expect("Failed to hash password"));
    }

    let mut conn: PooledConnection<ConnectionManager<SqliteConnection>> = db.get().expect("Failed to get DB connection");

    // Execute the update in the database with conditional fields
    let result = web::block(move || {
        diesel::update(users::table.find(*user_id))
            .set(&updated_user) // Only non-None fields will be updated
            .execute(&mut conn)
    }).await;

    match result {
        Ok(_) => {
            info!("User updated successfully.");
            HttpResponse::Ok().json(json!({"message": "User updated successfully"}))
        },
        Err(err) => {
            error!("Failed to update user: {:?}", err);
            HttpResponse::InternalServerError().finish()
        },
    }
}

// Delete a user by ID
pub async fn delete_user(
    db: web::Data<DbConnection>,
    user_id: web::Path<i32>,
) -> impl Responder {
    info!("Deleting user with ID: {}", user_id);

    let mut conn: PooledConnection<ConnectionManager<SqliteConnection>> = match db.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let result = web::block(move || {
        diesel::delete(users::table.filter(users::id.eq(*user_id)))
            .execute(&mut conn)
    })
    .await;

    match result {
        Ok(Ok(_)) => HttpResponse::Ok().json(json!({"message": "User deleted successfully"})),
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
