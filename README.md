# URL Shortener API

A high-performance RESTful API for URL shortening built with Rust. The system supports user authentication with JWT, Redis-based redirect caching, click analytics tracking, and full admin management — designed for production workloads and containerized deployment via Docker.

---

## Table of Contents

- [Overview](#overview)
- [Technology Stack](#technology-stack)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Database Schema](#database-schema)
- [API Endpoints](#api-endpoints)
- [Key Technical Features](#key-technical-features)
- [Environment Variables](#environment-variables)
- [Getting Started](#getting-started)
- [API Documentation](#api-documentation)

---

## Overview

This project implements a production-ready URL shortening service backend. The core flow converts a long URL into a unique short code using a Base62 encoding algorithm derived from the database sequence ID, ensuring globally unique and collision-free codes. On redirect, the system checks a Redis cache layer first before hitting PostgreSQL, enabling sub-millisecond response times for hot links.

Key capabilities:
- User registration, login, logout, and token refresh
- Short link creation, listing, and soft deletion per user
- Daily click analytics aggregated per user
- Admin panel for managing all users and links
- Rate limiting per IP to prevent abuse
- Liveness and readiness health check endpoints for container orchestration

---

## Technology Stack

| Component       | Technology                              |
|-----------------|-----------------------------------------|
| Language        | Rust (Edition 2021)                     |
| Web Framework   | Axum 0.8                                |
| Async Runtime   | Tokio                                   |
| Database        | PostgreSQL 15 via SQLx 0.8              |
| Cache Layer     | Redis 7 via deadpool-redis              |
| Authentication  | JWT (HS256) — jsonwebtoken crate        |
| Password Hashing| bcrypt (DEFAULT_COST)                   |
| Migrations      | sqlx-cli                                |
| API Docs        | utoipa (OpenAPI 3 / Swagger UI)         |
| Containerization| Docker + Docker Compose                 |
| Logging/Tracing | tracing + tracing-subscriber            |

---

## Architecture

The project follows a layered architecture pattern, separating concerns across distinct modules:

```
HTTP Request
     |
     v
[Rate Limit Middleware]  -- per-IP sliding window counter (in-memory)
     |
     v
[Auth / Admin Middleware] -- JWT Bearer token validation, role check
     |
     v
[Handlers]               -- request parsing, input validation, response shaping
     |
     v
[Services]               -- business logic (Base62 encoding, bcrypt, token management)
     |
     v
[Repositories]           -- compile-time verified SQL queries via sqlx::query_as!
     |
     v
[PostgreSQL] <---[Redis Cache]--- redirect_link handler (cache-aside pattern)
```

**Cache-Aside Strategy**: On redirect, the handler checks Redis first. On a cache miss, it queries PostgreSQL and writes the result back to Redis with a 1-hour TTL. On soft-delete, the cache entry is invalidated immediately.

**Async Click Tracking**: After a redirect, click count and daily analytics are updated via `tokio::spawn`, ensuring the redirect response is returned without waiting for the analytics write to complete.

---

## Project Structure

```
url_shortener/
├── src/
│   ├── main.rs                     # Entry point: router assembly, middleware stack, server binding
│   ├── state.rs                    # AppState: shared PgPool + Redis Pool
│   ├── db.rs                       # PostgreSQL connection pool initialization
│   ├── error.rs                    # AppError enum + IntoResponse impl (unified error handling)
│   ├── docs.rs                     # utoipa OpenAPI schema + Swagger UI page
│   ├── dtos/
│   │   ├── claims.rs               # JWT Claims struct (sub, role, iat, exp, jti)
│   │   ├── user.rs                 # RegisterUser, LoginUser, LoginResponse, UserResponse, etc.
│   │   └── link.rs                 # CreateLinkRequest, LinkResponse, DailyAnalyticsResponse, etc.
│   ├── models/
│   │   ├── user.rs                 # User DB model (FromRow)
│   │   ├── link.rs                 # Link DB model (FromRow)
│   │   ├── link_analytics.rs       # LinkAnalytics, DailyClickTotal DB models
│   │   └── refresh_tokens.rs       # RefreshToken DB model
│   ├── repositories/
│   │   ├── user_repository.rs      # All user SQL: find, register, soft/hard delete, refresh tokens
│   │   └── link_repository.rs      # All link SQL: create, find, soft delete, analytics queries
│   ├── services/
│   │   ├── user_service.rs         # Business logic: register, login, refresh, logout (bcrypt + JWT)
│   │   ├── link_service.rs         # Business logic: Base62 encoding, create/get/delete, analytics
│   │   └── cache_service.rs        # Redis get/set/invalidate with key prefix "url:"
│   ├── handlers/
│   │   ├── user_handler.rs         # HTTP handlers for auth endpoints
│   │   ├── link_handler.rs         # HTTP handlers for link CRUD and redirect
│   │   ├── admin_handler.rs        # HTTP handlers for admin user/link management
│   │   └── health_handler.rs       # Liveness and readiness probes
│   ├── middleware/
│   │   ├── auth_middleware.rs      # Bearer JWT validation, injects Claims into extensions
│   │   ├── admin_middleware.rs     # Bearer JWT validation + role == "admin" check
│   │   └── rate_limit_middleware.rs# Sliding window rate limiter (IP-based, configurable)
│   ├── routes/
│   │   ├── user_route.rs           # Public: /register, /login, /refresh, /logout; Protected: /users/me
│   │   ├── link_route.rs           # Public: /{short_code}; Protected: /links/*
│   │   ├── admin_route.rs          # Admin-only: /admin/users/*, /admin/links/*
│   │   └── health_route.rs         # /health/live, /health/ready
│   └── utils/
│       ├── jwt.rs                  # encode_access_token, encode_refresh_token, decode_jwt
│       └── validation.rs           # validate_email, validate_password, validate_url, validate_title
├── migrations/
│   ├── 20260207014058_init_db.up.sql            # Create users, links, link_analytics tables
│   ├── 20260207014823_add_refresh_tokens.up.sql # Create refresh_tokens table
│   ├── 20260326000000_extend_short_code.up.sql  # Extend short_code column to VARCHAR(16)
│   ├── 20260327000000_unique_owner_url.up.sql   # Add unique index on (owner_id, original_url)
│   └── 20260327001000_unique_owner_url_active.up.sql # Partial unique index for active links only
├── scripts/
│   ├── seed.sql                    # Sample data for development
│   ├── run_api_batch_curl.ps1      # PowerShell batch test script
│   ├── run_api_phase1_prepare_data.ps1
│   └── run_api_phase2_redirect_delete.ps1
├── Dockerfile                      # Multi-stage build with cargo-watch and sqlx-cli
├── docker-compose.yml              # Orchestrates: PostgreSQL 15, Redis 7, Rust API
├── test_api.http                   # HTTP client test file (VS Code REST Client)
└── Cargo.toml                      # Dependencies declaration
```

---

## Database Schema

### Table: users

| Column          | Type           | Description                                      |
|-----------------|----------------|--------------------------------------------------|
| id              | BIGSERIAL PK   | Auto-incrementing primary key                    |
| username        | VARCHAR(50)    | Unique, not null                                 |
| password_hash   | VARCHAR(255)   | bcrypt hashed password                           |
| email           | VARCHAR(100)   | Unique, nullable                                 |
| role            | VARCHAR(20)    | Default: 'user'; can be 'admin'                  |
| is_active       | BOOLEAN        | Soft delete flag, default TRUE                   |
| created_at      | TIMESTAMPTZ    | Record creation timestamp                        |
| updated_at      | TIMESTAMPTZ    | Record last update timestamp                     |

### Table: links

| Column        | Type           | Description                                        |
|---------------|----------------|----------------------------------------------------|
| id            | BIGSERIAL PK   | Used as input to Base62 encoder for short_code     |
| owner_id      | BIGINT FK      | References users(id), ON DELETE SET NULL           |
| original_url  | TEXT           | The full original URL                              |
| short_code    | VARCHAR(16)    | Base62-encoded unique identifier                   |
| title         | VARCHAR(255)   | Optional user-supplied label                       |
| click_count   | BIGINT         | Cumulative redirect count, default 0               |
| is_active     | BOOLEAN        | Soft delete flag, default TRUE                     |
| created_at    | TIMESTAMPTZ    |                                                    |
| updated_at    | TIMESTAMPTZ    |                                                    |

Indexes: `idx_links_short_code` on `short_code`; partial unique index on `(owner_id, original_url)` WHERE active.

### Table: link_analytics

| Column   | Type        | Description                                 |
|----------|-------------|---------------------------------------------|
| id       | BIGSERIAL PK|                                             |
| link_id  | BIGINT FK   | References links(id), ON DELETE CASCADE     |
| date     | DATE        | The date of click aggregation (VN timezone) |
| clicks   | INT         | Click count for the day, upserted atomically|

Unique constraint on `(link_id, date)`.

### Table: refresh_tokens

| Column       | Type           | Description                                   |
|--------------|----------------|-----------------------------------------------|
| id           | BIGSERIAL PK   |                                               |
| user_id      | BIGINT FK      | References users(id), ON DELETE CASCADE       |
| token_hash   | VARCHAR(255)   | SHA-256 hash of the refresh token JWT         |
| user_agent   | TEXT           | Optional metadata                             |
| ip_address   | VARCHAR(45)    | Optional metadata                             |
| expires_at   | TIMESTAMPTZ    | Expiry time (30 days default)                 |
| revoked_at   | TIMESTAMPTZ    | Set on logout or admin user disable           |
| created_at   | TIMESTAMPTZ    |                                               |

Index on `token_hash` for fast lookup during refresh and logout.

---

## API Endpoints

### Public Endpoints (no authentication)

| Method | Path                    | Description                              |
|--------|-------------------------|------------------------------------------|
| GET    | /                       | Health check text response               |
| GET    | /health/live            | Liveness probe                           |
| GET    | /health/ready           | Readiness probe (checks DB + Redis)      |
| GET    | /docs                   | Swagger UI (interactive documentation)   |
| GET    | /api-docs/openapi.json  | Raw OpenAPI 3.0 JSON schema              |
| GET    | /{short_code}           | Redirect to original URL (307)           |
| POST   | /register               | Register a new user account              |
| POST   | /login                  | Login, returns access_token + refresh_token |
| POST   | /refresh                | Exchange refresh token for new access token |
| POST   | /logout                 | Revoke refresh token (invalidate session)|

### Protected Endpoints (Bearer access token required)

| Method | Path                    | Description                              |
|--------|-------------------------|------------------------------------------|
| GET    | /users/me               | Get current authenticated user's profile |
| POST   | /links                  | Create a new short link                  |
| GET    | /links/my-links         | List all links owned by the current user |
| DELETE | /links/{id}             | Soft delete a link (owner only)          |
| GET    | /links/analytics        | Daily click totals for a date range      |

Query parameters for analytics: `?from=YYYY-MM-DD&to=YYYY-MM-DD`

### Admin Endpoints (Bearer token with role "admin" required)

| Method | Path                    | Description                              |
|--------|-------------------------|------------------------------------------|
| GET    | /admin/users            | List all registered users                |
| GET    | /admin/users/{id}       | Get a specific user by ID                |
| DELETE | /admin/users/{id}       | Soft disable a user (revokes all tokens) |
| DELETE | /admin/users/{id}/hard  | Permanently delete a user                |
| GET    | /admin/links            | List all links in the system             |
| DELETE | /admin/links/{id}       | Soft disable a link + invalidate cache   |

---

## Key Technical Features

### 1. Base62 Short Code Generation

Short codes are generated deterministically from the PostgreSQL sequence value of the link ID. The sequence ID is encoded in Base62 (digits + uppercase + lowercase), producing compact and URL-safe codes without requiring a separate random generator or collision detection loop.

```rust
const BASE62_CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ...";

fn encode_base62(mut num: i64) -> String {
    // Converts the sequence ID to a Base62 string
}
```

Idempotency: If a user submits the same URL again, the service returns the existing active link instead of creating a duplicate (enforced by a partial unique index on `(owner_id, original_url)` WHERE active).

### 2. JWT Authentication with Dual-Token Strategy

- **Access token**: Short-lived (default 15 minutes), signed with `JWT_SECRET`, carries `sub` (user ID) and `role`.
- **Refresh token**: Long-lived (default 30 days), signed with `JWT_REFRESH_SECRET`, includes a `jti` (UUID v4) for uniqueness.
- **Refresh token hashing**: The raw JWT string is never stored in the database. It is hashed with SHA-256 before persistence. On logout or user disable, the `revoked_at` field is set, invalidating the token immediately.

### 3. Redis Cache-Aside for Redirects

Redirects are the most latency-sensitive operation. The cache layer:
1. Checks `url:{short_code}` key in Redis.
2. On hit: returns the URL immediately without touching PostgreSQL.
3. On miss: queries PostgreSQL, writes the result to Redis with a 1-hour TTL, then redirects.
4. On soft delete: calls `invalidate_cache` to delete the Redis key proactively.

Cache errors are handled gracefully via `tracing::warn!` — a Redis failure does not break the redirect flow, it falls back to the database.

### 4. Non-Blocking Async Analytics

Click counting and daily analytics aggregation are decoupled from the redirect response using `tokio::spawn`. The redirect HTTP response is returned immediately to the client while the database writes happen concurrently in the background. The analytics upsert uses `ON CONFLICT (link_id, date) DO UPDATE` to atomically increment the click count.

### 5. Role-Based Access Control (RBAC)

Two middleware layers enforce authorization:
- `auth_middleware`: Validates any valid JWT, injects `Claims` into request extensions for handler access.
- `admin_middleware`: Validates JWT and additionally checks `claims.role == "admin"`. Rejects with 403 Forbidden if the role is insufficient.

An admin cannot delete or disable their own account (enforced with an explicit `admin_id == user_id` check).

### 6. IP-Based Rate Limiting

A sliding window rate limiter is implemented in pure async Rust using `tokio::Mutex` and `std::time::Instant`. It is applied globally to all routes as a middleware layer. The client key is resolved from `X-Forwarded-For`, `X-Real-IP`, or `User-Agent` headers in that order. The default limit is 120 requests per minute and is configurable via environment variable.

### 7. Compile-Time Verified SQL

All database queries use the `sqlx::query_as!` and `sqlx::query!` macros with compile-time verification (offline mode with `.sqlx` cache). This catches SQL type mismatches and typos at compile time rather than at runtime.

### 8. Unified Error Handling

A custom `AppError` enum implements `IntoResponse` for Axum, mapping each error variant to the appropriate HTTP status code and a consistent JSON body: `{ "error": "...", "status": 4xx }`. The `AppResult<T>` type alias is used as the return type for all handlers and service functions.

### 9. Health Check Endpoints

- `/health/live`: Always returns 200 — used by container orchestrators to detect process crashes.
- `/health/ready`: Executes `SELECT 1` against PostgreSQL and `PING` against Redis, returning 500 if either dependency is unavailable. Used by load balancers to prevent traffic from reaching an unready instance.

### 10. Docker Deployment

The `docker-compose.yml` orchestrates three services:
- **db**: PostgreSQL 15 (Alpine) with a health check that gates the app startup.
- **redis**: Redis 7 (Alpine) with a persistent volume.
- **app**: The Rust API, configured with `depends_on` (health condition) so it only starts after the database is ready. On startup, it runs `sqlx migrate run` automatically before launching the server.

---

## Environment Variables

| Variable                        | Default                        | Description                                  |
|---------------------------------|--------------------------------|----------------------------------------------|
| `DATABASE_URL`                  | (required)                     | PostgreSQL connection string                 |
| `REDIS_URL`                     | `redis://127.0.0.1/`           | Redis connection URL                         |
| `REDIS_POOL_MAX`                | `32`                           | Maximum Redis connection pool size           |
| `JWT_SECRET`                    | (required, min 32 chars)       | HMAC secret for access token signing         |
| `JWT_REFRESH_SECRET`            | (required, min 32 chars)       | HMAC secret for refresh token signing        |
| `ACCESS_TOKEN_EXPIRE`           | `900` (15 minutes)             | Access token TTL in seconds                  |
| `REFRESH_TOKEN_EXPIRE`          | `2592000` (30 days)            | Refresh token TTL in seconds                 |
| `BIND_ADDR`                     | `0.0.0.0:8080`                 | Server bind address                          |
| `DB_MAX_CONNECTIONS`            | `50`                           | PostgreSQL connection pool max size          |
| `DB_MIN_CONNECTIONS`            | `5`                            | PostgreSQL connection pool min size          |
| `CORS_ALLOWED_ORIGINS`          | `http://localhost:3000,...`    | Comma-separated allowed origins (or `*`)     |
| `RATE_LIMIT_REQUESTS_PER_MINUTE`| `120`                          | Rate limit threshold per client per minute   |

---

## Getting Started

### Prerequisites

- Docker and Docker Compose installed

### Run with Docker Compose

1. Clone the repository:
   ```bash
   git clone https://github.com/DATCAOTAN/url_shortener.git
   cd url_shortener
   ```

2. Copy and configure the environment (or rely on defaults in `docker-compose.yml`):
   ```bash
   # Edit JWT secrets in docker-compose.yml before running in production
   ```

3. Start all services:
   ```bash
   docker compose up --build
   ```
   The API will be available at `http://localhost:8080`.
   Database migrations run automatically on container startup.

### Run Locally (without Docker)

1. Ensure PostgreSQL and Redis are running locally.

2. Create a `.env` file:
   ```env
   DATABASE_URL=postgresql://user:password@localhost:5432/shortener_db
   REDIS_URL=redis://127.0.0.1/
   JWT_SECRET=your_access_secret_at_least_32_characters
   JWT_REFRESH_SECRET=your_refresh_secret_at_least_32_characters
   ```

3. Run migrations:
   ```bash
   sqlx migrate run
   ```

4. Start the server:
   ```bash
   cargo run
   ```

---

## API Documentation

An interactive Swagger UI is available at `http://localhost:8080/docs` after starting the server. The raw OpenAPI 3.0 JSON schema is available at `http://localhost:8080/api-docs/openapi.json`.

The `test_api.http` file in the repository root contains ready-to-use request examples for all endpoints, compatible with the VS Code REST Client extension.
