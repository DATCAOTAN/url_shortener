use sqlx::PgPool;
use crate::models::user::User;
use chrono::{DateTime, Utc};

pub async fn find_by_id(pool: &PgPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn register(pool: &PgPool, username: &str, email: &str, password_hash: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn save_refresh_token(
    pool: &PgPool,
    user_id: i64,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn is_refresh_token_active(pool: &PgPool, token_hash: &str) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM refresh_tokens WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW())",
    )
    .bind(token_hash)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}