use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateLinkRequest {
    pub original_url: String,
    pub title: Option<String>,
    pub expires_in_seconds: Option<u64>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LinkResponse {
    pub id: i64,
    pub short_code: String,
    pub original_url: String,
    pub title: Option<String>,
    pub click_count: i64,
    pub is_active: bool,
    pub expires_at: Option<u64>,
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
pub struct SearchQuery{
    pub min_clicks: i64,
    pub is_active: bool
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginationQuery{
    pub current_page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginationMetadata {
    pub limit: u32,
    pub offset: u64,
    pub sort_by: String,
    pub total_items: i64,
    pub total_pages: u32,
    pub current_page: u32,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginationResponse {
    pub data: Vec<LinkResponse>,
    pub metadata: PaginationMetadata,
}