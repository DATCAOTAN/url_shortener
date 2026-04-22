use std::{collections::HashMap, sync::Arc, time::Instant};

use sqlx::PgPool;
use deadpool_redis::Pool;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Pool,
    pub cooldown: Arc<Mutex<HashMap<i64, Instant>>>,
}

impl AppState {
    pub fn new(db: PgPool, redis: Pool) -> Self {
        Self {
            db,
            redis,
            cooldown: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
