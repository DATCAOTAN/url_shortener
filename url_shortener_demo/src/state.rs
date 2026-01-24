use redis::Client as RedisClient;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;

/// Application state for dependency injection via Axum's State extractor
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisClient,
    pub config: Config,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new(db: PgPool, redis: RedisClient, config: Config) -> Arc<Self> {
        Arc::new(Self { db, redis, config })
    }
}

/// Type alias for the shared state used in handlers
pub type SharedState = Arc<AppState>;
