use axum::{
    Json,
    extract::State,
};
use serde::Serialize;

use crate::error::AppResult;
use crate::state::AppState;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ReadyResponse {
    pub status: String,
    pub database: String,
    pub redis: String,
}

#[utoipa::path(
    get,
    path = "/health/live",
    tag = "Health",
    responses(
        (status = 200, description = "Liveness probe", body = HealthResponse)
    )
)]
pub async fn liveness() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Readiness probe", body = ReadyResponse),
        (status = 503, description = "Service dependency is unavailable", body = crate::error::ErrorResponse)
    )
)]
pub async fn readiness(State(state): State<AppState>) -> AppResult<Json<ReadyResponse>> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await?;

    let mut redis_conn = state.redis.get().await?;

    deadpool_redis::redis::cmd("PING")
        .query_async::<_, String>(&mut redis_conn)
        .await?;

    Ok(Json(ReadyResponse {
        status: "ok".to_string(),
        database: "up".to_string(),
        redis: "up".to_string(),
    }))
}
