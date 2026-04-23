use axum::{
    Router,
    middleware,
    routing::{get, post, delete},
};
use crate::handlers::link_handler;
use crate::middleware::auth_middleware::auth_middleware;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    let public_routes = Router::new()
        .route("/{short_code}", get(link_handler::redirect_link));

    let protected_routes = Router::new()
        .route("/links", post(link_handler::create_link))
        .route("/links/analytics", get(link_handler::get_daily_analytics))
        .route("/links/my-links", get(link_handler::get_my_links))
        .route("/links/{id}", delete(link_handler::delete_link))
        .route("/links/advanced-search", get(link_handler::get_advanced_search_results))
        .route_layer(middleware::from_fn(auth_middleware));

    public_routes.merge(protected_routes)
}
