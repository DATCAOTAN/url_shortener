use axum::{
    Extension,
    Json,
    extract::{Path, State},
};
use crate::error::{AppError, AppResult};
use crate::services::user_service;
use crate::dtos::claims::Claims;
use crate::dtos::user::{LoginResponse, LoginUser, LogoutRequest, LogoutResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterUser, UserResponse};
use crate::state::AppState;
use crate::utils::validation::{validate_email, validate_password, validate_username};

pub async fn get_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> AppResult<Json<UserResponse>> {
    let user_id = claims.sub.parse::<i64>().map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

    if user_id != id {
        return Err(AppError::Forbidden("Ban khong co quyen xem user nay".to_string()));
    }

    match user_service::get_user(&state.db, id).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!("User {} not found", id))),
        Err(e) => {
            eprintln!("get_user database error: {}", e);
            Err(e.into())
        }
    }
}

pub async fn get_me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<UserResponse>> {
    let user_id = claims.sub.parse::<i64>().map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

    match user_service::get_user(&state.db, user_id).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!("User {} not found", user_id))),
        Err(e) => {
            eprintln!("get_me database error: {}", e);
            Err(e.into())
        }
    }
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUser>,
) -> AppResult<Json<UserResponse>> {
    if !validate_username(&payload.username) {
        return Err(AppError::BadRequest("Username must be 3-50 characters".to_string()));
    }
    if !validate_email(&payload.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }
    if !validate_password(&payload.password) {
        return Err(AppError::BadRequest("Password must be 8-128 characters".to_string()));
    }

    match user_service::register_user(&state.db, &payload.username, &payload.email, &payload.password).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(e) => {
            eprintln!("register_user error: {}", e);
            Err(e.into())
        }
    }
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> AppResult<Json<LoginResponse>> {
    if !validate_email(&payload.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }
    if !validate_password(&payload.password) {
        return Err(AppError::BadRequest("Password must be 8-128 characters".to_string()));
    }

    match user_service::login_user(&state.db, &payload.email, &payload.password).await {
        Ok(login_response) => Ok(Json(login_response)),
        Err(sqlx::Error::RowNotFound) => Err(AppError::Unauthorized("Thong tin dang nhap sai".to_string())),
        Err(sqlx::Error::Io(io_err)) if io_err.to_string() == "PASSWORD_INVALID" => {
            Err(AppError::Unauthorized("Password sai".to_string()))
        }
        Err(e) => {
            eprintln!("login_user error: {}", e);
            Err(e.into())
        }
    }
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<RefreshTokenResponse>> {
    match user_service::refresh_access_token(&state.db, &payload.refresh_token).await {
        Ok(response) => Ok(Json(response)),
        Err(sqlx::Error::Io(io_err)) if io_err.to_string() == "REFRESH_TOKEN_INVALID" => {
            Err(AppError::Unauthorized("Refresh token khong hop le".to_string()))
        }
        Err(e) => {
            eprintln!("refresh_token error: {}", e);
            Err(e.into())
        }
    }
}

pub async fn logout_user(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> AppResult<Json<LogoutResponse>> {
    match user_service::logout_user(&state.db, &payload.refresh_token).await {
        Ok(_) => Ok(Json(LogoutResponse {
            message: "Logout thanh cong".to_string(),
        })),
        Err(sqlx::Error::Io(io_err)) if io_err.to_string() == "REFRESH_TOKEN_INVALID" => {
            Err(AppError::Unauthorized("Refresh token khong hop le".to_string()))
        }
        Err(e) => {
            eprintln!("logout_user error: {}", e);
            Err(e.into())
        }
    }
}
