# Giải thích chi tiết Kỹ thuật Dự án URL Shortener (End-to-End)

Tài liệu này giải thích chi tiết luồng hoạt động từ đầu đến cuối (End-to-End) của 5 tính năng cốt lõi trong dự án URL Shortener.

---

## 1. Advanced Search Link (Tìm kiếm nâng cao)
**Mục tiêu:** Cho phép người dùng và admin lọc các đường dẫn (links) dựa trên nhiều tiêu chí phức tạp.

**Luồng hoạt động (End-to-End):**
1. **API Layer (`link_handler.rs`):**
   - Client gửi request `GET /links/advanced-search` kèm theo các Query Parameters (e.g. `?min_clicks=10&domain=google&is_active=true`).
   - Tham số được parse tự động vào DTO `AdvancedSearchQuery` bởi Axum extractor.
   - Handler kiểm tra tính hợp lệ cơ bản: `min_clicks <= max_clicks`, `from <= to`. Sau đó gọi xuống Service Layer.
2. **Service Layer (`link_service.rs`):** 
   - Đóng vai trò cầu nối, chuyển các tham số này xuống Repository.
3. **Repository Layer (`link_repository.rs`):**
   - Sử dụng `sqlx::QueryBuilder` để **build SQL động (Dynamic SQL)**. Mặc định QueryBuilder sẽ tạo chuỗi SQL: `SELECT ... FROM links WHERE owner_id = $1`.
   - Các tham số tùy chọn (`Option<T>`) sẽ được kiểm tra:
     - Dựa trên `is_active` (true/false/none), hệ thống nối thêm chuỗi SQL lọc trạng thái cờ trong DB kết hợp thời gian `expires_at <= NOW()`.
     - Nếu có `min_clicks`, nối thêm `AND click_count >= $2`.
     - Nếu có `domain`, dùng toán tử `ILIKE` để tìm kiếm tương đối (không phân biệt hoa thường).
   - Biến được gán qua `.push_bind()` để chống lỗi bảo mật SQL Injection.
   - Kết quả trả về cho Handler và được parse sang `LinkResponse`.

---

## 2. Cooldown Redis (Giới hạn tỷ lệ tạo link - Rate Limiting)
**Mục tiêu:** Ngăn chặn spam bằng cách giới hạn một user chỉ được tạo 1 link mới sau mỗi `X` giây.

**Luồng hoạt động (End-to-End):**
1. **API Layer (`link_handler.rs`):**
   - Khi gọi `POST /links`, hệ thống đọc biến môi trường `LINK_CREATE_COOLDOWN` (mặc định 5s).
   - Gọi hàm `cache_service::try_acquire_link_cooldown`.
2. **Cache Service (`cache_service.rs`):**
   - Hàm này thực thi câu lệnh Redis: `SET cooldown:create_link:user:{user_id} 1 EX {cooldown_seconds} NX`
     - `EX {cooldown_seconds}`: Khóa này sẽ tự động biến mất khỏi bộ nhớ Redis sau `X` giây.
     - `NX` (Not eXists): Lệnh `SET` chỉ thành công nếu khóa **chưa tồn tại**. 
   - Nếu `NX` thành công, Redis trả về `OK` (ứng với user được quyền tạo link).
   - Nếu `NX` thất bại (khóa vẫn còn do chưa hết `X` giây), Redis trả về `Nil`.
3. **Xử lý kết quả:**
   - Nếu hàm trả về `false`, API chặn lại và ném ra lỗi HTTP 429 `Too Many Requests`.

---

## 3. Phân trang & Sắp xếp (Pagination & Sorting)
**Mục tiêu:** Tối ưu hóa hiệu suất truy vấn CSDL khi user/admin có hàng triệu links bằng cách chỉ lấy một phần dữ liệu (page).

**Luồng hoạt động (End-to-End):**
1. **API Layer:**
   - DTO `ListLinksQuery` nhận các tham số `page`, `page_size`, `sort_by`, `sort_order`.
   - Kiểm tra logic: `page >= 1`, `page_size` từ 1 đến 100 (chống user set `page_size` quá lớn làm sập DB).
   - Giới hạn các cột được phép sắp xếp: chỉ cho phép `created_at`, `click_count`, `title` (White-listing) để ngăn chặn tấn công chèn SQL.
