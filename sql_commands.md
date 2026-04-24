# Tổng Hợp Các Câu Lệnh SQL Trong Dự Án (Kèm Giải Thích Chi Tiết)

Dự án này sử dụng PostgreSQL kết hợp với `sqlx` (Rust). Dưới đây là danh sách toàn bộ các câu lệnh SQL gốc (Raw SQL) đang được thực thi ngầm phía dưới các Repository, được phân loại theo từng Bảng dữ liệu. Mỗi câu lệnh đều có phần giải thích cơ chế hoạt động.

---

## 1. Bảng `users` (Quản lý Người dùng)

- **Đăng ký User mới (Insert):**
  ```sql
  INSERT INTO users (username, email, password_hash)
  VALUES ($1, $2, $3)
  RETURNING *;
  ```
  > **Giải thích:** Thêm mới một bản ghi vào bảng `users`. Mệnh đề `RETURNING *` là tính năng đặc biệt của PostgreSQL giúp lập tức trả về toàn bộ dữ liệu của bản ghi vừa chèn (bao gồm `id` tự tăng và `created_at`), giúp API trả về kết quả ngay mà không cần gọi lệnh SELECT lần 2.

- **Tìm User theo ID / Email (Select One):**
  ```sql
  SELECT * FROM users WHERE id = $1;
  SELECT * FROM users WHERE email = $1;
  ```
  > **Giải thích:** Lấy toàn bộ thông tin của 1 user duy nhất dựa trên ID hoặc Email. Các cột ID và Email thường được đánh Index (chỉ mục) nên tốc độ tìm kiếm cực kỳ nhanh.

- **Lấy toàn bộ User (Select All - Admin):**
  ```sql
  SELECT * FROM users ORDER BY created_at DESC;
  ```
  > **Giải thích:** Quét toàn bộ bảng users và sắp xếp giảm dần theo thời gian tạo (`DESC`), giúp danh sách hiển thị những user đăng ký mới nhất lên trên cùng.

- **Xóa mềm User (Soft Delete):**
  ```sql
  UPDATE users SET is_active = FALSE, updated_at = NOW() 
  WHERE id = $1 RETURNING *;
  ```
  > **Giải thích:** Thay vì xóa hẳn dữ liệu khỏi ổ cứng, ta chỉ cập nhật cờ `is_active` thành cờ `FALSE` (khóa tài khoản). Hàm `NOW()` tự động lấy giờ hiện tại của server DB để lưu lịch sử sửa đổi.

- **Xóa cứng User (Hard Delete):**
  ```sql
  DELETE FROM users WHERE id = $1;
  ```
  > **Giải thích:** Lệnh xóa vĩnh viễn (bốc hơi) dữ liệu của user khỏi bảng. Sẽ gây lỗi `Foreign Key Constraint` nếu user này đang có chứa Links ở bảng khác (trừ khi set `ON DELETE CASCADE` ở Database).

---

## 2. Bảng `refresh_tokens` (Quản lý Phiên đăng nhập)

- **Lưu Refresh Token mới:**
  ```sql
  INSERT INTO refresh_tokens (user_id, token_hash, expires_at) 
  VALUES ($1, $2, $3) RETURNING *;
  ```
  > **Giải thích:** Khi user đăng nhập thành công, một phiên mới (session) được tạo ra. Dữ liệu nhạy cảm là token đã được băm (`token_hash`) để tránh bị lộ nếu hacker tấn công DB.

- **Kiểm tra Refresh Token tồn tại:**
  ```sql
  SELECT * FROM refresh_tokens WHERE token_hash = $1;
  ```
  > **Giải thích:** Khi user dùng Refresh Token để xin Access Token mới, hệ thống tìm kiếm trong DB xem token đó có tồn tại hay không.

- **Thu hồi (Revoke) 1 Token cụ thể (Khi Đăng xuất):**
  ```sql
  UPDATE refresh_tokens SET revoked_at = $1 
  WHERE token_hash = $2 AND revoked_at IS NULL;
  ```
  > **Giải thích:** Đăng xuất thiết bị hiện tại. Việc gán giá trị thời gian cho `revoked_at` (thay vì xóa) giúp Admin lưu lại được Log Audit (vết thời gian) lúc user chủ động đăng xuất. Điều kiện `revoked_at IS NULL` đảm bảo không update lại những token đã thu hồi.

- **Thu hồi toàn bộ Token của 1 User (Đăng xuất khỏi mọi thiết bị):**
  ```sql
  UPDATE refresh_tokens SET revoked_at = $1 
  WHERE user_id = $2 AND revoked_at IS NULL;
  ```
  > **Giải thích:** Update hàng loạt (`UPDATE ... WHERE user_id`). Ứng dụng trong chức năng "Đổi mật khẩu" hoặc "Bị hack tài khoản", ép tất cả các điện thoại/máy tính khác phải văng ra ngoài.

---

