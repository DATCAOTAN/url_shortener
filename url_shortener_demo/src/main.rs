mod config;
mod error;
mod handlers;
mod models;
mod services;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::path::PathBuf;
use tower_http::services::ServeDir;

use crate::config::Config;
use crate::handlers::{create_url, health_check, list_urls, redirect};
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Configuration loaded successfully");

    // Create PostgreSQL connection pool
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");
    tracing::info!("Connected to PostgreSQL");

    // Run database migrations (create tables if not exist)
    run_migrations(&db).await?;

    // Create Redis client
    let redis = redis::Client::open(config.redis_url.clone())
        .expect("Failed to create Redis client");
    
    // Test Redis connection
    let mut redis_conn = redis
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    let _: () = redis::cmd("PING")
        .query_async(&mut redis_conn)
        .await
        .expect("Failed to ping Redis");
    tracing::info!("Connected to Redis");

    // Create shared application state
    let state = AppState::new(db, redis, config.clone());

    // Setup static file serving
    let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");

    // Build the router
    let app = Router::new()
        // API routes
        .route("/api/shorten", post(create_url))
        .route("/api/urls", get(list_urls))
        .route("/api/health", get(health_check))
        // Redirect route (must be after API routes)
        .route("/{code}", get(redirect))
        // Static files and index
        .nest_service("/static", ServeDir::new(&static_dir))
        .route("/", get(serve_index))
        // Application state
        .with_state(state);

    // Start the server
    let addr = config.server_addr();
    tracing::info!("🚀 Server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve the index.html file
async fn serve_index() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/index.html"))
}

/// Run database migrations
async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Running database migrations...");

    // Create urls table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS urls (
            id BIGSERIAL PRIMARY KEY,
            short_code VARCHAR(20) NOT NULL DEFAULT '',
            original_url TEXT NOT NULL,
            clicks BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on short_code
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_urls_short_code ON urls(short_code)")
        .execute(pool)
        .await?;

    // Create index on created_at
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_urls_created_at ON urls(created_at DESC)")
        .execute(pool)
        .await?;

    tracing::info!("Database migrations completed");
    Ok(())
}
