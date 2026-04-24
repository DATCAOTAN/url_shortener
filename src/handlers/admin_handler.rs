use axum::{
    Extension,
    Json,
    extract::{Path, Query, State},
};

use crate::dtos::claims::Claims;
use crate::dtos::link::{DeleteLinkResponse, LinkResponse, PaginationMetadata, PaginationQuery, PaginationResponse, SearchQuery};
use crate::dtos::user::{LogoutResponse, UserResponse};
use crate::error::{AppError, AppResult};
use crate::services::{link_service, user_service};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/admin/users",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List all users", body = [UserResponse]),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<UserResponse>>> {
    let users = user_service::list_users(&state.db).await?;

    let response = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/admin/users/{id}",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "User ID")),
    responses(
        (status = 200, description = "Get user by ID", body = UserResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_user_by_id(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> AppResult<Json<UserResponse>> {
    match user_service::get_user(&state.db, id).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!("User {} not found", id))),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    get,
    path = "/admin/links",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(
        ("current_page" = Option<u32>, Query, description = "Page number (>= 1)"),
        ("limit" = Option<u32>, Query, description = "Page size (>= 1)"),
        ("sort_by" = Option<String>, Query, description = "Sort order: clicks_desc or clicks_asc")
    ),
    responses(
        (status = 200, description = "List all links with pagination", body = PaginationResponse),
        (status = 400, description = "Invalid query", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_links(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<PaginationResponse>> {
    // Mac dinh phan trang: current_page=1, limit=10 neu client khong truyen hoac truyen <= 0.
    let current_page = pagination.current_page.filter(|page| *page > 0).unwrap_or(1);
    let limit = pagination.limit.filter(|size| *size > 0).unwrap_or(10);
    // Mac dinh sap xep giam dan theo luot click.
    let sort_by = pagination.sort_by.unwrap_or_else(|| "clicks_desc".to_string());

    let links = link_service::get_all_links(&state.db).await?;

    let mut response: Vec<LinkResponse> = links
        .into_iter()
        .map(|link| LinkResponse {
            id: link.id,
            short_code: link.short_code,
            original_url: link.original_url,
            title: link.title,
            click_count: link.click_count.unwrap_or(0),
            is_active: link.is_active.unwrap(),
            expires_at: link.expires_at,
        })
        .collect();

    // Chi chap nhan 2 gia tri sort_by: clicks_desc hoac clicks_asc.
    match sort_by.as_str() {
        "clicks_desc" => response.sort_by(|a, b| b.click_count.cmp(&a.click_count)),
        "clicks_asc" => response.sort_by(|a, b| a.click_count.cmp(&b.click_count)),
        _ => {
            return Err(AppError::BadRequest(
                "sort_by chi nhan mot trong hai gia tri: clicks_desc hoac clicks_asc".to_string(),
            ));
        }
    }

    let total_items = response.len() as i64;
    let total_pages = if total_items == 0 {
        0
    } else {
        ((total_items as u64 + limit as u64 - 1) / limit as u64) as u32
    };

    // Cong thuc offset: (page - 1) * limit de xac dinh vi tri bat dau cat mang.
    let offset = (current_page.saturating_sub(1) as u64).saturating_mul(limit as u64);
    let paged_data = response
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    // Metadata giup client biet tong so ban ghi/trang va thong tin truy van hien tai.
    let metadata = PaginationMetadata {
        limit,
        offset,
        sort_by,
        total_items,
        total_pages,
        current_page,
    };

    let response = PaginationResponse {
        data: paged_data,
        metadata,
    };

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/admin/links/search",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(
        ("min_clicks" = i64, Query, description = "Minimum click count"),
        ("is_active" = bool, Query, description = "Filter by active status")
    ),
    responses(
        (status = 200, description = "Search links by conditions", body = [LinkResponse]),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse)
    )
)]
pub async fn search_links(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<LinkResponse>>> {
    let links = link_service::get_links_with_min_clicks(
        &state.db,
        query.min_clicks,
        query.is_active,
    )
    .await?;

    let response = links
        .into_iter()
        .map(|link| LinkResponse {
            id: link.id,
            short_code: link.short_code,
            original_url: link.original_url,
            title: link.title,
            click_count: link.click_count.unwrap_or(0),
            is_active: link.is_active.unwrap(),
            expires_at: link.expires_at,
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/admin/links/{id}",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "Link ID")),
    responses(
        (status = 200, description = "Disable link by admin", body = DeleteLinkResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Link not found", body = crate::error::ErrorResponse)
    )
)]
pub async fn disable_link(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(link_id): Path<i64>,
) -> AppResult<Json<DeleteLinkResponse>> {
    match link_service::admin_soft_delete_link(&state.db, link_id).await {
        Ok(Some(link)) => {
            if let Err(e) = crate::services::cache_service::invalidate_cache(&state.redis, &link.short_code).await {
                tracing::warn!("Redis cache invalidate error: {:?}", e);
            }

            Ok(Json(DeleteLinkResponse {
                message: "Link disabled by admin".to_string(),
            }))
        }
        Ok(None) => Err(AppError::NotFound(format!("Link {} not found", link_id))),
        Err(e) => Err(AppError::Database(e)),
    }
}

#[utoipa::path(
    delete,
    path = "/admin/users/{id}",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "User ID")),
    responses(
        (status = 200, description = "Soft delete user", body = LogoutResponse),
        (status = 400, description = "Bad request", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse)
    )
)]
pub async fn soft_delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<i64>,
) -> AppResult<Json<LogoutResponse>> {
    let admin_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("Invalid admin ID in token".to_string()))?;

    if admin_id == user_id {
        return Err(AppError::BadRequest("Admin cannot delete own account".to_string()));
    }

    match user_service::admin_soft_delete_user(&state.db, user_id).await {
        Ok(Some(_)) => Ok(Json(LogoutResponse {
            message: "User disabled by admin".to_string(),
        })),
        Ok(None) => Err(AppError::NotFound(format!("User {} not found", user_id))),
        Err(e) => Err(AppError::Database(e)),
    }
}

#[utoipa::path(
    delete,
    path = "/admin/users/{id}/hard",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "User ID")),
    responses(
        (status = 200, description = "Hard delete user", body = LogoutResponse),
        (status = 400, description = "Bad request", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse)
    )
)]
pub async fn hard_delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<i64>,
) -> AppResult<Json<LogoutResponse>> {
    let admin_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::Unauthorized("Invalid admin ID in token".to_string()))?;

    if admin_id == user_id {
        return Err(AppError::BadRequest("Admin cannot delete own account".to_string()));
    }

    match user_service::admin_hard_delete_user(&state.db, user_id).await {
        Ok(true) => Ok(Json(LogoutResponse {
            message: "User deleted permanently by admin".to_string(),
        })),
        Ok(false) => Err(AppError::NotFound(format!("User {} not found", user_id))),
        Err(e) => Err(AppError::Database(e)),
    }
}
