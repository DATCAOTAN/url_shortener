use axum::{Router, serve};
use dotenvy::dotenv;
use std::env;

use crate::db::init_db;
use crate::routes::user_route;

mod dtos;
mod db;
mod routes;
mod handlers;
mod error;
mod middleware;
mod services;
mod repositories;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e) => {
            eprintln!("DATABASE_URL not set: {}", e);
            return;
        }
    };

    let db_pool = init_db(&database_url).await;

    println!("Đã kết nối database thành công!");

    // Tạo router với một endpoint đơn giản
    let app = Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }))
        .merge(user_route::routes())
        .with_state(db_pool);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:8080").await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind 127.0.0.1:8080: {}", e);
            return;
        }
    };
    println!("Server started at http://127.0.0.1:8080");

    if let Err(e) = serve(listener, app).await {
        eprintln!("Server error: {}", e);
    }
}