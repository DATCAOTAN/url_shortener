use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};

use crate::error::{AppError, AppResult};
use crate::models::{CreateUrlRequest, CreateUrlResponse, Url, UrlListResponse};
use crate::services::UrlService;
use crate::state::SharedState;

/// POST /api/shorten - Create a new short URL
pub async fn create_url(
    State(state): State<SharedState>,
    Json(payload): Json<CreateUrlRequest>,
) -> AppResult<Json<CreateUrlResponse>> {
    // Validate the URL
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Create the short URL
    let url = UrlService::create(&state, &payload.url).await?;

    // Build the full short URL
    let short_url = format!(
        "http://localhost:{}/{}",
        state.config.server_port,
        url.short_code
    );

    Ok(Json(CreateUrlResponse {
        id: url.id,
        short_code: url.short_code,
        short_url,
        original_url: url.original_url,
    }))
}

/// GET /api/urls - List all URLs
pub async fn list_urls(
    State(state): State<SharedState>,
) -> AppResult<Json<UrlListResponse>> {
    let urls = UrlService::list(&state, 100, 0).await?;
    let total = urls.len();

    Ok(Json(UrlListResponse { urls, total }))
}

/// GET /api/health - Health check endpoint
pub async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}
