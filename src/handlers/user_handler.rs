use axum::{
    Extension,
    Json,
    extract::{Path, State},
};
use sqlx::PgPool;
use crate::error::{AppError, AppResult};
use crate::services::user_service;
use crate::dtos::claims::Claims;
use crate::dtos::user::{LoginResponse, LoginUser, LogoutRequest, LogoutResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterUser, UserResponse};

pub async fn get_user(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> AppResult<Json<UserResponse>> {
    if claims.id != id {
        return Err(AppError::Forbidden("Ban khong co quyen xem user nay".to_string()));
    }

    match user_service::get_user(&pool, id).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!("User {} not found", id))),
        Err(e) => {
            eprintln!("get_user database error: {}", e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn get_me(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<UserResponse>> {
    match user_service::get_user(&pool, claims.id).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!("User {} not found", claims.id))),
        Err(e) => {
            eprintln!("get_me database error: {}", e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn register_user(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterUser>,
) -> AppResult<Json<UserResponse>> {
    match user_service::register_user(&pool, &payload.username, &payload.email, &payload.password).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(e) => {
            eprintln!("register_user error: {}", e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn login_user(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> AppResult<Json<LoginResponse>> {
    match user_service::login_user(&pool, &payload.email, &payload.password).await {
        Ok(login_response) => Ok(Json(login_response)),
        Err(sqlx::Error::RowNotFound) => Err(AppError::Unauthorized("Thong tin dang nhap sai".to_string())),
        Err(sqlx::Error::Io(io_err)) if io_err.to_string() == "PASSWORD_INVALID" => {
            Err(AppError::Unauthorized("Password sai".to_string()))
        }
        Err(e) => {
            eprintln!("login_user error: {}", e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn refresh_token(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<RefreshTokenResponse>> {
    match user_service::refresh_access_token(&pool, &payload.refresh_token).await {
        Ok(response) => Ok(Json(response)),
        Err(sqlx::Error::Io(io_err)) if io_err.to_string() == "REFRESH_TOKEN_INVALID" => {
            Err(AppError::Unauthorized("Refresh token khong hop le".to_string()))
        }
        Err(e) => {
            eprintln!("refresh_token error: {}", e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn logout_user(
    State(pool): State<PgPool>,
    Json(payload): Json<LogoutRequest>,
) -> AppResult<Json<LogoutResponse>> {
    match user_service::logout_user(&pool, &payload.refresh_token).await {
        Ok(_) => Ok(Json(LogoutResponse {
            message: "Logout thanh cong".to_string(),
        })),
        Err(sqlx::Error::Io(io_err)) if io_err.to_string() == "REFRESH_TOKEN_INVALID" => {
            Err(AppError::Unauthorized("Refresh token khong hop le".to_string()))
        }
        Err(e) => {
            eprintln!("logout_user error: {}", e);
            Err(AppError::Database(e))
        }
    }
}