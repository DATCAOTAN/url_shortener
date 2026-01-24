use redis::AsyncCommands;
use std::sync::Arc;

use crate::error::AppResult;
use crate::state::AppState;

/// Cache service implementing the Cache-Aside pattern
pub struct CacheService;

/// Cache key prefix for URL mappings
const URL_CACHE_PREFIX: &str = "url:";
/// Default TTL for cached URLs (1 hour)
const CACHE_TTL_SECONDS: u64 = 3600;

impl CacheService {
    /// Get a cached URL by its short code
    ///
    /// Returns None if not found in cache (cache miss)
    pub async fn get_cached_url(
        state: &Arc<AppState>,
        short_code: &str,
    ) -> AppResult<Option<String>> {
        let mut conn = state
            .redis
            .get_multiplexed_async_connection()
            .await?;

        let cache_key = format!("{}{}", URL_CACHE_PREFIX, short_code);
        let result: Option<String> = conn.get(&cache_key).await?;

        Ok(result)
    }

    /// Store a URL in the cache with TTL
    pub async fn set_cached_url(
        state: &Arc<AppState>,
        short_code: &str,
        original_url: &str,
    ) -> AppResult<()> {
        let mut conn = state
            .redis
            .get_multiplexed_async_connection()
            .await?;

        let cache_key = format!("{}{}", URL_CACHE_PREFIX, short_code);
        conn.set_ex::<_, _, ()>(&cache_key, original_url, CACHE_TTL_SECONDS).await?;

        Ok(())
    }

    /// Delete a URL from the cache (for updates or deletions)
    pub async fn invalidate_cache(
        state: &Arc<AppState>,
        short_code: &str,
    ) -> AppResult<()> {
        let mut conn = state
            .redis
            .get_multiplexed_async_connection()
            .await?;

        let cache_key = format!("{}{}", URL_CACHE_PREFIX, short_code);
        conn.del::<_, ()>(&cache_key).await?;

        Ok(())
    }

    /// Get URL with Cache-Aside pattern: check cache first, then DB
    /// Returns (original_url, was_cache_hit)
    pub async fn get_url_with_cache_aside(
        state: &Arc<AppState>,
        short_code: &str,
    ) -> AppResult<Option<(String, bool)>> {
        // Try cache first
        match Self::get_cached_url(state, short_code).await {
            Ok(Some(url)) => {
                // Cache hit!
                return Ok(Some((url, true)));
            }
            Ok(None) => {
                // Cache miss, will query DB below
            }
            Err(e) => {
                // Log Redis error but continue to DB (graceful degradation)
                tracing::warn!("Redis error (falling back to DB): {:?}", e);
            }
        }

        // Cache miss: query database
        use crate::services::UrlService;
        if let Some(url_record) = UrlService::get_by_code(state, short_code).await? {
            // Update cache asynchronously (fire and forget)
            let state_clone = Arc::clone(state);
            let short_code = short_code.to_string();
            let original_url = url_record.original_url.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::set_cached_url(&state_clone, &short_code, &original_url).await {
                    tracing::warn!("Failed to update cache: {:?}", e);
                }
            });

            return Ok(Some((url_record.original_url, false)));
        }

        Ok(None)
    }
}
