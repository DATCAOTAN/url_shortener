# URL Shortener API - README Tong Hop

Tai lieu nay la README duy nhat cua du an, tong hop:
- Cach chay he thong
- Bien moi truong can thiet
- Co che auth va phan quyen
- Toan bo API hien co
- Cach test nhanh

## 1. Tong quan

Du an backend URL Shortener su dung:
- Rust 2021 + Axum
- PostgreSQL qua SQLx
- Redis qua deadpool-redis
- JWT Access Token va Refresh Token
- OpenAPI JSON qua utoipa

## 2. Khoi dong nhanh

Yeu cau:
- Docker
- Docker Compose

Lenh chay:
- docker compose up --build -d

Kiem tra service:
- GET http://localhost:8080/health/live
- GET http://localhost:8080/health/ready

OpenAPI:
- http://localhost:8080/api-docs/openapi.json
- http://localhost:8080/docs

## 3. Bien moi truong quan trong

- DATABASE_URL
- REDIS_URL
- JWT_SECRET (toi thieu 32 ky tu)
- JWT_REFRESH_SECRET (toi thieu 32 ky tu)
- ACCESS_TOKEN_EXPIRE (mac dinh 900 giay)
- REFRESH_TOKEN_EXPIRE (mac dinh 2592000 giay)
- RATE_LIMIT_REQUESTS_PER_MINUTE (mac dinh 120)
- CORS_ALLOWED_ORIGINS
- BIND_ADDR (mac dinh 0.0.0.0:8080)

Luu y:
- Neu JWT_SECRET hoac JWT_REFRESH_SECRET qua ngan, login se loi.

## 4. Xac thuc va phan quyen

1. Login tao access_token va refresh_token.
2. Access token chua claims gom sub, role, iat, exp.
3. Route user dung middleware auth de verify token hop le.
4. Route admin dung middleware rieng, bat buoc role = admin.
5. User bi disable is_active = false se:
   - Khong login duoc
   - Khong refresh duoc
   - Khi admin disable user, refresh token con hieu luc bi revoke

Cap quyen admin hien tai:
- Chua co API promote role.
- Su dung SQL:
- docker exec -i url_db psql -U user -d shortener_db -c "UPDATE users SET role='admin' WHERE email='admin@example.com';"
- Sau khi doi role, bat buoc login lai de lay token moi.

## 5. Danh sach API

### 5.1 Public

- GET /
  - Mo ta: Kiem tra root endpoint
  - Auth: Khong

- GET /health/live
  - Mo ta: Liveness probe
  - Auth: Khong

- GET /health/ready
  - Mo ta: Readiness probe
  - Auth: Khong

- GET /api-docs/openapi.json
  - Mo ta: Lay OpenAPI JSON
  - Auth: Khong

- GET /docs
  - Mo ta: Trang hint duong dan OpenAPI
  - Auth: Khong

- GET /{short_code}
  - Mo ta: Redirect sang original URL
  - Auth: Khong

### 5.2 Auth

- POST /register
  - Mo ta: Dang ky user
  - Auth: Khong

- POST /login
  - Mo ta: Dang nhap, tra ve access_token va refresh_token
  - Auth: Khong

- POST /refresh
  - Mo ta: Doi access token moi bang refresh token
  - Auth: Khong, gui refresh token trong body

- POST /logout
  - Mo ta: Revoke refresh token
  - Auth: Khong, gui refresh token trong body

### 5.3 User APIs

- GET /users/me
  - Mo ta: Lay profile nguoi dang nhap
  - Auth: Bearer user/admin

- GET /users/{id}
  - Mo ta: Lay user theo id, owner-only tren user route
  - Auth: Bearer user/admin

- POST /links
  - Mo ta: Tao short link
  - Auth: Bearer user/admin

- GET /links/my-links
  - Mo ta: Danh sach link cua user hien tai
  - Auth: Bearer user/admin

- GET /links/analytics?from=YYYY-MM-DD&to=YYYY-MM-DD
  - Mo ta: Thong ke click theo ngay cho links cua user
  - Auth: Bearer user/admin

- DELETE /links/{id}
  - Mo ta: Soft delete link cua owner
  - Auth: Bearer user/admin

### 5.4 Admin APIs

- GET /admin/users
  - Mo ta: Lay tat ca users
  - Auth: Bearer admin

- GET /admin/users/{id}
  - Mo ta: Lay chi tiet 1 user
  - Auth: Bearer admin

- DELETE /admin/users/{id}
  - Mo ta: Soft delete user
  - Auth: Bearer admin

- DELETE /admin/users/{id}/hard
  - Mo ta: Hard delete user
  - Auth: Bearer admin

- GET /admin/links
  - Mo ta: Lay tat ca links
  - Auth: Bearer admin

- DELETE /admin/links/{id}
  - Mo ta: Disable soft delete 1 link
  - Auth: Bearer admin

## 6. Rate limit va CORS

- Rate limit hien tai: in-memory window 60 giay, default 120 request/phut/client key.
- Client key uu tien x-forwarded-for, roi x-real-ip, roi user-agent.
- CORS doc gia tri tu CORS_ALLOWED_ORIGINS, ho tro danh sach origin tach boi dau phay hoac *.

## 7. Test nhanh voi file HTTP

Du an da co file [test_api.http](test_api.http).

Thu tu chay goi y:
1. Register user va admin neu can
2. Login user va admin
3. Dan access/refresh token vao bien trong file
4. Tao link va test user APIs
5. Test admin APIs voi admin token

## 8. Ghi chu

- Neu doi role trong DB ma van bi loi Admin role required, nguyen nhan thuong la token cu chua role user.
- Hay login lai de nhan token moi.
