use axum::{
    Extension,
    Json,
    extract::{Path, State},
};

use crate::dtos::claims::Claims;
use crate::dtos::link::{DeleteLinkResponse, LinkResponse};
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
        (status = 200, description = "List all users", body = [crate::dtos::user::UserResponse]),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 500, description = "Database error", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<UserResponse>>> {
    let users = user_service::list_users(&state.db)
        .await
        .map_err(AppError::Database)?;

    let response = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/admin/users/{id}",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Get user by id", body = crate::dtos::user::UserResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Database error", body = crate::error::ErrorResponse)
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
    responses(
        (status = 200, description = "List all links", body = [crate::dtos::link::LinkResponse]),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 500, description = "Database error", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_links(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<LinkResponse>>> {
    let links = link_service::get_all_links(&state.db)
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
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/admin/links/{id}",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(
        ("id" = i64, Path, description = "Link ID")
    ),
    responses(
        (status = 200, description = "Disable link", body = crate::dtos::link::DeleteLinkResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Link not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Database error", body = crate::error::ErrorResponse)
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
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Soft delete user", body = crate::dtos::user::LogoutResponse),
        (status = 400, description = "Bad request", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Database error", body = crate::error::ErrorResponse)
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
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Hard delete user", body = crate::dtos::user::LogoutResponse),
        (status = 400, description = "Bad request", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Database error", body = crate::error::ErrorResponse)
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
