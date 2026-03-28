use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use crate::repositories::user_repository;
use crate::models::user::User;
use crate::dtos::user::{LoginResponse, RefreshTokenResponse};
use crate::utils::jwt::{encode_access_token, encode_refresh_token, decode_refresh_token};

fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn get_user(pool: &PgPool, id: i64) -> Result<User, sqlx::Error> {
    match user_repository::find_by_id(pool, id).await? {
        Some(user) => Ok(user),
        None => Err(sqlx::Error::RowNotFound),
    }
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    user_repository::get_all(pool).await
}

pub async fn admin_soft_delete_user(pool: &PgPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    let user = user_repository::soft_delete_by_id(pool, user_id).await?;
    if user.is_some() {
        let _ = user_repository::revoke_all_refresh_tokens_by_user_id(pool, user_id).await?;
    }
    Ok(user)
}

pub async fn admin_hard_delete_user(pool: &PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    user_repository::hard_delete_by_id(pool, user_id).await
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

    if !user.is_active {
        return Err(sqlx::Error::Io(std::io::Error::other("USER_DISABLED")));
    }

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
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    user_repository::save_refresh_token(pool, user.id, &refresh_token_hash, refresh_exp).await?;

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

    let refresh_token_hash = hash_refresh_token(refresh_token);

    let is_active = user_repository::is_refresh_token_active(pool, &refresh_token_hash).await?;
    if !is_active {
        return Err(sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")));
    }

    let user_id = token_data
        .sub
        .parse::<i64>()
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")))?;

    let user = user_repository::find_by_id(pool, user_id).await?;
    match user {
        Some(user) if user.is_active => {}
        Some(_) => return Err(sqlx::Error::Io(std::io::Error::other("USER_DISABLED"))),
        None => return Err(sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID"))),
    }

    let access_token = encode_access_token(token_data.sub, token_data.role)
        .map_err(|e| sqlx::Error::Io(std::io::Error::other(format!("Access JWT encode error: {e}"))))?;

    Ok(RefreshTokenResponse { access_token })
}

pub async fn logout_user(pool: &PgPool, refresh_token: &str) -> Result<(), sqlx::Error> {
    decode_refresh_token(refresh_token)
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")))?;

    let refresh_token_hash = hash_refresh_token(refresh_token);

    let revoked = user_repository::revoke_refresh_token(pool, &refresh_token_hash).await?;
    if !revoked {
        return Err(sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")));
    }

    Ok(())
}
