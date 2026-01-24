use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::services::{CacheService, UrlService};
use crate::state::SharedState;

/// GET /:code - High-performance redirect with Cache-Aside pattern
///
/// 1. Check Redis cache first
/// 2. On cache miss, query PostgreSQL
/// 3. Update cache asynchronously
/// 4. Increment clicks via tokio::spawn (non-blocking analytics)
/// 5. Return 302 redirect
pub async fn redirect(
    State(state): State<SharedState>,
    Path(code): Path<String>,
) -> AppResult<Response> {
    // Use Cache-Aside pattern to get the URL
    let result = CacheService::get_url_with_cache_aside(&state, &code).await?;

    match result {
        Some((original_url, was_cache_hit)) => {
            // Log cache hit/miss for monitoring
            if was_cache_hit {
                tracing::debug!("Cache HIT for code: {}", code);
            } else {
                tracing::debug!("Cache MISS for code: {}", code);
            }

            // Async analytics: increment clicks without blocking the redirect
            let state_clone = Arc::clone(&state);
            let code_clone = code.clone();
            tokio::spawn(async move {
                if let Err(e) = UrlService::increment_clicks(&state_clone, &code_clone).await {
                    tracing::warn!("Failed to increment clicks for {}: {:?}", code_clone, e);
                }
            });

            // Return 302 Found redirect
            let response = Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, &original_url)
                .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                .body(axum::body::Body::empty())
                .map_err(|e| AppError::Internal(e.to_string()))?;

            Ok(response)
        }
        None => Err(AppError::NotFound(format!(
            "Short URL '{}' not found",
            code
        ))),
    }
}
