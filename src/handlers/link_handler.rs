use axum::{
    Json,
    extract::{Path, State, Query},
    response::Redirect,
    Extension,
};
use crate::error::{AppError, AppResult};
use crate::services::{link_service, cache_service};
use crate::dtos::link::{CreateLinkRequest, LinkResponse, DeleteLinkResponse, DailyAnalyticsResponse};
use crate::dtos::claims::Claims;
use chrono::NaiveDate;
use crate::state::AppState;
use utoipa::ToSchema;
use crate::utils::validation::{validate_title, validate_url};
// use crate::models::link::Link;

#[utoipa::path(
    post,
    path = "/links",
    tag = "Links",
    security(("bearer_auth" = [])),
    request_body = CreateLinkRequest,
    responses(
        (status = 200, description = "Create short link", body = LinkResponse),
        (status = 400, description = "Invalid input", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_link(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateLinkRequest>,
) -> AppResult<Json<LinkResponse>> {
    let user_id = claims.sub.parse::<i64>().map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    if !validate_url(&payload.original_url) {
        return Err(AppError::BadRequest("Invalid URL (must be http/https)".to_string()));
    }
    if let Some(title) = payload.title.as_deref() {
        if !validate_title(title) {
            return Err(AppError::BadRequest("Title must be 1-255 characters".to_string()));
        }
    }
    
    let link = link_service::create_short_link(&state.db, &payload.original_url, Some(user_id), payload.title).await
        .map_err(AppError::Database)?;

    Ok(Json(LinkResponse {
        id: link.id,
        short_code: link.short_code,
        original_url: link.original_url,
        title: link.title,
        click_count: link.click_count.unwrap_or(0),
    }))
}

#[utoipa::path(
    get,
    path = "/{short_code}",
    tag = "Links",
    params(("short_code" = String, Path, description = "Short code")),
    responses(
        (status = 307, description = "Temporary redirect"),
        (status = 404, description = "Short code not found", body = crate::error::ErrorResponse)
    )
)]
pub async fn redirect_link(
    State(state): State<AppState>,
    Path(short_code): Path<String>,
) -> AppResult<Redirect> {
    match cache_service::get_cached_url(&state.redis, &short_code).await {
        Ok(Some(url)) => {
            link_service::track_click_by_short_code(&state.db, &short_code).await;
            return Ok(Redirect::to(&url));
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!("Redis cache read error: {:?}", e);
        }
    }

    match link_service::get_original_url(&state.db, &short_code).await {
        Ok(Some(url)) => {
            if let Err(e) = cache_service::set_cached_url(&state.redis, &short_code, &url).await {
                tracing::warn!("Redis cache write error: {:?}", e);
            }
            Ok(Redirect::to(&url))
        }
        Ok(None) => Err(AppError::NotFound(format!("Link {} not found", short_code))),
        Err(e) => Err(AppError::Database(e)),
    }
}

#[utoipa::path(
    get,
    path = "/links/my-links",
    tag = "Links",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List my links", body = [LinkResponse]),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_my_links(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<LinkResponse>>> {
    let user_id = claims.sub.parse::<i64>().map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let links = link_service::get_user_links(&state.db, user_id).await
        .map_err(AppError::Database)?;

    let response = links.into_iter().map(|link| LinkResponse {
        id: link.id,
        short_code: link.short_code,
        original_url: link.original_url,
        title: link.title,
        click_count: link.click_count.unwrap_or(0),
    }).collect();

    Ok(Json(response))
}

#[derive(serde::Deserialize, ToSchema)]
pub struct AnalyticsQuery {
    pub from: String,
    pub to: String,
}

#[utoipa::path(
    delete,
    path = "/links/{id}",
    tag = "Links",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "Link ID")),
    responses(
        (status = 200, description = "Soft delete success", body = DeleteLinkResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Link not found", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_link(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(link_id): Path<i64>,
) -> AppResult<Json<DeleteLinkResponse>> {
    let user_id = claims.sub.parse::<i64>().map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    match link_service::soft_delete_link(&state.db, user_id, link_id).await {
        Ok(Some(link)) => {
            if let Err(e) = cache_service::invalidate_cache(&state.redis, &link.short_code).await {
                tracing::warn!("Redis cache invalidate error: {:?}", e);
            }
            Ok(Json(DeleteLinkResponse {
                message: "Link disabled".to_string(),
            }))
        }
        Ok(None) => Err(AppError::NotFound(format!("Link {} not found", link_id))),
        Err(e) => Err(AppError::Database(e)),
    }
}

#[utoipa::path(
    get,
    path = "/links/analytics",
    tag = "Links",
    security(("bearer_auth" = [])),
    params(
        ("from" = String, Query, description = "Start date (YYYY-MM-DD)"),
        ("to" = String, Query, description = "End date (YYYY-MM-DD)")
    ),
    responses(
        (status = 200, description = "Daily analytics", body = [DailyAnalyticsResponse]),
        (status = 400, description = "Invalid date range", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_daily_analytics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<AnalyticsQuery>,
) -> AppResult<Json<Vec<DailyAnalyticsResponse>>> {
    let user_id = claims.sub.parse::<i64>().map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let from_date = NaiveDate::parse_from_str(&params.from, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid from date".to_string()))?;
    let to_date = NaiveDate::parse_from_str(&params.to, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid to date".to_string()))?;

    if from_date > to_date {
        return Err(AppError::BadRequest("from must be <= to".to_string()));
    }

    let totals = link_service::get_daily_analytics(&state.db, user_id, from_date, to_date)
        .await
        .map_err(AppError::Database)?;

    let response = totals
        .into_iter()
        .map(|item| DailyAnalyticsResponse {
            date: item.date.format("%Y-%m-%d").to_string(),
            total_clicks: item.total_clicks,
        })
        .collect();

    Ok(Json(response))
}