2. **Repository Layer (`link_repository.rs`):**
   - Tính toán `OFFSET = (page - 1) * page_size`. Ví dụ: Trang 2, kích thước 20 -> Bỏ qua 20 bản ghi đầu tiên (`OFFSET = 20`).
   - Viết raw SQL query: `SELECT ... ORDER BY {order_by} {order_dir} LIMIT $1 OFFSET $2`.
   - Tham số $1 là `LIMIT` (page_size), $2 là `OFFSET`.

---

## 4. Quản lý Link hết hạn (Link Expiration)
**Mục tiêu:** Đảm bảo link chỉ có thể sử dụng trong một khoảng thời gian nhất định (TTL - Time to Live).

**Luồng hoạt động (End-to-End):**
1. **Lưu trữ lúc tạo:** 
   - Nhận `ttl_seconds` từ Request. 
   - Tính thời điểm chết: `expires_at = thời điểm tạo + ttl_seconds` và lưu timestamp này vào cột `expires_at` của CSDL.
2. **Quản lý trạng thái (Model `Link`):**
   - Hàm `is_active_now()` được thiết kế để kết hợp 2 yếu tố: Nếu DB báo `is_active = false` HOẶC `expires_at <= NOW()` (đã qua thời điểm hết hạn), hàm sẽ trả về false. Giúp các API List luôn hiển thị trạng thái chính xác.
3. **Khi Redirect (`link_handler.rs`):**
   - Nếu link được truy cập, hệ thống kiểm tra `link.expires_at`.
   - Nếu thời gian hiện tại (`chrono::Utc::now()`) đã lớn hơn `expires_at`, ngay lập tức chặn lại và trả về lỗi HTTP 404 Not Found.
   - Cập nhật lại thời gian sống trong Redis (để Redis cũng tự xóa khi đến hạn).

---

## 5. Giảm tải Postgres bằng cách lưu URL vào Redis
**Mục tiêu:** Giảm tối đa chi phí đọc cho PostgreSQL. Redis hoạt động trên RAM nên tốc độ đọc cực kỳ nhanh (dưới 1ms), chịu được lưu lượng cực lớn.

**Luồng hoạt động (End-to-End):**
Hệ thống kết hợp cả 2 chiến lược Cache hiện đại: **Write-Through (Eager Caching)** và **Cache-Aside (Lazy Caching)**.

1. **Eager Caching (Lưu ngay lúc tạo):**
   - API `POST /links`: Ngay sau khi chèn `original_url` vào PostgreSQL thành công, hệ thống không dừng lại mà lập tức gọi `set_cached_url` để đưa cặp key-value `(short_code -> original_url)` lên Redis.
   - **Lợi ích:** Đảm bảo "First Hit" (lần click đầu tiên của user) đã có sẵn trong Redis, không cần đụng đến DB.

2. **Cache-Aside (Kiểm tra khi truy cập):**
   - API `GET /{short_code}` (Hàm `redirect_link`):
     - **Bước 1 (Check Cache):** Gọi vào Redis tìm short_code. Nếu có => Redirect luôn. (Tốc độ siêu nhanh, không chạm DB).
     - **Bước 2 (Cache Miss):** Nếu Redis không có (có thể do Redis khởi động lại, bị trôi cache, hoặc đã quá hạn mặc định), hệ thống mới chọc vào PostgreSQL (`link_repository::find_active_by_short_code`).
     - **Bước 3 (Set Cache lại):** Khi lấy được kết quả từ DB, hệ thống lập tức mang kết quả này lưu ngược lại vào Redis để các lần truy cập sau được tăng tốc, đồng thời tính toán TTL còn lại sao cho khớp với `expires_at` của link.

Nhờ cơ chế này, PostgreSQL được bảo vệ an toàn khỏi các đợt lưu lượng tăng đột biến (Spike Traffic) khi một đường link trở nên "viral".
