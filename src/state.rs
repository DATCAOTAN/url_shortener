use sqlx::PgPool;
use deadpool_redis::Pool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Pool,
}

impl AppState {
    pub fn new(db: PgPool, redis: Pool) -> Self {
        Self {
            db,
            redis,
        }
    }
}
