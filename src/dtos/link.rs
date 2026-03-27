use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateLinkRequest {
    pub original_url: String,
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LinkResponse {
    pub id: i64,
    pub short_code: String,
    pub original_url: String,
    pub title: Option<String>,
    pub click_count: i64,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteLinkResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct DailyAnalyticsResponse {
    pub date: String,
    pub total_clicks: i64,
}
