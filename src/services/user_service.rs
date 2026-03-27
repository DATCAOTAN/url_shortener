use chrono::{Duration, Utc};
use sqlx::PgPool;
use crate::repositories::user_repository;
use crate::models::user::User;
use crate::dtos::user::{LoginResponse, RefreshTokenResponse};
use crate::utils::jwt::{encode_access_token, encode_refresh_token, decode_refresh_token};

pub async fn get_user(pool: &PgPool, id: i64) -> Result<User, sqlx::Error> {
    match user_repository::find_by_id(pool, id).await? {
        Some(user) => Ok(user),
        None => Err(sqlx::Error::RowNotFound),
    }
}

pub async fn register_user(pool: &PgPool, username: &str, email: &str, password: &str) -> Result<User, sqlx::Error> {
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| sqlx::Error::Io(std::io::Error::other(format!("bcrypt error: {e}"))))?;
    let user = user_repository::register(pool, username, email, &password_hash).await?;
    Ok(user)
}

pub async fn login_user(pool: &PgPool, email: &str, password: &str) -> Result<LoginResponse, sqlx::Error> {
    let user = match user_repository::find_by_email(pool, email).await? {
        Some(user) => user,
        None => return Err(sqlx::Error::RowNotFound),
    };

    let is_valid = bcrypt::verify(password, &user.password_hash)
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("PASSWORD_INVALID")))?;

    if !is_valid {
        return Err(sqlx::Error::Io(std::io::Error::other("PASSWORD_INVALID")));
    }

    let access_token = encode_access_token(user.id.to_string(), user.role.clone())
        .map_err(|e| sqlx::Error::Io(std::io::Error::other(format!("Access JWT encode error: {e}"))))?;

    let now = Utc::now();
    let (refresh_token, refresh_exp_timestamp) = encode_refresh_token(user.id.to_string(), user.role.clone(), now.timestamp())
        .map_err(|e| sqlx::Error::Io(std::io::Error::other(format!("Refresh JWT encode error: {e}"))))?;
    
    let refresh_exp = DateTime::from_timestamp(refresh_exp_timestamp, 0).unwrap_or(now + Duration::days(30));

    user_repository::save_refresh_token(pool, user.id, &refresh_token, refresh_exp).await?;

    Ok(LoginResponse {
        access_token,
        refresh_token,
    })
}

pub async fn refresh_access_token(
    pool: &PgPool,
    refresh_token: &str,
) -> Result<RefreshTokenResponse, sqlx::Error> {
    let token_data = decode_refresh_token(refresh_token)
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")))?;

    let is_active = user_repository::is_refresh_token_active(pool, refresh_token).await?;
    if !is_active {
        return Err(sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")));
    }

    let access_token = encode_access_token(token_data.sub, token_data.role)
        .map_err(|e| sqlx::Error::Io(std::io::Error::other(format!("Access JWT encode error: {e}"))))?;

    Ok(RefreshTokenResponse { access_token })
}

pub async fn logout_user(pool: &PgPool, refresh_token: &str) -> Result<(), sqlx::Error> {
    decode_refresh_token(refresh_token)
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")))?;

    let revoked = user_repository::revoke_refresh_token(pool, refresh_token).await?;
    if !revoked {
        return Err(sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")));
    }

    Ok(())
}
use chrono::DateTime;
