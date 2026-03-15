use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sqlx::PgPool;
use crate::repositories::user_repository;
use crate::models::user::User;
use crate::dtos::user::{LoginResponse, RefreshTokenResponse};
use crate::dtos::claims::Claims;

pub async fn get_user(pool: &PgPool, id: i64) -> Result<User, sqlx::Error> {
    match user_repository::find_by_id(pool, id).await? {
        Some(user) => Ok(user),
        None => Err(sqlx::Error::RowNotFound),
    }
}

//service nhận password, hash rồi gọi repo
pub async fn register_user(pool: &PgPool,username: &str,email: &str,password: &str,) -> Result<User, sqlx::Error> {
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

    let access_secret = std::env::var("JWT_SECRET")
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("Missing JWT_SECRET")))?;
    let refresh_secret = std::env::var("JWT_REFRESH_SECRET")
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("Missing JWT_REFRESH_SECRET")))?;

    let access_exp = Utc::now() + std::env::var("ACCESS_TOKEN_EXPIRE")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .map(Duration::seconds)
        .unwrap_or_else(|| Duration::seconds(900));
    let access_claims = Claims {
        id: user.id,
        sub: user.email.clone(),
        exp: access_exp.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(access_secret.as_bytes())
    )
    .map_err(|e| sqlx::Error::Io(std::io::Error::other(format!("Access JWT encode error: {e}"))))?;

    let refresh_exp = Utc::now() + std::env::var("REFRESH_TOKEN_EXPIRE")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .map(Duration::seconds)
        .unwrap_or_else(|| Duration::days(30));
    let refresh_claims = Claims {
        id: user.id,
        sub: user.email.clone(),
        exp: refresh_exp.timestamp() as usize,
    };

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(refresh_secret.as_bytes())
    )
    .map_err(|e| sqlx::Error::Io(std::io::Error::other(format!("Refresh JWT encode error: {e}"))))?;

    // TODO: Hash refresh_token before persisting in production.
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
    let refresh_secret = std::env::var("JWT_REFRESH_SECRET")
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("Missing JWT_REFRESH_SECRET")))?;

    let token_data = decode::<Claims>(
        refresh_token,
        &DecodingKey::from_secret(refresh_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")))?;

    let is_active = user_repository::is_refresh_token_active(pool, refresh_token).await?;
    if !is_active {
        return Err(sqlx::Error::Io(std::io::Error::other("REFRESH_TOKEN_INVALID")));
    }

    let access_secret = std::env::var("JWT_SECRET")
        .map_err(|_| sqlx::Error::Io(std::io::Error::other("Missing JWT_SECRET")))?;
    let access_exp = Utc::now() + std::env::var("ACCESS_TOKEN_EXPIRE")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .map(Duration::seconds)
        .unwrap_or_else(|| Duration::seconds(900));

    let access_claims = Claims {
        id: token_data.claims.id,
        sub: token_data.claims.sub,
        exp: access_exp.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(access_secret.as_bytes()),
    )
    .map_err(|e| sqlx::Error::Io(std::io::Error::other(format!("Access JWT encode error: {e}"))))?;

    Ok(RefreshTokenResponse { access_token })
}