use axum::{
    Router,
    routing::get,
};

use crate::handlers::demo_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/demo/spawn", get(demo_handler::spawn_demo))
}
