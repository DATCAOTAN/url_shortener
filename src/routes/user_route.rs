use axum::{
    Router,
    middleware,
    routing::{get, post},
};
use crate::handlers::user_handler;
use crate::middleware::auth_middleware::auth_guard;

pub fn routes() -> Router<sqlx::PgPool> {
    let public_routes = Router::new()
        .route("/register", post(user_handler::register_user))
        .route("/login", post(user_handler::login_user))
        .route("/refresh", post(user_handler::refresh_token));

    let protected_routes = Router::new()
        .route("/users/me", get(user_handler::get_me))
        .route("/users/{id}", get(user_handler::get_user))
        .route_layer(middleware::from_fn(auth_guard));

    public_routes.merge(protected_routes)
}