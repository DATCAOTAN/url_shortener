use deadpool_redis::redis::AsyncCommands;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),

    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
}

const URL_CACHE_PREFIX: &str = "url:";
const CACHE_TTL_SECONDS: u64 = 3600;
const LINK_COOLDOWN_PREFIX: &str = "cooldown:create_link:user:";


pub async fn get_cached_url(
    redis: &deadpool_redis::Pool,
    short_code: &str,
) -> Result<Option<String>, CacheError> {
    let mut conn = redis.get().await?;
    let cache_key = format!("{}{}", URL_CACHE_PREFIX, short_code);
    let result: Option<String> = conn.get(&cache_key).await?;
    Ok(result)
}

pub async fn set_cached_url(
    redis: &deadpool_redis::Pool,
    short_code: &str,
    original_url: &str,
) -> Result<(), CacheError> {
    let mut conn = redis.get().await?;
    let cache_key = format!("{}{}", URL_CACHE_PREFIX, short_code);
    conn.set_ex::<_, _, ()>(&cache_key, original_url, CACHE_TTL_SECONDS).await?;
    Ok(())
}

pub async fn invalidate_cache(
    redis: &deadpool_redis::Pool,
    short_code: &str,
) -> Result<(), CacheError> {
    let mut conn = redis.get().await?;
    let cache_key = format!("{}{}", URL_CACHE_PREFIX, short_code);
    conn.del::<_, ()>(&cache_key).await?;
    Ok(())
}

pub async fn try_acquire_link_cooldown(
    redis: &deadpool_redis::Pool,
    user_id: i64,
    cooldown_seconds: u64,
) -> Result<bool, CacheError> {
    let mut conn = redis.get().await?;
    let key = format!("{}{}", LINK_COOLDOWN_PREFIX, user_id);
    let result: Option<String> = deadpool_redis::redis::cmd("SET")
        .arg(&key)
        .arg("1")
        .arg("EX")
        .arg(cooldown_seconds)
        .arg("NX")
        .query_async(&mut conn)
        .await?;

    Ok(result.is_some())
}