use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDate;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LinkAnalytics {
	pub id: i64,
	pub link_id: i64,
	pub date: NaiveDate,
	pub clicks: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DailyClickTotal {
	pub date: NaiveDate,
	pub total_clicks: i64,
}