use axum::{routing::{get, post}, Router};
use sqlx::PgPool;
use crate::handlers::link_handler;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/api/shorten", post(link_handler::create_link))
        .route("/s/{short_code}", get(link_handler::redirect)) 
}