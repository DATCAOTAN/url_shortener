use sqlx::{PgPool, Error};
use chrono::{NaiveDate, Utc, FixedOffset};
use crate::models::link::Link;
use crate::models::link_analytics::DailyClickTotal;
use crate::repositories::link_repository;

const BASE62_CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn encode_base62(mut num: i64) -> String {
    if num == 0 {
        return "0".to_string();
    }

    let base = BASE62_CHARSET.len() as i64;
    let mut buf = Vec::new();

    while num > 0 {
        let idx = (num % base) as usize;
        buf.push(BASE62_CHARSET[idx] as char);
        num /= base;
    }

    buf.iter().rev().collect()
}

pub async fn create_short_link(
    pool: &PgPool,
    original_url: &str,
    owner_id: Option<i64>,
    title: Option<String>,
) -> Result<Link, Error> {
    if let Some(owner_id) = owner_id {
        if let Some(existing) = link_repository::find_by_owner_and_original_url(pool, owner_id, original_url).await? {
            return Ok(existing);
        }
    }

    let id = link_repository::next_link_id(pool).await?;
    let short_code = encode_base62(id);
    match link_repository::create_with_id(pool, id, owner_id, original_url, &short_code, title).await {
        Ok(link) => Ok(link),
        Err(e) if is_unique_violation(&e) => {
            if let Some(owner_id) = owner_id {
                if let Some(existing) = link_repository::find_by_owner_and_original_url(pool, owner_id, original_url).await? {
                    return Ok(existing);
                }
            }
            Err(e)
        }
        Err(e) => Err(e),
    }
}

pub async fn get_original_url(pool: &PgPool, short_code: &str) -> Result<Option<String>, Error> {
    if let Some(link) = link_repository::find_active_by_short_code(pool, short_code).await? {
        let today = current_date_vn();
        let pool = pool.clone();
        let link_id = link.id;
        tokio::spawn(async move {
            if let Err(e) = link_repository::increment_click_and_analytics(&pool, link_id, today).await {
                tracing::warn!("Async analytics update failed: {:?}", e);
            }
        });
        Ok(Some(link.original_url))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
pub async fn get_link_details(pool: &PgPool, short_code: &str) -> Result<Option<Link>, Error> {
    link_repository::find_by_short_code(pool, short_code).await
}

pub async fn get_user_links(pool: &PgPool, user_id: i64) -> Result<Vec<Link>, Error> {
    link_repository::get_all_by_user(pool, user_id).await
}

pub async fn get_all_links(pool: &PgPool) -> Result<Vec<Link>, Error> {
    link_repository::get_all(pool).await
}

pub async fn soft_delete_link(pool: &PgPool, user_id: i64, link_id: i64) -> Result<Option<Link>, Error> {
    link_repository::soft_delete_by_owner(pool, link_id, user_id).await
}

pub async fn admin_soft_delete_link(pool: &PgPool, link_id: i64) -> Result<Option<Link>, Error> {
    link_repository::soft_delete_by_id(pool, link_id).await
}

pub async fn advanced_search(
    pool: &PgPool,
    id_owner: i64,
    min_clicks: Option<i64>,
    max_clicks: Option<i64>,
    from_date: Option<NaiveDate>,
    to_date: Option<NaiveDate>,
    is_active: Option<bool>,
) -> Result<Vec<Link>, Error> {
    link_repository::advanced_search(pool,id_owner, min_clicks, max_clicks, from_date, to_date, is_active).await
}

pub async fn get_daily_analytics(
    pool: &PgPool,
    user_id: i64,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> Result<Vec<DailyClickTotal>, Error> {
    link_repository::get_daily_analytics_by_user(pool, user_id, from_date, to_date).await
}

fn current_date_vn() -> NaiveDate {
    let offset = FixedOffset::east_opt(7 * 3600).unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
    Utc::now().with_timezone(&offset).date_naive()
}

fn is_unique_violation(err: &Error) -> bool {
    match err {
        Error::Database(db_err) => db_err.code().as_deref() == Some("23505"),
        _ => false,
    }
}
