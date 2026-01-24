use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::models::Url;
use crate::services::base62_encode;
use crate::state::AppState;

/// URL service for database operations
pub struct UrlService;

impl UrlService {
    /// Create a new short URL
    ///
    /// Inserts the URL into the database, generates a Base62 short code,
    /// and updates the record with the code.
    pub async fn create(state: &Arc<AppState>, original_url: &str) -> AppResult<Url> {
        // Insert the URL and get the generated ID
        let row = sqlx::query_as::<_, Url>(
            r#"
            INSERT INTO urls (original_url, short_code, clicks, created_at)
            VALUES ($1, '', 0, NOW())
            RETURNING id, short_code, original_url, clicks, created_at
            "#,
        )
        .bind(original_url)
        .fetch_one(&state.db)
        .await?;

        // Generate short code from ID
        let short_code = base62_encode(row.id);

        // Update the record with the short code
        let url = sqlx::query_as::<_, Url>(
            r#"
            UPDATE urls 
            SET short_code = $1 
            WHERE id = $2
            RETURNING id, short_code, original_url, clicks, created_at
            "#,
        )
        .bind(&short_code)
        .bind(row.id)
        .fetch_one(&state.db)
        .await?;

        Ok(url)
    }

    /// Get a URL by its short code
    pub async fn get_by_code(state: &Arc<AppState>, short_code: &str) -> AppResult<Option<Url>> {
        let url = sqlx::query_as::<_, Url>(
            r#"
            SELECT id, short_code, original_url, clicks, created_at
            FROM urls
            WHERE short_code = $1
            "#,
        )
        .bind(short_code)
        .fetch_optional(&state.db)
        .await?;

        Ok(url)
    }

    /// Get a URL by its ID
    pub async fn get_by_id(state: &Arc<AppState>, id: i64) -> AppResult<Option<Url>> {
        let url = sqlx::query_as::<_, Url>(
            r#"
            SELECT id, short_code, original_url, clicks, created_at
            FROM urls
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

        Ok(url)
    }

    /// Increment the click count for a URL (async analytics)
    ///
    /// This is designed to be called via tokio::spawn to not block the redirect
    pub async fn increment_clicks(state: &Arc<AppState>, short_code: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE urls 
            SET clicks = clicks + 1 
            WHERE short_code = $1
            "#,
        )
        .bind(short_code)
        .execute(&state.db)
        .await?;

        Ok(())
    }

    /// List all URLs with pagination
    pub async fn list(state: &Arc<AppState>, limit: i64, offset: i64) -> AppResult<Vec<Url>> {
        let urls = sqlx::query_as::<_, Url>(
            r#"
            SELECT id, short_code, original_url, clicks, created_at
            FROM urls
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

        Ok(urls)
    }

    /// Get total count of URLs
    pub async fn count(state: &Arc<AppState>) -> AppResult<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM urls")
            .fetch_one(&state.db)
            .await?;

        Ok(row.0)
    }

    /// Delete a URL by short code
    pub async fn delete(state: &Arc<AppState>, short_code: &str) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM urls WHERE short_code = $1")
            .bind(short_code)
            .execute(&state.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
