# Các Bài Tập Cơ Bản - Kiểm Tra Kỹ Thuật (URL Shortener)

Dưới đây là 3 bài tập ở mức độ cơ bản (Basic), tập trung vào việc giúp bạn làm quen và kiểm tra các thao tác quen thuộc nhất với Framework Axum, SQLx và Redis.

---

## Bài Tập 1: Thêm trường `description` (Ghi chú) cho Link

**Bài toán:** Người dùng muốn lưu thêm một đoạn ghi chú nhỏ (không bắt buộc) cho mỗi đường link họ tạo để dễ nhớ.
**Yêu cầu:** 
- Thêm trường `description` vào payload tạo link.
- Lưu xuống CSDL và trả về trường này trong danh sách link.

### Giải pháp Code

**1. Sửa CSDL (Database):**
Bạn cần chạy câu lệnh SQL để cập nhật bảng:
```sql
ALTER TABLE links ADD COLUMN description TEXT;
```

**2. Cập nhật Model và DTO (`src/models/link.rs` và `src/dtos/link.rs`):**
```rust
// Trong src/models/link.rs
pub struct Link {
    // ... các trường cũ
    pub description: Option<String>,
}

// Trong src/dtos/link.rs
pub struct CreateLinkRequest {
    pub original_url: String,
    // ...
    pub description: Option<String>, // Thêm trường này
}

pub struct LinkResponse {
    // ...
    pub description: Option<String>, // Thêm trường này
}
```

**3. Cập nhật Repository (`src/repositories/link_repository.rs`):**
Tìm đến hàm `create_with_id` và thêm tham số `description`:
```rust
pub async fn create_with_id(
    // ... tham số cũ
    description: Option<String>,
) -> Result<Link, Error> {
    sqlx::query_as::<_, Link>(
        r#"
        INSERT INTO links (id, owner_id, original_url, short_code, title, expires_at, description)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at, description
        "#,
    )
    // ... các hàm bind cũ
    .bind(description) // Gán thêm giá trị $7
    .fetch_one(pool)
    .await
}
```
*(Bạn cũng cần thêm cột `description` vào các câu `SELECT` ở các hàm repository khác).*

**4. Cập nhật Handler (`src/handlers/link_handler.rs`):**
```rust
// Cập nhật hàm create_link để truyền payload.description xuống DB
let link = link_service::create_short_link(
    &state.db,
    &payload.original_url,
    // ...
    payload.description, // Truyền thêm
).await?;

// Map ra LinkResponse
LinkResponse {
    // ...
    description: link.description,
}
```

### Giải thích End-to-End:
Bài tập này kiểm tra kỹ năng cơ bản nhất: **Thêm một cột dữ liệu**.
- Luồng đi từ ngoài vào trong: User gửi JSON chứa `description`. Axum parse JSON đó vào struct `CreateLinkRequest` (DTO).
- Handler trích xuất dữ liệu, đẩy xuống Service, Service đẩy xuống Repository.
- Repository sử dụng SQLx QueryBuilder với lệnh `INSERT` và `RETURNING` để chèn vào PostgreSQL và lấy ra kết quả ngay lập tức. Sau đó, Handler lại map nó ra `LinkResponse` để hiển thị lại cho Client.

---

## Bài Tập 2: API Xóa Redis Cache Thủ Công (Dành cho Admin)

**Bài toán:** Khi hệ thống bị lỗi hoặc URL đích thay đổi, Admin cần một nút bấm API để ép hệ thống xóa ngay lập tức bộ nhớ tạm trong Redis của một short link.
**Yêu cầu:** Viết một endpoint `DELETE /admin/cache/{short_code}` chỉ dành cho Admin.

### Giải pháp Code

