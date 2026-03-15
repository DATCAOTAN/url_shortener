use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_db(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .connect(database_url)
        .await
        .expect("Failed to connect to the database")
}