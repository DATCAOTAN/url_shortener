use axum::{
    Router,
    middleware,
    routing::{get, post},
};
use crate::handlers::user_handler;
use crate::middleware::auth_middleware::auth_middleware;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    let public_routes = Router::new()
        .route("/register", post(user_handler::register_user))
        .route("/login", post(user_handler::login_user))
        .route("/refresh", post(user_handler::refresh_token))
        .route("/logout", post(user_handler::logout_user));

    let protected_routes = Router::new()
        .route("/users/me", get(user_handler::get_me))
        .route_layer(middleware::from_fn(auth_middleware));

    public_routes.merge(protected_routes)
}