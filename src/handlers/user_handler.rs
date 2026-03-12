use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::PgPool;
use crate::services::user_service;
use crate::models::user::User;
use crate::dtos::user::{LoginResponse, LoginUser, RefreshTokenRequest, RefreshTokenResponse, RegisterUser};

pub async fn get_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<User>, (StatusCode, String)> {
    match user_service::get_user(&pool, id).await {
        Ok(user) => Ok(Json(user)),
        Err(sqlx::Error::RowNotFound) => {
            Err((StatusCode::NOT_FOUND, format!("User {} not found", id)))
        }
        Err(e) => {
            eprintln!("get_user database error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()))
        }
    }
}

pub async fn register_user(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    match user_service::register_user(&pool, &payload.username, &payload.email, &payload.password).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            eprintln!("register_user error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()))
        }
    }
}

pub async fn login_user(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    match user_service::login_user(&pool, &payload.email, &payload.password).await {
        Ok(login_response) => Ok(Json(login_response)),
        Err(sqlx::Error::RowNotFound) => {
            Err((StatusCode::UNAUTHORIZED, "Thong tin dang nhap sai".to_string()))
        }
        Err(sqlx::Error::Io(io_err)) if io_err.to_string() == "PASSWORD_INVALID" => {
            Err((StatusCode::UNAUTHORIZED, "Password sai".to_string()))
        }
        Err(e) => {
            eprintln!("login_user error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()))
        }
    }
}

pub async fn refresh_token(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, (StatusCode, String)> {
    match user_service::refresh_access_token(&pool, &payload.refresh_token).await {
        Ok(response) => Ok(Json(response)),
        Err(sqlx::Error::Io(io_err)) if io_err.to_string() == "REFRESH_TOKEN_INVALID" => {
            Err((StatusCode::UNAUTHORIZED, "Refresh token khong hop le".to_string()))
        }
        Err(e) => {
            eprintln!("refresh_token error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()))
        }
    }
}