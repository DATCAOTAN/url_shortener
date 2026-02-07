# 🗄️ Database Migrations & Seeding Guide

Thư mục này chứa toàn bộ lịch sử thay đổi cấu trúc Database (Schema) của dự án **URL Shortener**, được quản lý bởi công cụ **SQLx**.

Mọi thay đổi về cấu trúc bảng (Table), chỉ mục (Index), hay khóa ngoại (Foreign Key) đều phải được thực hiện thông qua Migration để đảm bảo sự đồng bộ giữa các môi trường (Dev, Production, và máy của các thành viên khác).

---
## 🚀 Quy trình làm việc (Workflow)

Chúng ta sử dụng **Docker** cho Database (PostgreSQL và Redis), còn ứng dụng Rust chạy trực tiếp trên máy host. Migration được thực thi bằng **sqlx-cli** từ máy local.

### 0. Thiết lập môi trường

**Khởi động Database:**
```bash
docker-compose up -d
```

**Cài đặt SQLx CLI (nếu chưa có):**

> ⚠️ **Lưu ý:** `sqlx-cli` là **command-line tool** để chạy migrations, KHÔNG phải dependency trong `Cargo.toml`.  
> Thư viện `sqlx` trong project sẽ tự động tải khi `cargo build`, nhưng CLI tool cần cài riêng.

```bash
# Kiểm tra đã cài chưa:
sqlx --version

# Nếu chưa có, cài đặt:
cargo install sqlx-cli --no-default-features --features postgres
```

### 1. Tạo Migration mới

Khi bạn cần thay đổi DB (ví dụ: thêm bảng mới, thêm cột...), hãy chạy lệnh sau:

```bash
# Cú pháp: sqlx migrate add -r <ten_mo_ta>
# Ví dụ: Thêm bảng payment
sqlx migrate add -r add_payment_table
```

* **Kết quả:** Lệnh này sẽ sinh ra 2 file trong thư mục `migrations/`:
  * `<timestamp>_ten.up.sql`: Viết code tạo/thêm mới vào đây.
  * `<timestamp>_ten.down.sql`: Viết code xóa/hoàn tác vào đây.

### 2. Chạy Migration (Apply)

Để cập nhật Database lên phiên bản mới nhất:

```bash
sqlx migrate run
```

**Lưu ý:** Đảm bảo `DATABASE_URL` đã được set (xem phần Thiết lập môi trường).

### 3. Hoàn tác Migration (Revert)

Nếu lỡ chạy sai hoặc muốn test tính năng rollback, lệnh này sẽ chạy file `.down.sql` của migration gần nhất:

```bash
sqlx migrate revert
```

### 4. Kiểm tra trạng thái

Xem danh sách các migration đã chạy và chưa chạy:

```bash
sqlx migrate info
```

---
## 🌱 Seeding Data (Nạp dữ liệu mẫu)

Dự án có sẵn bộ dữ liệu mẫu để phục vụ kiểm thử (Test) và phát triển (Dev).

* **File nguồn:** `scripts/seed.sql`
* **Dữ liệu bao gồm:**
* User Admin & User thường.
* Các Link mẫu.
* Dữ liệu thống kê (Analytics) giả lập.



**Lệnh nạp dữ liệu (Lưu ý: Lệnh này sẽ XÓA SẠCH dữ liệu cũ):**

```bash
# Windows PowerShell:
Get-Content scripts\seed.sql | docker exec -i url_db psql -U user -d shortener_db

# Linux/Mac/Git Bash:
docker exec -i url_db psql -U user -d shortener_db < scripts/seed.sql
```

**Tài khoản mặc định sau khi seed:**
| Role | Username | Password |
| :--- | :--- | :--- |
| **Admin** | `admin` | `password123` |
| **User** | `basic_user` | `password123` |

---

## ⚠️ Quy tắc vàng (Team Rules)

Để tránh xung đột (Conflict) và lỗi mất dữ liệu, toàn bộ team cần tuân thủ:

1. **KHÔNG sửa file Migration đã chạy:**
Nếu file `.up.sql` đã được merge vào nhánh chính (`main`) hoặc đã chạy trên máy người khác, **tuyệt đối không được sửa nội dung**.
* *Cách đúng:* Tạo một file migration mới để sửa đổi (ví dụ: `fix_column_type`).


2. **Luôn viết file Down:**
Luôn phải viết code trong file `.down.sql` để đảm bảo hệ thống có thể rollback khi gặp sự cố nghiêm trọng.
3. **Không chứa logic nghiệp vụ:**
Migration chỉ nên chứa lệnh DDL (`CREATE`, `ALTER`, `DROP`). Không nên chứa lệnh `INSERT` dữ liệu nghiệp vụ (hãy để việc đó cho file Seed).
4. **Kiểm tra kỹ Index:**
Khi tạo bảng mới hoặc khóa ngoại, hãy nhớ tạo `INDEX` cho các cột hay được query (như `short_code`, `user_id`) để đảm bảo hiệu năng High Load.

---
## 🆘 Troubleshooting (Sửa lỗi)

**Lỗi: `Database locked` hoặc Migration bị kẹt**
Nếu quá trình migration bị ngắt giữa chừng, bảng `_sqlx_migrations` có thể bị khóa. Hãy thử restart lại container DB:

```bash
docker-compose restart db
```

**Cần Reset hoàn toàn Database?**
Nếu Database trên máy Dev quá lộn xộn và bạn muốn làm lại từ đầu (Sạch sẽ 100%):

```bash
# 1. Xóa toàn bộ database cũ và tạo lại db trắng
sqlx database reset

# 2. Nạp lại dữ liệu mẫu
# Windows PowerShell:
Get-Content scripts\seed.sql | docker exec -i url_db psql -U user -d shortener_db

# Linux/Mac:
docker exec -i url_db psql -U user -d shortener_db < scripts/seed.sql
```

**Kiểm tra kết nối Database:**
```bash
# Kết nối vào PostgreSQL
docker exec -it url_db psql -U user -d shortener_db

# Xem danh sách tables
docker exec url_db psql -U user -d shortener_db -c "\dt"
```