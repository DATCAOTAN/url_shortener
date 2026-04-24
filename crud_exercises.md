# 5 Bài Tập CRUD Cơ Bản (Create - Read - Update - Delete)

Dưới đây là 5 bài tập thực hành thao tác CRUD cơ bản nhất. Đối với mỗi bài tập, bạn sẽ thực hành việc định nghĩa Route, viết Handler, và viết câu lệnh SQL tương ứng để tương tác với Database.

---

## Bài Tập 11: Lấy chi tiết 1 Link của User (Read One)
**Bài toán:** API hiện tại chỉ cho phép lấy *toàn bộ* danh sách link (`GET /links/my-links`). Hệ thống cần một API để lấy chi tiết đúng 1 link duy nhất khi người dùng ấn vào link đó trên giao diện.
**Yêu cầu:** Viết API `GET /links/{id}` trả về thông tin chi tiết của 1 link. Phải đảm bảo link này thuộc về user đang gọi API.

**Giải pháp Code:**
```rust
// 1. Repository
pub async fn get_my_link(pool: &PgPool, link_id: i64, owner_id: i64) -> Result<Option<Link>, Error> {
    sqlx::query_as!(
        Link,
        "SELECT * FROM links WHERE id = $1 AND owner_id = $2",
        link_id, owner_id
    ).fetch_optional(pool).await
}

// 2. Handler
pub async fn get_single_link(State(state): State<AppState>, Extension(claims): Extension<Claims>, Path(id): Path<i64>) -> AppResult<Json<LinkResponse>> {
    let user_id = claims.sub.parse::<i64>().unwrap();
    match link_repository::get_my_link(&state.db, id, user_id).await? {
        Some(link) => {
            // Map Link sang LinkResponse rồi trả về...
            Ok(Json(LinkResponse { /* ... */ }))
        }
        None => Err(AppError::NotFound("Link không tồn tại".into()))
    }
}
```
**Giải thích E2E:** Trích xuất biến `id` từ URL (Path Param). Điều kiện tiên quyết trong SQL là `owner_id = $2` (Tránh lỗi bảo mật Insecure Direct Object Reference - IDOR, user này xem trộm link user khác).

---

## Bài Tập 12: Cập nhật nội dung Link (Update - PUT)
**Bài toán:** Người dùng lỡ dán nhầm `original_url` lúc tạo link, họ muốn sửa lại URL đích đó mà không muốn phải xóa đi tạo lại link mới (vì đã lỡ in mã QR).
**Yêu cầu:** Viết API `PUT /links/{id}` nhận JSON chứa `original_url` và `title` mới. Cập nhật chúng vào CSDL và xóa Redis cache tương ứng.

**Giải pháp Code:**
```rust
// 1. DTO
#[derive(Deserialize)]
pub struct UpdateLinkRequest {
    pub original_url: String,
    pub title: Option<String>,
}

// 2. Repository
pub async fn update_link(pool: &PgPool, link_id: i64, owner_id: i64, url: &str, title: Option<String>) -> Result<Option<Link>, Error> {
    sqlx::query_as!(
        Link,
        "UPDATE links SET original_url = $1, title = $2, updated_at = NOW() WHERE id = $3 AND owner_id = $4 RETURNING *",
        url, title, link_id, owner_id
    ).fetch_optional(pool).await
}

// 3. Handler
pub async fn update_my_link(State(state): State<AppState>, Extension(claims): Extension<Claims>, Path(id): Path<i64>, Json(payload): Json<UpdateLinkRequest>) -> AppResult<Json<LinkResponse>> {
    let user_id = claims.sub.parse::<i64>().unwrap();
    match link_repository::update_link(&state.db, id, user_id, &payload.original_url, payload.title).await? {
        Some(link) => {
            // Quan trọng: Vì original_url đã đổi, phải xóa cache cũ trên Redis để tránh nhảy nhầm link cũ
            let _ = cache_service::invalidate_cache(&state.redis, &link.short_code).await;
            Ok(Json(LinkResponse { /* map link... */ }))
        }
        None => Err(AppError::NotFound("Link không tồn tại".into()))
    }
}
```
**Giải thích E2E:** Phương thức HTTP `PUT` dùng để ghi đè toàn bộ resource. Lệnh `UPDATE ... RETURNING *` trong PostgreSQL rất tiện lợi: vừa ghi dữ liệu vừa trả về chính bản ghi vừa được sửa. Cực kỳ lưu ý: vì URL gốc đã bị đổi, bộ nhớ Redis Cache của short_code đó đang lưu URL cũ sẽ bị sai lệch, nên bước bắt buộc là phải chạy lệnh `invalidate_cache`.

---

## Bài Tập 13: Xóa vĩnh viễn Link (Hard Delete)
**Bài toán:** API hiện tại `DELETE /links/{id}` chỉ là xóa mềm (soft delete - đổi `is_active = FALSE`). Nhiều user muốn xóa sạch dữ liệu thực sự khỏi DB vì lý do riêng tư.
**Yêu cầu:** Viết API `DELETE /links/{id}/hard` thực hiện lệnh `DELETE FROM` để xóa bản ghi khỏi ổ cứng.

