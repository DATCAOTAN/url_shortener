# URL Shortener API

README này mô tả luong hoat dong chinh cua app hien tai: dang ky, dang nhap, cap lai access token bang refresh token, va lay thong tin user.

## Tong quan kien truc

Ung dung dung mo hinh theo tang:

- `routes`: dinh nghia endpoint HTTP
- `handlers`: nhan request/tra response HTTP
- `services`: business logic (hash password, tao/verify JWT)
- `repositories`: truy cap PostgreSQL bang `sqlx`
- `models`: struct map voi bang DB
- `dtos`: request/response object cua API

Duong di request:

`Client -> Route -> Handler -> Service -> Repository -> PostgreSQL -> Service -> Handler -> Client`

## Bien moi truong

File `.env` can co toi thieu:

```env
DATABASE_URL=postgresql://user:password@localhost:5433/shortener_db
JWT_SECRET=your_access_secret
JWT_REFRESH_SECRET=your_refresh_secret
ACCESS_TOKEN_EXPIRE=900
REFRESH_TOKEN_EXPIRE=2592000
```

Y nghia:

- `ACCESS_TOKEN_EXPIRE`: so giay song cua access token (mac dinh 900s)
- `REFRESH_TOKEN_EXPIRE`: so giay song cua refresh token (mac dinh 30 ngay)

## Endpoints hien tai

- `GET /` health check co ban
- `GET /users/{id}` lay user theo id
- `POST /register` dang ky user moi
- `POST /login` dang nhap, tra `access_token` va `refresh_token`
- `POST /refresh` dung refresh token de cap lai access token

## Luong dang ky (`POST /register`)

1. Handler nhan `RegisterUser { username, email, password }`.
2. Service hash password bang `bcrypt`.
3. Repository insert vao bang `users`.
4. Handler tra ve user vua tao.

## Luong dang nhap (`POST /login`)

1. Handler nhan `LoginUser { email, password }`.
2. Service tim user theo email.
3. Service verify password bang `bcrypt::verify`.
4. Neu hop le, service tao:
   - `access_token` bang `JWT_SECRET`
   - `refresh_token` bang `JWT_REFRESH_SECRET`
5. Service luu refresh token vao bang `refresh_tokens`.
6. Handler tra ve:

```json
{
  "access_token": "...",
  "refresh_token": "..."
}
```

## Luong refresh (`POST /refresh`)

1. Handler nhan `RefreshTokenRequest { refresh_token }`.
2. Service decode/verify refresh token bang `JWT_REFRESH_SECRET`.
3. Service kiem tra token con active trong DB:
   - ton tai trong bang `refresh_tokens`
   - `revoked_at IS NULL`
   - `expires_at > NOW()`
4. Neu hop le, service tao access token moi bang `JWT_SECRET`.
5. Handler tra ve:

```json
{
  "access_token": "..."
}
```

## Ma loi auth hien tai

- Sai email: `401` - `Thong tin dang nhap sai`
- Sai password: `401` - `Password sai`
- Refresh token khong hop le: `401` - `Refresh token khong hop le`

## Chay du an

1. Khoi dong database:

```bash
docker-compose up -d
```

2. Chay app:

```bash
cargo run
```

Neu gap `PoolTimedOut`, hay kiem tra PostgreSQL da chay va cong `5433` dang mo.

## Vi du request nhanh

Dang ky:

```http
POST /register
Content-Type: application/json

{
  "username": "alice",
  "email": "alice@example.com",
  "password": "12345678"
}
```

Dang nhap:

```http
POST /login
Content-Type: application/json

{
  "email": "alice@example.com",
  "password": "12345678"
}
```

Refresh:

```http
POST /refresh
Content-Type: application/json

{
  "refresh_token": "<refresh-token-tu-login>"
}
```

## Ghi chu bao mat

- Nen tra ve `UserResponse` thay vi tra truc tiep model `User` de tranh lo `password_hash`.
- Nen hash refresh token truoc khi luu DB.
- Nen them endpoint logout/revoke refresh token.
