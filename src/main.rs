use axum::{Router, serve};
use dotenvy::dotenv;
use std::env;

use crate::db::init_db;
use crate::routes::{user_route, link_route};
use crate::state::AppState;
use deadpool_redis::{Config as RedisConfig, Runtime, PoolConfig};

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
mod state;

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

    let db_pool = init_db(&database_url).await.expect("Failed to init db");

    println!("Đã kết nối database thành công!");

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis_max = env::var("REDIS_POOL_MAX")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(32);
    let mut redis_cfg = RedisConfig::from_url(redis_url);
    redis_cfg.pool = Some(PoolConfig::new(redis_max));

    let redis_pool = match redis_cfg.create_pool(Some(Runtime::Tokio1)) {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to create Redis pool: {}", e);
            return;
        }
    };

    let state = AppState::new(db_pool, redis_pool);

    // Tạo router với một endpoint đơn giản
    let app = Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }))
        .merge(user_route::routes())
        .merge(link_route::routes())
        .with_state(state);

    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind {}: {}", bind_addr, e);
            return;
        }
    };
    println!("Server started at http://{}", bind_addr);

    if let Err(e) = serve(listener, app).await {
        eprintln!("Server error: {}", e);
    }
}
