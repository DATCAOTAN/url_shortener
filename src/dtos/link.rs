use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateLinkRequest {
    pub original_url: String,
    pub title: Option<String>,
}

impl From<CreateLinkRequest> for crate::models::link::Link {
    fn from(req: CreateLinkRequest) -> Self {
        Self {
            id: 0, // This will be set by the database
            owner_id: None, // This can be set based on authentication context
            original_url: req.original_url,
            short_code: String::new(), // This will be generated in the service layer
            title: req.title,
            click_count: 0,
            is_active: true,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[derive(Serialize)]
pub struct LinkResponse {
    pub id: i64,
    pub short_code: String,
    pub original_url: String,
    pub title: Option<String>,
}

impl From<crate::models::link::Link> for LinkResponse {
    fn from(link: crate::models::link::Link) -> Self {
        Self {
            id: link.id,
            short_code: format!("http://localhost:8080/s/{}", link.short_code),
            original_url: link.original_url,
            title: link.title,
        }
    }
}