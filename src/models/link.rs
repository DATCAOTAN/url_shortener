use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub original_url: String,
    pub short_code: String,
    pub title: Option<String>,
    pub click_count: i64,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}