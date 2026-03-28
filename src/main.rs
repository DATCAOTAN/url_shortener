use axum::{Router, serve};
use dotenvy::dotenv;
use std::env;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use axum::Json;

use crate::db::init_db;
use crate::middleware::rate_limit_middleware::{RateLimiter, rate_limit_middleware};
use crate::routes::{admin_route, health_route, link_route, user_route};
use crate::state::AppState;
use deadpool_redis::{Config as RedisConfig, Runtime, PoolConfig};
use axum::http::{HeaderValue, Method};
use axum::middleware as axum_middleware;

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
mod docs;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

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
    let rate_limiter = RateLimiter::from_env();

    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".to_string());

    let cors_layer = if allowed_origins.trim() == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
            .allow_headers(Any)
            .max_age(Duration::from_secs(3600))
    } else {
        let origins: Vec<HeaderValue> = allowed_origins
            .split(',')
            .filter_map(|origin| HeaderValue::from_str(origin.trim()).ok())
            .collect();

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
            .allow_headers(Any)
            .max_age(Duration::from_secs(3600))
    };

    let app = Router::new()
        .route("/", axum::routing::get(|| async { "URL Shortener API" }))
        .route(
            "/api-docs/openapi.json",
            axum::routing::get(|| async { Json(docs::ApiDoc::openapi()) }),
        )
        .route(
            "/docs",
            axum::routing::get(|| async {
                "OpenAPI JSON is available at /api-docs/openapi.json"
            }),
        )
        .merge(health_route::routes())
        .merge(user_route::routes())
        .merge(link_route::routes())
        .merge(admin_route::routes())
        .layer(axum_middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(cors_layer)
        .layer(TraceLayer::new_for_http())
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
