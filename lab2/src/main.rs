extern crate diesel;

mod db;
mod handlers;
mod models;
mod schema;

use actix_web::{middleware, web, App, HttpServer};
use db::establish_connection;
use env_logger::Target;
use log::info;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let pool = establish_connection();

    env_logger::Builder::new()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::Debug)
        .init();

    info!("Starting application...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .route("/products", web::post().to(handlers::create_product))
            .route("/products", web::get().to(handlers::get_products))
            .route("/products/{id}", web::get().to(handlers::get_product))
            .route("/products/{id}", web::put().to(handlers::update_product))
            .route("/products/{id}", web::delete().to(handlers::delete_product))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}