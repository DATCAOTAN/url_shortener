use sqlx::{PgPool, Error};
use chrono::{NaiveDate, Utc, FixedOffset};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::link::Link;
use crate::models::link_analytics::DailyClickTotal;
use crate::repositories::link_repository;

const BASE62_CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub enum RedirectResolution {
    Found(String),
    Expired,
    NotFound,
}

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
    expires_at: Option<u64>,
) -> Result<Link, Error> {
    if let Some(owner_id) = owner_id {
        if let Some(existing) = link_repository::find_by_owner_and_original_url(pool, owner_id, original_url).await? {
            return Ok(existing);
        }
    }

    let id = link_repository::next_link_id(pool).await?;
    let short_code = encode_base62(id);
    match link_repository::create_with_id(pool, id, owner_id, original_url, &short_code, title, expires_at).await {
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

pub async fn resolve_redirect_target(pool: &PgPool, short_code: &str) -> Result<RedirectResolution, Error> {
    if let Some(link) = link_repository::find_active_by_short_code(pool, short_code).await? {
        if let Some(expires_at) = link.expires_at {
            // Link het han khi expires_at nho hon timestamp hien tai (don vi giay).
            if expires_at < current_unix_timestamp() {
                return Ok(RedirectResolution::Expired);
            }
        }

        let today = current_date_vn();
        let pool = pool.clone();
        let link_id = link.id;
        tokio::spawn(async move {
            if let Err(e) = link_repository::increment_click_and_analytics(&pool, link_id, today).await {
                tracing::warn!("Async analytics update failed: {:?}", e);
            }
        });
        Ok(RedirectResolution::Found(link.original_url))
    } else {
        Ok(RedirectResolution::NotFound)
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

pub async fn get_links_with_min_clicks(pool: &PgPool, min_clicks: i64, is_active: bool) -> Result<Vec<Link>, Error> {
    link_repository::get_links_with_min_clicks(pool, min_clicks, is_active).await
}

pub async fn soft_delete_link(pool: &PgPool, user_id: i64, link_id: i64) -> Result<Option<Link>, Error> {
    link_repository::soft_delete_by_owner(pool, link_id, user_id).await
}

pub async fn admin_soft_delete_link(pool: &PgPool, link_id: i64) -> Result<Option<Link>, Error> {
    link_repository::soft_delete_by_id(pool, link_id).await
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

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn is_unique_violation(err: &Error) -> bool {
    match err {
        Error::Database(db_err) => db_err.code().as_deref() == Some("23505"),
        _ => false,
    }
}
