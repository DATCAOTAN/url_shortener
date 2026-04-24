use sqlx::{PgPool, Error, Postgres, QueryBuilder, Transaction};
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
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Link, Error> {
    sqlx::query_as::<_, Link>(
        r#"
        INSERT INTO links (id, owner_id, original_url, short_code, title, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(owner_id)
    .bind(original_url)
    .bind(short_code)
    .bind(title)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn find_by_short_code(pool: &PgPool, short_code: &str) -> Result<Option<Link>, Error> {
    sqlx::query_as::<_, Link>(
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at FROM links WHERE short_code = $1",
    )
    .bind(short_code)
    .fetch_optional(pool)
    .await
}

pub async fn find_active_by_short_code(pool: &PgPool, short_code: &str) -> Result<Option<Link>, Error> {
    sqlx::query_as::<_, Link>(
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at FROM links WHERE short_code = $1 AND (is_active IS NULL OR is_active = TRUE) AND (expires_at IS NULL OR expires_at > NOW())",
    )
    .bind(short_code)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_owner_and_original_url(
    pool: &PgPool,
    owner_id: i64,
    original_url: &str,
) -> Result<Option<Link>, Error> {
    sqlx::query_as::<_, Link>(
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at FROM links WHERE owner_id = $1 AND original_url = $2 AND (is_active IS NULL OR is_active = TRUE) AND (expires_at IS NULL OR expires_at > NOW())",
    )
    .bind(owner_id)
    .bind(original_url)
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

pub async fn get_all_by_user(
    pool: &PgPool,
    user_id: i64,
    page: i64,
    page_size: i64,
    sort_by: &str,
    sort_order: &str,
) -> Result<Vec<Link>, Error> {
    let offset = (page - 1) * page_size;
    let order_by = match sort_by {
        "click_count" => "COALESCE(click_count, 0)",
        "title" => "title",
        _ => "created_at",
    };
    let order_dir = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    let sql = format!(
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at FROM links WHERE owner_id = $1 ORDER BY {} {} LIMIT $2 OFFSET $3",
        order_by,
        order_dir
    );

    sqlx::query_as::<_, Link>(&sql)
        .bind(user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn get_all(
    pool: &PgPool,
    page: i64,
    page_size: i64,
    sort_by: &str,
    sort_order: &str,
) -> Result<Vec<Link>, Error> {
    let offset = (page - 1) * page_size;
    let order_by = match sort_by {
        "click_count" => "COALESCE(click_count, 0)",
        "title" => "title",
        _ => "created_at",
    };
    let order_dir = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    let sql = format!(
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at FROM links ORDER BY {} {} LIMIT $1 OFFSET $2",
        order_by,
        order_dir
    );

    sqlx::query_as::<_, Link>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn soft_delete_by_owner(
    pool: &PgPool,
    link_id: i64,
    owner_id: i64,
) -> Result<Option<Link>, Error> {
    sqlx::query_as::<_, Link>(
        "UPDATE links SET is_active = FALSE, updated_at = NOW() WHERE id = $1 AND owner_id = $2 RETURNING id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at",
    )
    .bind(link_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await
}

pub async fn soft_delete_by_id(pool: &PgPool, link_id: i64) -> Result<Option<Link>, Error> {
    sqlx::query_as::<_, Link>(
        "UPDATE links SET is_active = FALSE, updated_at = NOW() WHERE id = $1 RETURNING id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at",
    )
    .bind(link_id)
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
pub async fn advanced_search_links(
    pool: &PgPool,
    owner_id: i64,
    min_clicks: Option<i64>,
    max_clicks: Option<i64>,
    from_date: Option<NaiveDate>,
    to_date: Option<NaiveDate>,
    domain: Option<String>,
    is_active: Option<bool>,
) -> Result<Vec<Link>, Error> {
    let mut builder = QueryBuilder::<Postgres>::new(
        "SELECT id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at FROM links WHERE owner_id = ",
    );
    builder.push_bind(owner_id);

    match is_active {
        Some(true) => {
            builder.push(" AND (is_active IS NULL OR is_active = TRUE) AND (expires_at IS NULL OR expires_at > NOW())");
        }
        Some(false) => {
            builder.push(" AND (is_active = FALSE OR expires_at <= NOW())");
        }
        None => {
            builder.push(" AND (is_active IS NULL OR is_active = TRUE)");
        }
    }

    if let Some(min) = min_clicks {
        builder.push(" AND COALESCE(click_count, 0) >= ");
        builder.push_bind(min);
    }

    if let Some(max) = max_clicks {
        builder.push(" AND COALESCE(click_count, 0) <= ");
        builder.push_bind(max);
    }

    if let Some(from) = from_date {
        builder.push(" AND created_at::date >= ");
        builder.push_bind(from);
    }

    if let Some(to) = to_date {
        builder.push(" AND created_at::date <= ");
        builder.push_bind(to);
    }

    if let Some(domain_value) = domain {
        let pattern = format!("%{}%", domain_value);
        builder.push(" AND original_url ILIKE ");
        builder.push_bind(pattern);
    }

    builder.push(" ORDER BY created_at DESC");

    builder.build_query_as::<Link>().fetch_all(pool).await
}