## 3. Bảng `links` (Quản lý Đường dẫn rút gọn)

- **Lấy ID tiếp theo (Sequence - Dùng cho Base62 Encode):**
  ```sql
  SELECT nextval('links_id_seq') AS "id!";
  ```
  > **Giải thích:** Lấy ID kế tiếp từ chuỗi tự tăng (Sequence) của Postgres mà chưa cần `INSERT`. Kỹ thuật này bắt buộc phải có để lấy số ID mang đi mã hóa Base62 thành `short_code` trước khi thực sự lưu vào bảng.

- **Tạo Link rút gọn:**
  ```sql
  INSERT INTO links (id, owner_id, original_url, short_code, title, expires_at)
  VALUES ($1, $2, $3, $4, $5, $6)
  RETURNING id, owner_id, original_url, short_code, title, click_count, is_active, expires_at, created_at, updated_at;
  ```
  > **Giải thích:** Chèn bản ghi Link. Truyền sẵn giá trị `id` lấy từ bước trước vào cột `$1`. Dùng `RETURNING` để nhận lại toàn bộ schema giúp API trả thẳng về response json.

- **Tìm Link Đang Hoạt Động (Redirect Handler):**
  ```sql
  SELECT ... FROM links 
  WHERE short_code = $1 
    AND (is_active IS NULL OR is_active = TRUE) 
    AND (expires_at IS NULL OR expires_at > NOW());
  ```
  > **Giải thích:** Câu truy vấn lõi của hệ thống Redirect. Nó ép buộc link phải thỏa mãn 3 điều kiện: Mã khớp, Chưa bị khóa mềm, và Thời gian chưa bị quá hạn so với giờ hiện tại của CSDL (`NOW()`).

- **Cộng dồn lượt truy cập (Click Tracking):**
  ```sql
  UPDATE links SET click_count = click_count + 1 WHERE id = $1;
  ```
  > **Giải thích:** Lệnh đếm lượt click. PostgreSQL xử lý phép cộng biến `click_count + 1` một cách nguyên tử (Atomic), chống lỗi Race Condition khi có 1000 người click cùng 1 mili-giây.

- **Lấy danh sách Link của User có Phân trang & Sắp xếp:**
  ```sql
  SELECT ... FROM links 
  WHERE owner_id = $1 
  ORDER BY {click_count/title/created_at} {ASC/DESC} 
  LIMIT $2 OFFSET $3;
  ```
  > **Giải thích:** `LIMIT` quy định kích thước 1 trang (Page Size), `OFFSET` quy định số bản ghi phải nhảy qua. Các thông số ORDER BY được hệ thống kiểm soát chặt (Whitelist) để chống SQL Injection.

- **Tìm kiếm nâng cao (Dynamic SQL Builder):**
  ```sql
  SELECT ... FROM links 
  WHERE owner_id = $1 
    AND COALESCE(click_count, 0) >= $2
    AND original_url ILIKE '%google%'
  ORDER BY created_at DESC;
  ```
  > **Giải thích:** `COALESCE` dùng để ép giá trị `NULL` thành số `0` để so sánh toán học không bị lỗi. `ILIKE` dùng để tìm kiếm chuỗi con không phân biệt hoa thường. Các mảnh SQL này được gắn động vào tùy theo người dùng gửi filter nào.

---

## 4. Bảng `link_analytics` (Thống kê theo ngày)

- **Cộng lượt Click theo Ngày (Sử dụng lệnh UPSERT):**
  ```sql
  INSERT INTO link_analytics (link_id, date, clicks) 
  VALUES ($1, $2, 1) 
  ON CONFLICT (link_id, date) 
  DO UPDATE SET clicks = link_analytics.clicks + 1;
  ```
  > **Giải thích:** Hay còn gọi là kỹ thuật `UPSERT` (Update or Insert). Hệ thống cố gắng chèn 1 ngày mới với 1 lượt click. Nếu ngày đó đã được lưu (gây lỗi CONFLICT do cặp khóa chính `(link_id, date)`), DB sẽ không báo lỗi mà tự động chuyển sang chế độ UPDATE cộng thêm 1.

- **Lấy báo cáo Analytics cho 1 khoảng thời gian (JOIN 2 Bảng):**
  ```sql
  SELECT la.date, COALESCE(SUM(la.clicks), 0) AS "total_clicks!" 
  FROM link_analytics la 
  JOIN links l ON l.id = la.link_id 
  WHERE l.owner_id = $1 
    AND la.date BETWEEN $2 AND $3 
  GROUP BY la.date 
  ORDER BY la.date;
  ```
  > **Giải thích:** Lệnh `JOIN` ghép bảng thống kê với bảng Link gốc. Nhờ có `owner_id` từ bảng Link, API đảm bảo bảo mật. Hàm tổng hợp `SUM()` cộng dồn lượt click và `GROUP BY` cắt nhỏ báo cáo ra thành từng điểm trên biểu đồ (theo từng ngày `la.date`).

