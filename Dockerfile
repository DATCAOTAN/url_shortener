# Sử dụng Rust bản Slim (nhẹ, dựa trên Debian)
FROM rust:1-slim-bookworm

# 1. Cài đặt các gói hệ thống cần thiết
# - pkg-config, libssl-dev: Cần cho thư viện mã hóa (bcrypt, reqwest...)
# - postgresql-client: Cần để chạy lệnh psql (cho file seed.sql)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# 2. Cài đặt công cụ hỗ trợ
# - cargo-watch: Để Hot Reload
# - sqlx-cli: Để chạy migration (QUAN TRỌNG: Chỉ cài feature postgres cho nhẹ)
RUN cargo install cargo-watch --locked && \
    cargo install sqlx-cli --no-default-features --features postgres

# Thiết lập thư mục làm việc
WORKDIR /app

# 3. Copy file cấu hình trước (để tận dụng Docker Cache)
COPY Cargo.toml Cargo.lock ./

# 4. Copy toàn bộ source code và các thư mục (migrations, scripts, static...)
COPY . .

# 5. Lệnh chạy mặc định:
# "cargo watch -x run" sẽ theo dõi file thay đổi và chạy lại lệnh "cargo run"
CMD ["cargo", "watch", "-x", "run"]