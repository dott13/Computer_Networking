mod websocket;
mod tcp_server;
mod db;
mod handlers;
mod models;
mod schema;

use actix_web::{middleware, web, App, HttpServer};
use db::establish_connection;
use env_logger::Target;
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use actix::Addr;
use tokio::task;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let pool = establish_connection();

    // Shared state for chat rooms
    let chat_server = Arc::new(Mutex::new(HashMap::<String, Vec<Addr<websocket::Client>>>::new()));

    // Shared file resource for TCP server
    let shared_file = Arc::new(Mutex::new(String::new()));

    env_logger::Builder::new()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::Debug)
        .init();

    info!("Starting application...");

    // Start the TCP server
    let _tcp_server = {
        let shared_file = shared_file.clone();
        thread::spawn(move || {
            tcp_server::start_tcp_server(shared_file);
        })
    };

    // HTTP server on port 8080
    let http_server = {
        let pool = pool.clone();
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
    };

    // WebSocket server on port 8081
    let ws_server = {
        let chat_server = chat_server.clone();
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(chat_server.clone()))
                .route("/ws/{room}", web::get().to(websocket::start_chat)) // WebSocket route
        })
        .bind("0.0.0.0:8081")?
        .run()
    };

    // Run all servers
    task::spawn(http_server);
    ws_server.await
}
