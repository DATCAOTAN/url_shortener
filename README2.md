# Huong dan A-Z tao API hoan chinh theo kien truc du an nay

Tai lieu nay huong dan cach xay mot API endpoint moi tu dau den cuoi theo dung cau truc hien tai cua project.

## 1) Kien truc tong the

Project dung mo hinh layer ro rang:

- `routes`: khai bao endpoint, method, middleware.
- `handlers`: nhan request HTTP, parse DTO, goi service, tra response.
- `services`: business logic, validation, auth, token, xu ly nghiep vu.
- `repositories`: truy van DB bang `sqlx`.
- `models`: map bang DB.
- `dtos`: request/response struct cho API.
- `middleware`: cross-cutting logic nhu auth guard.
- `utils`: helper dung chung (vi du JWT verify).
- `error`: app error tap trung, tra loi JSON thong nhat.

Luong request:

`Client -> Route -> Middleware (neu co) -> Handler -> Service -> Repository -> DB -> Service -> Handler -> Client`

## 2) Chuan bi moi truong

File `.env` toi thieu:

```env
DATABASE_URL=postgresql://user:password@localhost:5433/shortener_db
JWT_SECRET=your_access_secret
JWT_REFRESH_SECRET=your_refresh_secret
ACCESS_TOKEN_EXPIRE=900
REFRESH_TOKEN_EXPIRE=2592000
```

Y nghia:

- `ACCESS_TOKEN_EXPIRE`: thoi gian song access token (giay), mac dinh 900.
- `REFRESH_TOKEN_EXPIRE`: thoi gian song refresh token (giay), mac dinh 30 ngay.

Chay app:

```bash
docker-compose up -d
cargo run
```

## 2.1) Chay full bang Docker Compose (khuyen dung)

Project da duoc set de chay ca `app + postgres + redis` trong Docker.

Lenh chay:

```bash
docker compose up --build -d
```

Kiem tra trang thai:

```bash
docker compose ps
```

API se san sang tai:

```text
http://127.0.0.1:8080
```

Kiem tra nhanh:

```bash
curl http://127.0.0.1:8080/
```

Doc log app:

```bash
docker compose logs -f app
```

Dung va xoa container:

```bash
docker compose down
```

Xoa ca volume (clean DB):

```bash
docker compose down -v
```

Ghi chu quan trong:

- Trong container, `DATABASE_URL` phai tro vao host service `db:5432` (khong dung localhost).
- Luc startup, service `app` tu chay migration voi `sqlx migrate run` roi moi `cargo run`.

## 3) Quy trinh tao mot API moi (template chuan)

Vi du: tao API `GET /users/me` (protected).

### Buoc A - Tao/sua DTO

Neu endpoint can request/response rieng, tao struct trong `src/dtos/...`.

Nguyen tac:

- Khong dung truc tiep model DB lam response neu co field nhay cam.
- Tach request DTO va response DTO rieng.

### Buoc B - Them ham Repository

Trong `src/repositories/...`:

- Viet query DB chuan SQLx.
- Ham repository chi lam viec voi data access, khong chua business rule phuc tap.

### Buoc C - Them ham Service

Trong `src/services/...`:

- Goi repository.
- Dat validation/business rule.
- Mapping loi noi bo thanh AppError phu hop.

### Buoc D - Them Handler

Trong `src/handlers/...`:

- Nhan `State<PgPool>` va DTO input.
- Goi service.
- Tra `AppResult<Json<T>>`.

### Buoc E - Wire route

Trong `src/routes/...`:

- Public route: merge vao `public_routes`.
- Protected route: dat trong `protected_routes` va gan auth middleware.

## 4) Auth va Protected Route trong project hien tai

### Public routes

- `POST /register`
- `POST /login`
- `POST /refresh`

### Protected routes

- `GET /users/me`
- `GET /users/{id}`

Protected routes duoc bao ve boi middleware `auth_guard`:

1. Doc header `Authorization`.
2. Kiem tra format `Bearer <token>`.
3. Verify access token qua `utils::jwt::verify_jwt`.
4. Chen `Claims` vao request extensions.
5. Cho request di tiep vao handler.

Trong handler protected, lay user tu:

- `Extension<Claims>` de lay `claims.id`.

## 5) Luong Login va Refresh token

### Login (`POST /login`)

1. Tim user theo email.
2. Verify password bang bcrypt.
3. Tao access token bang `JWT_SECRET`.
4. Tao refresh token bang `JWT_REFRESH_SECRET`.
5. Luu refresh token vao bang `refresh_tokens`.
6. Tra ve JSON:

```json
{
  "access_token": "...",
  "refresh_token": "..."
}
```

### Refresh (`POST /refresh`)

1. Nhan `refresh_token` tu body.
2. Decode/verify bang `JWT_REFRESH_SECRET`.
3. Kiem tra token con active trong DB:
   - ton tai,
   - `revoked_at IS NULL`,
   - `expires_at > NOW()`.
4. Tao access token moi bang `JWT_SECRET`.
5. Tra ve:

```json
{
  "access_token": "..."
}
```

Ghi chu:

- Access token khong luu DB (stateless), client se giu.
- Refresh token dang duoc luu DB de co the revoke va kiem soat session.

## 6) Error handling chuan trong project

Project da co `AppError` tap trung trong `src/error.rs`.

Muc tieu:

- Tra loi loi JSON thong nhat.
- Giam lap code `StatusCode + String` o tung handler.
- De mo rong logging, tracing va mapping loi.

Format loi JSON hien tai:

```json
{
  "error": "...",
  "status": 401
}
```

## 7) Checklist de tao endpoint moi khong bi thieu

1. Co DTO request/response chua?
2. Co repository query chua?
3. Service da xu ly business + mapping loi chua?
4. Handler da dung `AppResult` chua?
5. Route da duoc wire dung public/protected chua?
6. Neu protected: da gan middleware auth chua?
7. Da test success case + fail case chua?

## 8) Cach test API nhanh

Thu tu test auth:

1. `POST /register`
2. `POST /login` lay access + refresh token
3. `GET /users/me` voi `Authorization: Bearer <access_token>`
4. `POST /refresh` voi refresh token
5. Goi lai `GET /users/me` voi access token moi

Neu test bang script/REST client, co the luu token tam vao bien runtime.

## 9) Khuyen nghi de API san sang production

1. Khong tra truc tiep model co field nhay cam (password_hash).
2. Hash refresh token truoc khi luu DB.
3. Them logout/revoke endpoint.
4. Lam refresh-token rotation (moi lan refresh, cap token moi va revoke token cu).
5. Them request id + structured logging.
6. Them integration test cho auth flow.

---

Neu can, buoc tiep theo nen lam la: them endpoint logout + revoke refresh token de hoan tat vong doi session.
