use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Database model representing a shortened URL
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Url {
    pub id: i64,
    pub short_code: String,
    pub original_url: String,
    pub clicks: i64,
    pub created_at: DateTime<Utc>,
}

/// Request payload for creating a new short URL
#[derive(Debug, Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
}

impl CreateUrlRequest {
    /// Validate the URL format
    pub fn validate(&self) -> Result<(), &'static str> {
        let url = self.url.trim();
        
        if url.is_empty() {
            return Err("URL cannot be empty");
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("URL must start with http:// or https://");
        }

        if url.len() > 2048 {
            return Err("URL is too long (max 2048 characters)");
        }

        Ok(())
    }
}

/// Response payload after creating a short URL
#[derive(Debug, Serialize)]
pub struct CreateUrlResponse {
    pub id: i64,
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
}

/// Response for listing URLs
#[derive(Debug, Serialize)]
pub struct UrlListResponse {
    pub urls: Vec<Url>,
    pub total: usize,
}
