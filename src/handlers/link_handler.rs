use axum::{extract::State, Json};
use sqlx::PgPool;
use crate::error::{AppError, AppResult};
use crate::services::link_service;
use crate::dtos::link::{CreateLinkRequest, LinkResponse};

pub async fn create_link(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateLinkRequest>,
) -> AppResult<Json<LinkResponse>> {
    match link_service::create_link(&pool, None, &payload.original_url).await {
        Ok(link) => Ok(Json(LinkResponse::from(link))),
        Err(e) => {
            eprintln!("create_link error: {}", e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn redirect(
    State(pool): State<PgPool>,
    axum::extract::Path(short_code): axum::extract::Path<String>,
) -> AppResult<axum::response::Redirect> {
    match link_service::get_and_increment_click_count(&pool, &short_code).await {
        Ok(Some(original_url)) => Ok(axum::response::Redirect::to(&original_url)),
        Ok(None) => Err(AppError::NotFound(format!("Short code {} not found", short_code))),
        Err(e) => {
            eprintln!("redirect error: {}", e);
            Err(e)
        }
    }
}