**1. Viết Handler mới (`src/handlers/admin_handler.rs`):**
```rust
use crate::services::cache_service;
use axum::response::IntoResponse;

#[utoipa::path(
    delete,
    path = "/admin/cache/{short_code}",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("short_code" = String, Path, description = "Mã ngắn cần xóa cache")),
    responses((status = 200, description = "Xóa cache thành công"))
)]
pub async fn clear_link_cache(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>, // Token đã được middleware check role Admin
    Path(short_code): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    
    // Gọi thẳng vào cache_service đã có sẵn trong dự án
    match cache_service::invalidate_cache(&state.redis, &short_code).await {
        Ok(_) => Ok(Json(serde_json::json!({"message": "Đã xóa cache thành công"}))),
        Err(e) => {
            tracing::error!("Lỗi xóa cache: {:?}", e);
            Err(AppError::Internal("Không thể xóa cache".to_string()))
        }
    }
}
```

**2. Đăng ký Router (`src/routes/admin_routes.rs`):**
```rust
use crate::handlers::admin_handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        // ... các route cũ
        .route("/cache/:short_code", delete(admin_handler::clear_link_cache))
}
```

### Giải thích End-to-End:
Bài tập này kiểm tra **cách tạo một API endpoint hoàn toàn mới trong Axum** và **thao tác với Redis**.
- Middleware Auth (bọc ngoài `admin_routes`) đã xử lý việc kiểm tra xem JWT Token có phải là của quyền Admin hay không. Do đó, trong Handler ta chỉ cần dùng biến `Extension(_claims)` để chặn bắt (nếu chạy tới code này nghĩa là token hợp lệ).
- Axum dùng `Path(short_code)` để lấy biến động từ URL `/admin/cache/xyz`.
- Handler gọi thẳng `invalidate_cache` (bản chất là gọi lệnh `DEL url:xyz` vào Redis) và trả về HTTP 200.

---

## Bài Tập 3: API Lấy "Tổng Số Link Đang Hoạt Động" Của User

**Bài toán:** Giao diện người dùng cần hiển thị một ô thống kê: "Bạn đang có tổng cộng N links đang hoạt động".
**Yêu cầu:** Viết API `GET /users/me/stats` trả về tổng số link mà user hiện đang sở hữu (điều kiện: `is_active = true` và `expires_at` chưa hết hạn).

### Giải pháp Code

**1. Tạo DTO (`src/dtos/user.rs`):**
```rust
#[derive(Serialize, ToSchema)]
pub struct UserStatsResponse {
    pub active_links_count: i64,
}
```

**2. Thêm hàm vào Repository (`src/repositories/link_repository.rs`):**
```rust
pub async fn count_active_links_by_user(pool: &PgPool, owner_id: i64) -> Result<i64, Error> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM links 
        WHERE owner_id = $1 
          AND (is_active IS NULL OR is_active = TRUE) 
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        owner_id
    )
    .fetch_one(pool)
    .await?;

    // count trả về là Option<i64>, ta unwrap an toàn về 0 nếu không có gì
    Ok(count.unwrap_or(0))
}
```

**3. Viết Handler (`src/handlers/user_handler.rs`):**
```rust
#[utoipa::path(
    get,
    path = "/users/me/stats",
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn get_my_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<UserStatsResponse>> {
    let user_id = claims.sub.parse::<i64>().unwrap();

    let count = link_repository::count_active_links_by_user(&state.db, user_id)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(UserStatsResponse {
        active_links_count: count,
    }))
}
```

### Giải thích End-to-End:
Bài tập này kiểm tra kỹ năng **viết câu lệnh thống kê SQL (Aggregate Queries) cơ bản** và map vào logic dự án.
- Request đi từ Client vào Router `/users/me/stats`. Extractor lấy User ID từ JWT Claims.
- Service/Handler gọi hàm Repository.
- Repository sử dụng `sqlx::query_scalar!` vì chúng ta chỉ lấy đúng 1 cột vô hướng duy nhất là số đếm (`COUNT(*)`). SQL query lọc đúng điều kiện logic: link của chủ sở hữu đó, cờ active=true và thời gian sống > hiện tại.
- Database trả về 1 số nguyên (`i64`), Handler bọc nó vào `UserStatsResponse` dạng JSON rồi trả về.
