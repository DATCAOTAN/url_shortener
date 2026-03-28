use sqlx::{PgPool, Error, Postgres, Transaction};
use chrono::NaiveDate;
use crate::models::link::Link;
use crate::models::link_analytics::DailyClickTotal;

pub async fn next_link_id(pool: &PgPool) -> Result<i64, Error> {
    let id = sqlx::query_scalar!("SELECT nextval('links_id_seq') AS \"id!\"")
        .fetch_one(pool)
        .await?;
    Ok(id)
}

pub async fn create_with_id(
    pool: &PgPool,
    id: i64,
    owner_id: Option<i64>,
    original_url: &str,
    short_code: &str,
    title: Option<String>,
) -> Result<Link, Error> {
    sqlx::query_as!(
        Link,
        r#"
        INSERT INTO links (id, owner_id, original_url, short_code, title)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, owner_id, original_url, short_code, title, click_count, is_active, created_at, updated_at
        "#,
        id,
        owner_id,
        original_url,
        short_code,
        title
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_short_code(pool: &PgPool, short_code: &str) -> Result<Option<Link>, Error> {
    sqlx::query_as!(
        Link,
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, created_at, updated_at FROM links WHERE short_code = $1",
        short_code
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_active_by_short_code(pool: &PgPool, short_code: &str) -> Result<Option<Link>, Error> {
    sqlx::query_as!(
        Link,
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, created_at, updated_at FROM links WHERE short_code = $1 AND (is_active IS NULL OR is_active = TRUE)",
        short_code
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_owner_and_original_url(
    pool: &PgPool,
    owner_id: i64,
    original_url: &str,
) -> Result<Option<Link>, Error> {
    sqlx::query_as!(
        Link,
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, created_at, updated_at FROM links WHERE owner_id = $1 AND original_url = $2 AND (is_active IS NULL OR is_active = TRUE)",
        owner_id,
        original_url
    )
    .fetch_optional(pool)
    .await
}

pub async fn increment_click_and_analytics(
    pool: &PgPool,
    link_id: i64,
    date: NaiveDate,
) -> Result<(), Error> {
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    sqlx::query!(
        "UPDATE links SET click_count = click_count + 1 WHERE id = $1",
        link_id
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        "INSERT INTO link_analytics (link_id, date, clicks) VALUES ($1, $2, 1) ON CONFLICT (link_id, date) DO UPDATE SET clicks = link_analytics.clicks + 1",
        link_id,
        date
    )
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_all_by_user(pool: &PgPool, user_id: i64) -> Result<Vec<Link>, Error> {
    sqlx::query_as!(
        Link,
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, created_at, updated_at FROM links WHERE owner_id = $1 ORDER BY created_at DESC",
        user_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<Link>, Error> {
    sqlx::query_as!(
        Link,
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, created_at, updated_at FROM links ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn soft_delete_by_owner(
    pool: &PgPool,
    link_id: i64,
    owner_id: i64,
) -> Result<Option<Link>, Error> {
    sqlx::query_as!(
        Link,
        "UPDATE links SET is_active = FALSE, updated_at = NOW() WHERE id = $1 AND owner_id = $2 RETURNING id, owner_id, original_url, short_code, title, click_count, is_active, created_at, updated_at",
        link_id,
        owner_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn soft_delete_by_id(pool: &PgPool, link_id: i64) -> Result<Option<Link>, Error> {
    sqlx::query_as!(
        Link,
        "UPDATE links SET is_active = FALSE, updated_at = NOW() WHERE id = $1 RETURNING id, owner_id, original_url, short_code, title, click_count, is_active, created_at, updated_at",
        link_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_daily_analytics_by_user(
    pool: &PgPool,
    owner_id: i64,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> Result<Vec<DailyClickTotal>, Error> {
    sqlx::query_as!(
        DailyClickTotal,
        "SELECT la.date, COALESCE(SUM(la.clicks), 0) AS \"total_clicks!\" FROM link_analytics la JOIN links l ON l.id = la.link_id WHERE l.owner_id = $1 AND la.date BETWEEN $2 AND $3 GROUP BY la.date ORDER BY la.date",
        owner_id,
        from_date,
        to_date
    )
    .fetch_all(pool)
    .await
}
