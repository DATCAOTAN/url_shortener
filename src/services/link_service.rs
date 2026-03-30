use sqlx::PgPool;
use crate::repositories::link_repository;
use crate::models::link::Link;
use crate::error::{AppError, AppResult};
extern crate base62;

pub async fn create_link(pool: &PgPool, owner_id: Option<i64>, original_url: &str) -> Result<Link, sqlx::Error> {
    let next_id = link_repository::get_next_id(pool).await?;
    let short_code = base62::encode(next_id as u64);
    let link = link_repository::insert_url(pool, next_id, owner_id, original_url, &short_code).await?;
    Ok(link)
}

pub async fn get_original_url(pool: &PgPool, short_code: &str) -> AppResult<Option<String>> {
    match link_repository::get_orginal_url(pool, short_code).await {
        Ok(original_url) => Ok(original_url),
        Err(e) => {
            eprintln!("get_original_url database error: {}", e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn get_and_increment_click_count(pool: &PgPool, short_code: &str) -> AppResult<Option<String>> {
    let original_url = get_original_url(pool, short_code).await?;
    if original_url.is_some() {
        if let Err(e) = link_repository::increment_click_count(pool, short_code).await {
            eprintln!("increment_click_count error: {}", e);
        }
    }
    Ok(original_url)
}