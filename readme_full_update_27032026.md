# Tong hop cap nhat project (27/03/2026)

## 1. San pham hien co
### Chuc nang chinh
- Dang ky, dang nhap, refresh token, logout (JWT).
- Tao link rut gon (short_code) tu ID theo base62.
- Redirect tu short_code sang URL goc.
- Liet ke link cua user.
- Soft delete link (is_active = false).
- Thong ke analytics theo ngay (tong click theo ngay cho tat ca link cua user).
- Redis cache cho redirect (cache-aside) de tang toc.
- Analytics cap nhat bat dong bo (khong lam cham redirect).

### Bao mat va toi uu
- Auth middleware (Bearer token).
- Validate du lieu (email, password, URL, title).
- Idempotent create: neu user tao lai cung URL va link dang active thi tra ve link cu (khong tao ID moi).
- Unique index chi ap dung cho link active (owner_id + original_url), cho phep tao lai neu link cu da bi soft delete.
- DB pool co cau hinh max/min.
- Redis pool (deadpool-redis) co cau hinh max.

## 2. Thay doi va nang cap da thuc hien
### Core / Routing
- Chuyen sang AppState gom PgPool + Redis pool.
- Tich hop Redis cache cho redirect, co invalidate cache khi xoa link.
- Bo endpoint update link (da xoa route, handler, service, repository, DTO).

### Tao short_code
- Thay doi tu random sang base62 tu ID (khong trung lap).
- Mo rong do dai short_code trong DB len VARCHAR(16).

### Analytics
- Them ghi analytics theo ngay (link_analytics) voi upsert.
- Chuyen ghi analytics sang async (tokio::spawn) de khong lam cham redirect.

### Validation
- Kiem tra email, password, username, URL, title.

### DB / Migrations
- Migration mo rong short_code.
- Migration unique index cho (owner_id, original_url) chi ap dung voi link active.

### Redis
- Dung deadpool-redis de tao pool.
- Caching URL theo short_code (TTL 1 gio).

## 3. Huong dan chay project
### 3.1. Yeu cau
- Rust toolchain
- PostgreSQL
- Redis
- SQLx CLI (neu dung sqlx database reset/migrate)

### 3.2. Bien moi truong goi y
Tao file .env (neu can):
```
DATABASE_URL=postgresql://user:password@localhost:5433/shortener_db
JWT_SECRET=secret
JWT_REFRESH_SECRET=refresh_secret
ACCESS_TOKEN_EXPIRE=900
REFRESH_TOKEN_EXPIRE=2592000
REDIS_URL=redis://127.0.0.1/
DB_MAX_CONNECTIONS=100
DB_MIN_CONNECTIONS=10
REDIS_POOL_MAX=64
BIND_ADDR=0.0.0.0:8080
```

### 3.3. Reset DB va chay migrations
```
sqlx database reset
```
Chon yes khi duoc hoi.

### 3.4. Chay server
```
cargo run
```

### 3.5. Test nhanh bang HTTP file
Su dung file test:
- test_api.http

Quy trinh:
1. Dang ky / login de lay access_token.
2. Tao link.
3. Redirect bang short_code.
4. Xem analytics theo ngay.

## 4. Files chinh lien quan
- src/main.rs (AppState + Redis pool)
- src/state.rs
- src/handlers/link_handler.rs
- src/services/link_service.rs
- src/repositories/link_repository.rs
- src/services/cache_service.rs
- src/utils/validation.rs
- migrations/20260326000000_extend_short_code.up.sql
- migrations/20260327001000_unique_owner_url_active.up.sql

## 5. Ghi chu
- Analytics chi co du lieu sau khi redirect (GET /{short_code}).
- Ghi analytics bat dong bo nen co the tre 1-2 giay.
- Neu can thong ke co san, hay seed data truoc khi demo.
