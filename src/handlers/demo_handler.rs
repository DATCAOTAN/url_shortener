use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
pub struct SpawnDemoQuery {
    pub delay_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SpawnDemoResponse {
    pub message: String,
    pub delay_ms: u64,
    pub handler_elapsed_ms: u128,
}

pub async fn spawn_demo(Query(query): Query<SpawnDemoQuery>) -> Json<SpawnDemoResponse> {
    let started_at = Instant::now();
    let delay_ms = query.delay_ms.unwrap_or(5000);

    println!("[Main Thread] Nhận request /demo/spawn với delay_ms={}", delay_ms);

    tokio::spawn(async move {
        println!("[Background Thread] Bắt đầu tác vụ nền, sleep {}ms", delay_ms);
        tracing::info!("[spawn-demo-endpoint] background task started; delay={}ms", delay_ms);
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        println!("[Background Thread] Hoàn tất tác vụ nền sau {}ms", delay_ms);
        tracing::info!("[spawn-demo-endpoint] background task finished after {}ms", delay_ms);
    });

    println!("[Main Thread] Đã trả về HTTP response ngay lập tức cho người dùng");

    Json(SpawnDemoResponse {
        message: "Request returned immediately; check server logs for background completion".to_string(),
        delay_ms,
        handler_elapsed_ms: started_at.elapsed().as_millis(),
    })
}
