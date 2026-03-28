use axum::{
    Router,
    routing::get,
};

use crate::handlers::health_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health/live", get(health_handler::liveness))
        .route("/health/ready", get(health_handler::readiness))
}
