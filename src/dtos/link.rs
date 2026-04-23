use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateLinkRequest {
    pub original_url: String,
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LinkResponse {
    pub id: i64,
    pub short_code: String,
    pub original_url: String,
    pub title: Option<String>,
    pub click_count: i64,
    pub is_active: Option<bool>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DeleteLinkResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DailyAnalyticsResponse {
    pub date: String,
    pub total_clicks: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AdvancedSearchRequest {
   pub min_clicks: Option<i64>,
    pub max_clicks: Option<i64>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub domain: Option<String>,
}

