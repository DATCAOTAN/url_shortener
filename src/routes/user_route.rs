use axum::{Router, routing::{get, post}};
use crate::handlers::user_handler;

pub fn routes() -> Router<sqlx::PgPool> {
    Router::new()
        .route("/users/{id}", get(user_handler::get_user))
        .route("/register", post(user_handler::register_user))
        .route("/login", post(user_handler::login_user))
        .route("/refresh", post(user_handler::refresh_token))
}