**Giải pháp Code:**
```rust
// 1. Repository
pub async fn hard_delete_link(pool: &PgPool, link_id: i64, owner_id: i64) -> Result<Option<String>, Error> {
    // Trả về short_code để Tầng Handler đi xóa Redis
    let result = sqlx::query!(
        "DELETE FROM links WHERE id = $1 AND owner_id = $2 RETURNING short_code",
        link_id, owner_id
    ).fetch_optional(pool).await?;
    
    Ok(result.map(|r| r.short_code))
}

// 2. Handler
pub async fn destroy_link(State(state): State<AppState>, Extension(claims): Extension<Claims>, Path(id): Path<i64>) -> AppResult<Json<serde_json::Value>> {
    let user_id = claims.sub.parse::<i64>().unwrap();
    match link_repository::hard_delete_link(&state.db, id, user_id).await? {
        Some(short_code) => {
            let _ = cache_service::invalidate_cache(&state.redis, &short_code).await;
            Ok(Json(json!({"message": "Đã xóa vĩnh viễn"})))
        }
        None => Err(AppError::NotFound("Link không tồn tại".into()))
    }
}
```
**Giải thích E2E:** Câu lệnh `DELETE FROM` trong SQL sẽ bốc hơi dữ liệu khỏi bảng. Tương tự như sửa link, khi dữ liệu DB mất đi, chúng ta phải "dọn rác" trên Redis luôn để link đó thực sự không truy cập được nữa.

---

## Bài Tập 14: Lấy danh sách Users phân trang (Read List Pagination)
**Bài toán:** API hiện tại `GET /admin/users` sử dụng `SELECT * FROM users` không có `LIMIT`. Nếu có 1 triệu user hệ thống sẽ treo cứng vì RAM quá tải.
**Yêu cầu:** Sửa lại API `GET /admin/users` nhận query param `?page=x&page_size=y` và phân trang tương tự như phần Link.

**Giải pháp Code:**
```rust
// 1. DTO
#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// 2. Repository
pub async fn get_users_paginated(pool: &PgPool, page: i64, page_size: i64) -> Result<Vec<User>, Error> {
    let offset = (page - 1) * page_size;
    sqlx::query_as!(
        User,
        "SELECT * FROM users ORDER BY id DESC LIMIT $1 OFFSET $2",
        page_size, offset
    ).fetch_all(pool).await
}

// 3. Handler
pub async fn list_users(State(state): State<AppState>, Query(params): Query<PaginationQuery>) -> AppResult<Json<Vec<UserResponse>>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);

    let users = user_repository::get_users_paginated(&state.db, page, page_size).await?;
    // Map to JSON...
}
```
**Giải thích E2E:** Ứng dụng quy tắc cơ bản nhất của phân trang: `OFFSET = (Page - 1) * Limit`. Việc dùng `unwrap_or()` kết hợp `clamp(1, 100)` giúp bảo vệ API nếu Client vô tình gửi lên page_size = 1 tỷ.

---

## Bài Tập 15: Admin tạo Link "hộ" User (Create Custom Owner)
**Bài toán:** Quản trị viên (Admin) thi thoảng cần tạo link giúp một khách hàng VIP, làm sao để gán chủ sở hữu (owner_id) cho khách hàng đó thay vì gán cho Admin?
**Yêu cầu:** Viết API `POST /admin/links` nhận thêm tham số `target_user_id` trong Body để Admin tự gán chủ.

**Giải pháp Code:**
```rust
// 1. DTO
#[derive(Deserialize)]
pub struct AdminCreateLinkRequest {
    pub original_url: String,
    pub target_user_id: i64, // Admin truyền mã user khách hàng vào đây
}

// 2. Handler
pub async fn admin_create_link(
    State(state): State<AppState>, 
    Extension(_claims): Extension<Claims>, // Đã check role = admin ở middleware
    Json(payload): Json<AdminCreateLinkRequest>
) -> AppResult<Json<LinkResponse>> {
    
    // Tận dụng chính hàm create_short_link ở service, nhưng truyền payload.target_user_id
    let link = link_service::create_short_link(
        &state.db,
        &payload.original_url,
        Some(payload.target_user_id), // Gắn ID của user cần tạo hộ
        None,
        None,
    ).await.map_err(AppError::Database)?;

    // Map ra LinkResponse...
}
```
**Giải thích E2E:** Bài tập nhấn mạnh tính tái sử dụng Code (Code Reusability). Logic khởi tạo link rút gọn rất rườm rà (tính toán ID tự tăng, Base62, Analytics, ...) nhưng vì chúng ta đã đóng gói nó gọn gàng trong `link_service::create_short_link(..., owner_id, ...)`, ta chỉ việc gọi lại hàm đó và truyền vào tham số `owner_id` phù hợp. Middleware Auth của Admin đã bảo vệ an toàn để user thường không dùng lén API này được.
