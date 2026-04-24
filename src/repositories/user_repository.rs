use sqlx::PgPool;
use crate::models::user::User;
use crate::models::refresh_tokens::RefreshToken;
use chrono::{DateTime, Utc};

pub async fn find_by_id(pool: &PgPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id=$1",user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn register(
    pool: &PgPool, 
    username: &str, 
    email: &str, 
    password_hash: &str
) -> Result<User, sqlx::Error> {
    // Dùng query_as! với danh sách cột tường minh
    // Lưu ý: Các đối số truyền vào macro nằm ngay sau câu SQL
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        username,
        email,
        password_hash
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as!(
        User,
        "SELECT * FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

pub async fn soft_delete_by_id(pool: &PgPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        "UPDATE users SET is_active = FALSE, updated_at = NOW() WHERE id = $1 RETURNING *",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn hard_delete_by_id(pool: &PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM users WHERE id = $1",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn save_refresh_token(
    pool: &PgPool,
    user_id: i64,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken, sqlx::Error> {
    let refresh_token = sqlx::query_as!(
        RefreshToken,
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3) RETURNING *",
        user_id,
        token_hash,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(refresh_token)
}   


pub async fn is_refresh_token_active(pool: &PgPool, token_hash: &str) -> Result<bool, sqlx::Error> {
   let token = sqlx::query_as!(
        RefreshToken,
        "SELECT * FROM refresh_tokens WHERE token_hash = $1",
        token_hash
    )
    .fetch_optional(pool)
    .await?;

    Ok(token.map_or(false, |t| t.revoked_at.is_none() && t.expires_at > Utc::now()))
}

pub async fn revoke_refresh_token(pool: &PgPool, token_hash: &str) -> Result<bool, sqlx::Error> {
   let result = sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = $1 WHERE token_hash = $2 AND revoked_at IS NULL",
        Utc::now(),
        token_hash
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn revoke_all_refresh_tokens_by_user_id(pool: &PgPool, user_id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = $1 WHERE user_id = $2 AND revoked_at IS NULL",
        Utc::now(),
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