---

## 5. Các Câu Lệnh SQL Nâng Cao (Dành Cho Bài Tập / Luyện Tập)

- **5.1. Thống kê User có nhiều Link nhất:**
  ```sql
  SELECT u.username, COUNT(l.id) as total_links
  FROM users u LEFT JOIN links l ON u.id = l.owner_id
  GROUP BY u.id ORDER BY total_links DESC LIMIT 5;
  ```
  > **Giải thích:** Phải dùng `LEFT JOIN` thay vì `INNER JOIN` để những user chưa tạo link nào vẫn hiện ra với số lượng = 0. Hàm `COUNT` đếm số lượng Link tương ứng của từng ID user.

- **5.2. Thống kê Domain được rút gọn nhiều nhất:**
  ```sql
  SELECT substring(original_url from '^https?://([^/]+)') as domain, COUNT(*) as count
  FROM links GROUP BY domain ORDER BY count DESC LIMIT 10;
  ```
  > **Giải thích:** Dùng Regex `^https?://([^/]+)` kết hợp hàm `substring` để cắt bỏ "http://" và các đường dẫn con ở phía sau, chỉ lấy phần lõi `domain.com`. Nhóm theo domain đó để tìm ra tên miền phổ biến nhất hệ thống.

- **5.3. Dọn dẹp tài khoản "Ma" (Không tạo link nào trong 1 năm qua):**
  ```sql
  DELETE FROM users 
  WHERE id NOT IN (SELECT DISTINCT owner_id FROM links WHERE created_at > NOW() - INTERVAL '1 year') 
  AND created_at < NOW() - INTERVAL '1 year';
  ```
  > **Giải thích:** `NOT IN` kết hợp Subquery. Subquery gom danh sách tất cả những ai có hoạt động trong 1 năm qua. Phép toán `INTERVAL` trừ đi 1 khoảng thời gian một cách tự nhiên bằng ngôn ngữ SQL.

- **5.4. Lấy Link viral nhất của *từng* người dùng (Window Function):**
  ```sql
  SELECT * FROM (
      SELECT l.*, ROW_NUMBER() OVER(PARTITION BY owner_id ORDER BY click_count DESC) as rn
      FROM links l
  ) tmp WHERE rn = 1;
  ```
  > **Giải thích:** `ROW_NUMBER()` là hàm cửa sổ (Window function) cực mạnh. Nó chia dữ liệu thành các cụm (Partition) theo từng user. Trong mỗi cụm, nó xếp hạng link theo lượt click từ cao xuống thấp (1,2,3...). Ở truy vấn ngoài (Wrap), ta chỉ lọc lấy các dòng có hạng = 1.

- **5.5. Đóng băng các tài khoản Spam (Tạo quá 100 links/ngày):**
  ```sql
  UPDATE users SET is_active = FALSE 
  WHERE id IN (
      SELECT owner_id FROM links WHERE created_at::date = CURRENT_DATE 
      GROUP BY owner_id HAVING COUNT(*) > 100
  );
  ```
  > **Giải thích:** Mệnh đề `HAVING` là phiên bản `WHERE` dành riêng cho các hàm Aggregate. Nó nhóm số lượng link theo user trong ngày hôm nay (`CURRENT_DATE`), nếu số lượng lớn hơn 100, trả ID đó ra ngoài để lệnh UPDATE khóa cờ lại.

- **5.6. Báo cáo Tổng click toàn hệ thống trong 7 ngày qua:**
  ```sql
  SELECT SUM(clicks) as total_clicks 
  FROM link_analytics WHERE date >= CURRENT_DATE - INTERVAL '7 days';
  ```
  > **Giải thích:** Đơn giản và siêu tốc để tính tổng (`SUM`) nhờ lọc bằng `INTERVAL` để lấy khoảng thời gian rolling (quay vòng) 7 ngày gần nhất.

- **5.9. Tìm kiếm Full-Text Search cơ bản:**
  ```sql
  SELECT * FROM links 
  WHERE title ILIKE '%khuyến mãi%' OR original_url ILIKE '%shopee.vn%';
  ```
  > **Giải thích:** Cú pháp `ILIKE` giúp CSDL bỏ qua việc phân biệt chữ hoa, chữ thường khi so khớp chuỗi (ví dụ: gõ "KHUYẾN Mãi" vẫn ra).

- **5.10. Thêm tiền tố vào tất cả tiêu đề link đã bị khóa (Bulk Update):**
  ```sql
  UPDATE links 
  SET title = '[Đã khóa] ' || COALESCE(title, 'Không tên') 
  WHERE owner_id = $1 AND is_active = FALSE;
  ```
  > **Giải thích:** Toán tử `||` trong Postgres dùng để ghép chuỗi (Concat). Nếu `title` đang bị `NULL`, `COALESCE` sẽ thay bằng chữ 'Không tên' để phép ghép chuỗi không bị lỗi mất dữ liệu.
