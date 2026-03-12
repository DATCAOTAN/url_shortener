// use serde::{Deserialize, Serialize};
// use sqlx::FromRow;

// #[derive(Debug, Serialize, Deserialize, FromRow)]
// pub struct Link {
//     pub id: i32,
//     pub owner_id: Option<i32>,
//     pub original_url: String,
//     pub short_code: String,
//     pub title: Option<String>,
//     pub click_count: i32,
//     pub is_active: bool,
// }