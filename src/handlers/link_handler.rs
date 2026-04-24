use axum::{
    Json,
    extract::{Path, Query, State},
    response::Redirect,
    Extension,
};
use chrono::NaiveDate;

use crate::dtos::claims::Claims;
use crate::dtos::link::{AdvancedSearchQuery, AnalyticsQuery, CreateLinkRequest, DailyAnalyticsResponse, DeleteLinkResponse, LinkResponse, ListLinksQuery};
use crate::error::{AppError, AppResult};
use crate::services::{cache_service, link_service};
use std::env;
use crate::state::AppState;
use crate::utils::validation::{validate_title, validate_url};

#[utoipa::path(
    post,
    path = "/links",
    tag = "Links",
    security(("bearer_auth" = [])),
    request_body = CreateLinkRequest,
    responses(
        (status = 200, description = "Create short link", body = LinkResponse),
        (status = 400, description = "Invalid input", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 429, description = "Cooldown active", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_link(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateLinkRequest>,
) -> AppResult<Json<LinkResponse>> {
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let cooldown_seconds = env::var("LINK_CREATE_COOLDOWN")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5);

    let allowed = cache_service::try_acquire_link_cooldown(&state.redis, user_id, cooldown_seconds)
        .await
        .map_err(|e| AppError::Internal(format!("Cooldown cache error: {e}")))?;

    if !allowed {
        return Err(AppError::TooManyRequests(
            "Please wait before creating another link".to_string(),
        ));
    }

    if !validate_url(&payload.original_url) {
        return Err(AppError::BadRequest("Invalid URL (must be http/https)".to_string()));
    }
    if let Some(title) = payload.title.as_deref() {
        if !validate_title(title) {
            return Err(AppError::BadRequest("Title must be 1-255 characters".to_string()));
        }
    }
    if let Some(ttl_seconds) = payload.ttl_seconds {
        if ttl_seconds <= 0 {
            return Err(AppError::BadRequest("ttl_seconds must be > 0".to_string()));
        }
    }

    let link = link_service::create_short_link(
        &state.db,
        &payload.original_url,
        Some(user_id),
        payload.title,
        payload.ttl_seconds,
    )
        .await
        .map_err(AppError::Database)?;

    Ok(Json(LinkResponse {
        id: link.id,
        short_code: link.short_code,
        original_url: link.original_url,
        title: link.title,
        click_count: link.click_count.unwrap_or(0),
        is_active: link.is_active,
        expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
        created_at: link.created_at.to_rfc3339(),
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
        Ok(Some(url)) => return Ok(Redirect::to(&url)),
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
        Ok(None) => {
            if let Ok(Some(link)) = link_service::get_link_details(&state.db, &short_code).await {
                if let Some(expires_at) = link.expires_at {
                    if expires_at <= chrono::Utc::now() {
                        return Err(AppError::NotFound(format!("Link {} has expired", short_code)));
                    }
                }
                if let Some(false) = link.is_active {
                    return Err(AppError::NotFound(format!("Link {} is disabled", short_code)));
                }
            }
            Err(AppError::NotFound(format!("Link {} not found", short_code)))
        }
        Err(e) => Err(AppError::Database(e)),
    }
}

#[utoipa::path(
    get,
    path = "/links/my-links",
    tag = "Links",
    security(("bearer_auth" = [])),
    params(ListLinksQuery),
    responses(
        (status = 200, description = "List my links", body = [LinkResponse]),
        (status = 400, description = "Invalid pagination/sorting", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_my_links(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListLinksQuery>,
) -> AppResult<Json<Vec<LinkResponse>>> {
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);
    if page < 1 {
        return Err(AppError::BadRequest("page must be >= 1".to_string()));
    }
    if page_size < 1 || page_size > 100 {
        return Err(AppError::BadRequest("page_size must be between 1 and 100".to_string()));
    }
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    if !matches!(sort_by, "created_at" | "click_count" | "title") {
        return Err(AppError::BadRequest("sort_by must be one of: created_at, click_count, title".to_string()));
    }
    let sort_order = params.sort_order.as_deref().unwrap_or("desc");
    if !sort_order.eq_ignore_ascii_case("asc") && !sort_order.eq_ignore_ascii_case("desc") {
        return Err(AppError::BadRequest("sort_order must be asc or desc".to_string()));
    }

    let links = link_service::get_user_links(&state.db, user_id, page, page_size, sort_by, sort_order)
        .await
        .map_err(AppError::Database)?;

    let response = links
        .into_iter()
        .map(|link| LinkResponse {
            id: link.id,
            short_code: link.short_code,
            original_url: link.original_url,
            title: link.title,
            click_count: link.click_count.unwrap_or(0),
            is_active: link.is_active,
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            created_at: link.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
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
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

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
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

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

#[utoipa::path(
    get,
    path = "/links/advanced-search",
    tag = "Links",
    security(("bearer_auth" = [])),
    params(AdvancedSearchQuery),
    responses(
        (status = 200, description = "Search results", body = [LinkResponse]),
        (status = 400, description = "Invalid query", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse)
    )
)]
pub async fn advanced_search_links(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<AdvancedSearchQuery>,
) -> AppResult<Json<Vec<LinkResponse>>> {
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    if let (Some(min), Some(max)) = (params.min_clicks, params.max_clicks) {
        if min > max {
            return Err(AppError::BadRequest("min_clicks must be <= max_clicks".to_string()));
        }
    }

    let from_date = match params.from.as_deref() {
        Some(value) => Some(
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid from date".to_string()))?,
        ),
        None => None,
    };

    let to_date = match params.to.as_deref() {
        Some(value) => Some(
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid to date".to_string()))?,
        ),
        None => None,
    };

    if let (Some(from), Some(to)) = (from_date, to_date) {
        if from > to {
            return Err(AppError::BadRequest("from must be <= to".to_string()));
        }
    }

    let links = link_service::advanced_search_links(
        &state.db,
        user_id,
        params.min_clicks,
        params.max_clicks,
        from_date,
        to_date,
        params.domain.clone(),
    )
    .await
    .map_err(AppError::Database)?;

    let response = links
        .into_iter()
        .map(|link| LinkResponse {
            id: link.id,
            short_code: link.short_code,
            original_url: link.original_url,
            title: link.title,
            click_count: link.click_count.unwrap_or(0),
            is_active: link.is_active,
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            created_at: link.created_at.to_rfc3339(),
        })
        .collect();
    Ok(Json(response))
}


