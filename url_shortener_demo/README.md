# Hướng dẫn cài đặt hạ tầng dự án URL Shortener (Tuần 1)

Dưới đây là các bước chi tiết để thiết lập môi trường lập trình và hạ tầng cơ sở dữ liệu cho dự án rút gọn link.

---

### 1. Cài đặt môi trường WSL 2 và Rust

Sử dụng môi trường Linux (Ubuntu) qua WSL 2 để đảm bảo tính đồng bộ giữa các thành viên.

* **Cài đặt WSL 2:** Mở PowerShell với quyền Administrator và chạy lệnh: `wsl --install`.
* **Cài đặt Rust:** Trong terminal Ubuntu, thực hiện lệnh cài đặt:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
* **Kiểm tra phiên bản:** Chạy `rustc --version` để xác nhận đã cài đặt thành công (phiên bản khuyến nghị 1.92.0 trở lên).

---

### 2. Cấu hình mạng (Fix DNS cho WSL)

Thực hiện thay đổi DNS để tránh lỗi hết thời gian chờ (timeout) khi tải dữ liệu từ Docker Hub.

1. **Chỉnh sửa file resolv.conf:** Chạy lệnh `sudo nano /etc/resolv.conf`.
2. **Thay đổi nameserver:** Xóa nội dung cũ và thêm: `nameserver 8.8.8.8`.
3. **Khóa cấu hình DNS:** Chạy lệnh `sudo nano /etc/wsl.conf` và dán nội dung sau:
```ini
[network]
generateResolvConf = false

```



---

### 3. Cài đặt Docker Desktop

* Tải và cài đặt **Docker Desktop** trên Windows.
* Kích hoạt tính năng **WSL Integration**: Vào **Settings > Resources > WSL Integration**, chọn phân vùng Ubuntu đang sử dụng và nhấn **Apply & Restart**.

---

### 4. Thiết lập dự án và Biến môi trường

Dự án sử dụng file `.env` để quản lý thông tin kết nối nhạy cảm, file này không được đưa lên GitHub.

1. **Clone mã nguồn:** Sử dụng lệnh `git clone [URL_Repository]`.
2. **Tạo file cấu hình:** Truy cập thư mục dự án `cd url_shortener` và tạo file `.env`.
3. **Thiết lập nội dung:** Dán cấu hình kết nối sau vào file `.env`:
```ini
DATABASE_URL=postgres://user:password@localhost:5432/shortener_db
REDIS_URL=redis://127.0.0.1:6379
```.




```



---

### 5. Khởi chạy hạ tầng Database và Redis

Sử dụng Docker Compose để tự động thiết lập các dịch vụ cần thiết.

* **Lệnh khởi chạy:** Tại thư mục gốc dự án, chạy lệnh:
```bash
docker-compose up -d
```.

```


* **Kiểm tra trạng thái:** Chạy lệnh `docker ps` để đảm bảo hai container `url_db` (Postgres) và `url_cache` (Redis) đang ở trạng thái **Up**.
* **Lưu ý:** Có thể xuất hiện cảnh báo `WARN: No services to build`, đây là thông báo bình thường vì hệ thống đang sử dụng các image có sẵn.

---

### 6. Quy trình Git cho nhóm

* **Cập nhật code hàng ngày:** `git pull origin main`.
* **Đưa file vào vùng chờ:** `git add .`.
* **Xác nhận thay đổi:** `git commit -m "Mô tả công việc đã làm"`.
* **Đẩy code lên GitHub:** `git push origin main`.

