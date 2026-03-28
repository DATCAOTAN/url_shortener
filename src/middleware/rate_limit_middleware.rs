use std::{
    collections::HashMap,
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::State,
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};
use tokio::sync::Mutex;

use crate::error::AppError;

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, WindowCounter>>>,
    limit_per_minute: u32,
}

#[derive(Clone, Copy)]
struct WindowCounter {
    count: u32,
    window_start: Instant,
}

impl RateLimiter {
    pub fn from_env() -> Self {
        let limit_per_minute = env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(120);

        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            limit_per_minute,
        }
    }

    async fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut guard = self.inner.lock().await;

        let entry = guard.entry(key.to_string()).or_insert(WindowCounter {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start) >= Duration::from_secs(60) {
            entry.count = 0;
            entry.window_start = now;
        }

        if entry.count >= self.limit_per_minute {
            return false;
        }

        entry.count += 1;
        true
    }
}

pub async fn rate_limit_middleware(
    State(rate_limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let client_key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().to_string())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|v| v.to_string())
        })
        .or_else(|| {
            req.headers()
                .get(header::USER_AGENT)
                .and_then(|h| h.to_str().ok())
                .map(|ua| format!("ua:{ua}"))
        })
        .unwrap_or_else(|| "anonymous".to_string());

    if !rate_limiter.allow(&client_key).await {
        return Err(AppError::TooManyRequests(
            "Rate limit exceeded. Please retry in one minute".to_string(),
        ));
    }

    Ok(next.run(req).await)
}
