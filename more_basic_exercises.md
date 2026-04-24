# 7 Bài Tập Cơ Bản (Bổ Sung) - Kiểm Tra Kỹ Thuật

Dưới đây là 7 bài tập bổ sung, tiếp tục rèn luyện các thao tác cơ bản nhất của dự án từ thao tác DB (UPDATE, ILIKE, ORDER BY), thao tác HTTP (PATCH) cho đến kết nối Redis.

---

## Bài Tập 4: Tìm kiếm link theo chuỗi con của Tiêu đề (Search by Title)
**Bài toán:** Cung cấp cho user công cụ tìm kiếm nhanh các link của họ bằng cách gõ một vài chữ trong tiêu đề.
**Yêu cầu:** Viết API `GET /links/search?q=text` trả về danh sách link của user có chứa chuỗi `text` trong tiêu đề.

**Giải pháp Code:**
```rust
// 1. DTO
#[derive(Deserialize)]
pub struct SearchQuery { pub q: String }

// 2. Repository
pub async fn search_by_title(pool: &PgPool, owner_id: i64, query: &str) -> Result<Vec<Link>, Error> {
    let pattern = format!("%{}%", query);
    sqlx::query_as!(
        Link,
        "SELECT * FROM links WHERE owner_id = $1 AND title ILIKE $2",
        owner_id, pattern
    ).fetch_all(pool).await
}

// 3. Handler
pub async fn search_links(State(state): State<AppState>, Extension(claims): Extension<Claims>, Query(params): Query<SearchQuery>) -> AppResult<Json<Vec<LinkResponse>>> {
    let user_id = claims.sub.parse::<i64>().unwrap();
    let links = link_repository::search_by_title(&state.db, user_id, &params.q).await?;
    // Map links sang LinkResponse và trả về...
}
```
**Giải thích E2E:** Nhận chuỗi truy vấn `q` từ Query string. Nối chuỗi thêm `%` ở hai đầu để dùng với toán tử `ILIKE` trong PostgreSQL (tìm kiếm chuỗi con không phân biệt hoa thường).

---

## Bài Tập 5: Đổi Tiêu đề của Link (Edit Link)
**Bài toán:** User lỡ viết sai tiêu đề lúc tạo link và muốn đổi lại.
**Yêu cầu:** Viết API `PATCH /links/{id}/title` nhận JSON `{"title": "Tên mới"}`.

**Giải pháp Code:**
```rust
// 1. DTO
#[derive(Deserialize)]
pub struct UpdateTitleRequest { pub title: String }

// 2. Repository
pub async fn update_title(pool: &PgPool, link_id: i64, owner_id: i64, new_title: &str) -> Result<u64, Error> {
    let result = sqlx::query!(
        "UPDATE links SET title = $1, updated_at = NOW() WHERE id = $2 AND owner_id = $3",
        new_title, link_id, owner_id
    ).execute(pool).await?;
    Ok(result.rows_affected())
}

// 3. Handler
pub async fn edit_title(State(state): State<AppState>, Extension(claims): Extension<Claims>, Path(id): Path<i64>, Json(payload): Json<UpdateTitleRequest>) -> AppResult<Json<serde_json::Value>> {
    let user_id = claims.sub.parse::<i64>().unwrap();
    let rows = link_repository::update_title(&state.db, id, user_id, &payload.title).await?;
    if rows == 0 { return Err(AppError::NotFound("Link không tồn tại".into())); }
    Ok(Json(json!({"message": "Cập nhật thành công"})))
}
```
**Giải thích E2E:** Phương thức PATCH dùng để cập nhật một phần của bản ghi. Câu lệnh SQL `UPDATE` cần truyền thêm `owner_id` để đảm bảo user không được quyền đổi tiêu đề link của người khác. Hàm `rows_affected()` cho biết có bao nhiêu dòng bị ảnh hưởng (nếu là 0 tức là link không tồn tại hoặc không thuộc về user).

---

## Bài Tập 6: "Dọn rác" các Link đã hết hạn (Cleanup Job)
**Bài toán:** Database ngày càng phình to chứa toàn link quá hạn. Admin cần một nút bấm "Xóa diện rộng".
**Yêu cầu:** Viết API `DELETE /admin/links/expired` để cập nhật `is_active = FALSE` cho toàn bộ link có `expires_at <= NOW()`.

**Giải pháp Code:**
```rust
// 1. Repository
pub async fn disable_expired_links(pool: &PgPool) -> Result<u64, Error> {
    let result = sqlx::query!("UPDATE links SET is_active = FALSE WHERE expires_at <= NOW() AND is_active = TRUE")
        .execute(pool).await?;
    Ok(result.rows_affected())
}

// 2. Handler
pub async fn clean_expired(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let rows = link_repository::disable_expired_links(&state.db).await?;
    Ok(Json(json!({"message": format!("Đã khóa {} link hết hạn", rows)})))
}
```
**Giải thích E2E:** Thay vì SELECT từng dòng lên rồi dùng vòng lặp FOR để khóa rất chậm, ta dùng đúng 1 câu SQL `UPDATE ... WHERE ...` để PostgreSQL quét và xử lý hàng loạt siêu tốc. API trả về số lượng dòng đã dọn dẹp.

