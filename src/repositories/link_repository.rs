use sqlx::PgPool;
use crate::models::link::Link;


pub async fn get_next_id(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let next_id = sqlx::query_scalar::<_, i64>("SELECT nextval('links_id_seq')")
        .fetch_one(pool)
        .await?;
    Ok(next_id)
}

pub async fn insert_url(pool: &PgPool,id: i64, owner_id: Option<i64>, original_url: &str, short_code: &str) -> Result<Link, sqlx::Error> {
    let link = sqlx::query_as::<_, Link>(
        "INSERT INTO links (id, owner_id, original_url, short_code) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(id)
    .bind(owner_id)
    .bind(original_url)
    .bind(short_code)
    .fetch_one(pool)
    .await?;

    Ok(link)
}

pub async fn get_orginal_url(pool: &PgPool, short_code: &str) -> Result<Option<String>, sqlx::Error> {
    let original_url = sqlx::query_scalar::<_, String>(
            "SELECT original_url FROM links WHERE short_code = $1 AND is_active = true"
        )
        .bind(short_code)
        .fetch_optional(pool)
        .await?;

    Ok(original_url)
}

pub async fn increment_click_count(pool: &PgPool, short_code: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE links SET click_count = click_count + 1 WHERE short_code = $1")
        .bind(short_code)
        .execute(pool)
        .await?;
    Ok(())
}