use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub original_url: String,
    pub short_code: String,
    pub title: Option<String>,
    pub click_count: Option<i64>,
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Link {
    pub fn is_active_now(&self) -> bool {
        let active = self.is_active.unwrap_or(true);
        if !active {
            return false;
        }
        if let Some(expires_at) = self.expires_at {
            if expires_at <= Utc::now() {
                return false;
            }
        }
        true
    }
}
