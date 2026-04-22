use axum::{
    Router,
    middleware,
    routing::{delete, get},
};

use crate::handlers::admin_handler;
use crate::middleware::admin_middleware::admin_middleware;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(admin_handler::list_users))
        .route(
            "/admin/users/{id}",
            get(admin_handler::get_user_by_id).delete(admin_handler::soft_delete_user),
        )
        .route("/admin/users/{id}/hard", delete(admin_handler::hard_delete_user))
        .route("/admin/links", get(admin_handler::list_links))
        .route("/admin/links/{id}", delete(admin_handler::disable_link))
        .route("/admin/links/search", get(admin_handler::search_links))
        .route_layer(middleware::from_fn(admin_middleware))
}
