use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use std::env;

pub async fn init_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(50);
    let min_connections = env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(5);

    // 1. Cấu hình "đội quân" kết nối (Connection Pool)
    PgPoolOptions::new()
        // Tối đa 50 kết nối cùng lúc (Thoải mái cho đồ án)
        .max_connections(max_connections)
        
        // Luôn giữ ít nhất 5 kết nối "trực chiến" để khách vào là có ngay
        .min_connections(min_connections)
        
        // Nếu DB bận, chỉ đợi tối đa 5 giây rồi báo lỗi (không để khách đợi vô tận)
        .acquire_timeout(Duration::from_secs(5))
        
        // Sau 10 phút không dùng, hãy cho bớt "lính" về hưu để tiết kiệm RAM
        .idle_timeout(Duration::from_secs(600))
        
        // 2. Thực hiện kết nối
        .connect(database_url)
        .await
}