---

## Bài Tập 7: Lấy Top 5 Links "Viral" nhất hệ thống
**Bài toán:** Hiện trang chủ cho phép user xem top 5 đường link có nhiều lượt truy cập nhất hệ thống.
**Yêu cầu:** Viết API Public (không cần Token) `GET /links/top`.

**Giải pháp Code:**
```rust
// 1. Repository
pub async fn get_top_links(pool: &PgPool) -> Result<Vec<Link>, Error> {
    sqlx::query_as!(
        Link,
        "SELECT * FROM links WHERE is_active = TRUE ORDER BY COALESCE(click_count, 0) DESC LIMIT 5"
    ).fetch_all(pool).await
}

// 2. Handler
pub async fn top_links(State(state): State<AppState>) -> AppResult<Json<Vec<LinkResponse>>> {
    let links = link_repository::get_top_links(&state.db).await?;
    // Map to JSON Response...
}
```
**Giải thích E2E:** SQL sử dụng mệnh đề `ORDER BY ... DESC LIMIT 5`. Lưu ý hàm `COALESCE(click_count, 0)` để xử lý trường hợp giá trị đang bị `NULL` trong DB vẫn có thể đem ra so sánh số học an toàn.

---

## Bài Tập 8: API Kiểm tra tình trạng Redis (Ping)
**Bài toán:** Đôi khi Redis bị chết mà ta không biết, cần có một API để DevOps gọi kiểm tra xem Redis có đang sống không.
**Yêu cầu:** Viết API `GET /health/redis`.

**Giải pháp Code:**
```rust
// Handler
pub async fn ping_redis(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let mut conn = state.redis.get().await.map_err(|_| AppError::Internal("Redis down".into()))?;
    
    // Gọi lệnh PING tới Redis
    let result: Option<String> = deadpool_redis::redis::cmd("PING").query_async(&mut conn).await.ok();
    
    match result {
        Some(v) if v == "PONG" => Ok(Json(json!({"status": "healthy"}))),
        _ => Err(AppError::Internal("Redis no response".into()))
    }
}
```
**Giải thích E2E:** Trực tiếp mượn connection từ Redis Pool (`state.redis.get()`) và đẩy raw command `PING`. Theo tài liệu của Redis, nếu nó còn sống nó sẽ trả về chuỗi `PONG`. 

---

## Bài Tập 9: Lấy ngẫu nhiên 1 Link (Random Link)
**Bài toán:** Nút "I'm Feeling Lucky" - Đưa user tới 1 đường link hoàn toàn ngẫu nhiên trong CSDL.
**Yêu cầu:** Viết API `GET /links/random` trả về URL gốc của 1 link bất kỳ.

**Giải pháp Code:**
```rust
// 1. Repository
pub async fn get_random_link(pool: &PgPool) -> Result<Option<Link>, Error> {
    sqlx::query_as!(
        Link,
        "SELECT * FROM links WHERE is_active = TRUE ORDER BY RANDOM() LIMIT 1"
    ).fetch_optional(pool).await
}
// 2. Handler (Map sang JSON hoặc gọi thẳng Redirect)
```
**Giải thích E2E:** Đây là 1 trick rất nổi tiếng trong SQL. Cụm `ORDER BY RANDOM() LIMIT 1` yêu cầu CSDL xáo trộn ngẫu nhiên tất cả các record và bốc lấy bản ghi nằm trên cùng. Trong SQLx ta dùng `fetch_optional` vì có khả năng hệ thống chưa có cái link nào (bảng trống rỗng).

---

## Bài Tập 10: Lấy URL gốc nhưng "Không Redirect"
**Bài toán:** Ứng dụng Mobile cần lấy URL đích để hiển thị trước (preview) cho người dùng xem nó dẫn tới trang nào (chống lừa đảo, phishing), thay vì bị ép nhảy thẳng sang trang đó.
**Yêu cầu:** Viết API `GET /links/resolve/{short_code}`.

**Giải pháp Code:**
```rust
// Handler
pub async fn resolve_link(State(state): State<AppState>, Path(short_code): Path<String>) -> AppResult<Json<serde_json::Value>> {
    // Tận dụng lại hàm get_original_url đã có sẵn
    match link_service::get_original_url(&state.db, &short_code).await {
        Ok(Some(link)) => {
            // Thay vì trả về axum::response::Redirect::to, ta trả về JSON:
            Ok(Json(json!({"original_url": link.original_url})))
        }
        Ok(None) => Err(AppError::NotFound("Link không tồn tại".into())),
        Err(e) => Err(AppError::Database(e)),
    }
}
```
**Giải thích E2E:** Bài tập rèn luyện kỹ năng Tái Sử Dụng (Code Reusability). Chúng ta gọi lại đúng hàm `link_service::get_original_url` (hàm này đã bao bọc sẵn logic cộng lượt click và sinh analytics) nhưng ở bước cuối cùng, thay vì bọc chuỗi trong HTTP 307 Redirect, ta bọc nó trong JSON HTTP 200 thông thường để Frontend hiển thị ra giao diện.
