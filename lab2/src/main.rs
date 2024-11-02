use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;

mod routes;
mod controllers;
mod db;
mod models;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from the .env file
    dotenv().ok();
    env_logger::init();

    let pool = db::establish_connection();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) // Make the pool available to all handlers
            .configure(routes::user_routes::user_routes()) // Register user routes
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
