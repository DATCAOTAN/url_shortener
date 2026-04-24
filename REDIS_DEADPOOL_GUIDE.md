# URL Shortener - Tai Lieu Redis + Deadpool (Rust)

========================================
TAI LIEU REDIS + DEADPOOL (RUST)
========================================

## 1. TONG QUAN

deadpool-redis:
- La connection pool cho Redis.
- Giup tai su dung connection.
- Ho tro async voi Tokio.

Redis:
- In-memory database.
- Thuong dung cho cache, session, rate limit, cooldown.

Trong du an nay, Redis dang duoc dung cho:
- Cache redirect URL.
- Cooldown khi tao link.
- Health check readiness (PING Redis).

========================================
## 2. CAU HINH
========================================

### 2.1 Cargo.toml

```toml
[dependencies]
deadpool-redis = "0.14"
tokio = { version = "1", features = ["full"] }
```

### 2.2 Khoi tao pool

Vi du co ban:

```rust
use deadpool_redis::{Config, Runtime};

let cfg = Config::from_url("redis://127.0.0.1/");
let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
```

Ban hien tai trong project (co cau hinh pool size):

```rust
use deadpool_redis::{Config as RedisConfig, Runtime, PoolConfig};

let redis_url = std::env::var("REDIS_URL")
    .unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
let redis_max = std::env::var("REDIS_POOL_MAX")
    .ok()
    .and_then(|v| v.parse::<usize>().ok())
    .unwrap_or(32);

let mut redis_cfg = RedisConfig::from_url(redis_url);
redis_cfg.pool = Some(PoolConfig::new(redis_max));
let pool = redis_cfg.create_pool(Some(Runtime::Tokio1))?;
```

========================================
## 3. LAY CONNECTION
========================================

```rust
let mut conn = pool.get().await?;
```

Kieu:
- deadpool_redis::Connection

Luu y:
- Luon phai `.await`.
- Luon xu ly `Result` (`?` hoac map_err).

========================================
## 4. IMPORT DUNG
========================================

Nen dung:

```rust
use deadpool_redis::redis;
use deadpool_redis::redis::AsyncCommands;
```

Khong nen import truc tiep crate `redis` rieng neu project dang thong nhat qua `deadpool_redis::redis`.

========================================
## 5. CACH GOI REDIS
========================================

### 5.1 Dung `cmd()`

```rust
let val: String = redis::cmd("GET")
    .arg("key")
    .query_async(&mut conn)
    .await?;
```

### 5.2 Dung `AsyncCommands`

```rust
use deadpool_redis::redis::AsyncCommands;

let val: Option<String> = conn.get("key").await?;
```

========================================
## 6. CAC LENH PHO BIEN
========================================

SET:

```rust
conn.set::<_, _, ()>("key", "value").await?;
```

SET EX (TTL):

```rust
conn.set_ex::<_, _, ()>("key", "value", 60).await?;
```

SETNX:

```rust
let ok: bool = conn.set_nx("key", "value").await?;
```

SET NX EX (atomically):

```rust
let ok: bool = redis::cmd("SET")
    .arg("key")
    .arg("value")
    .arg("EX")
    .arg(5)
    .arg("NX")
    .query_async(&mut conn)
    .await?;
```

GET:

```rust
let val: Option<String> = conn.get("key").await?;
```

DEL:

```rust
conn.del::<_, ()>("key").await?;
```

TTL:

```rust
let ttl: i64 = conn.ttl("key").await?;
```

EXISTS:

```rust
let exists: bool = conn.exists("key").await?;
```

EXPIRE:

```rust
let ok: bool = conn.expire("key", 60).await?;
```

INCR:

```rust
let current: i64 = conn.incr("counter", 1).await?;
```

========================================
## 7. PATTERN QUAN TRONG
========================================

### 7.1 CACHE

```rust
if let Some(val) = conn.get::<_, Option<String>>("url:abc").await? {
    return Ok(val);
}

let data = fetch_from_db();
conn.set_ex::<_, _, ()>("url:abc", &data, 3600).await?;
```

### 7.2 RATE LIMIT / COOLDOWN

Pattern dang dung trong project:

```rust
let cooldown_key = format!("cooldown:user:{}", user_id);
let is_set: bool = redis::cmd("SET")
    .arg(&cooldown_key)
    .arg("1")
    .arg("EX")
    .arg(5)
    .arg("NX")
    .query_async(&mut *conn)
    .await?;

if !is_set {
    return Err(AppError::TooManyRequests(
        "Please wait 5s before creating another link".to_string(),
    ));
}
```

Y nghia:
- Neu key chua ton tai -> set thanh cong, cho qua.
- Neu key da ton tai -> chan request (429).

### 7.3 DISTRIBUTED LOCK

Nguyen tac:
- Dung `SET lock:resource 1 NX EX 10`.
- Luon co TTL de tranh dead lock khi process chet dot ngot.

========================================
## 8. ERROR HANDLING
========================================

Trong project nay, `AppError` da map san:
- deadpool_redis::PoolError
- deadpool_redis::redis::RedisError

Vi vay, nhieu cho co the dung truc tiep:

```rust
let mut conn = state.redis.get().await?;
```

Hoac map_err thu cong neu can log bo sung:

```rust
let mut conn = state.redis.get().await.map_err(|e| {
    tracing::error!("Redis pool error: {:?}", e);
    AppError::ServiceUnavailable("Redis unavailable".into())
})?;
```

========================================
## 9. LOI THUONG GAP
========================================

- Quen `.await` khi goi Redis async.
- Import sai namespace (`use redis;`) gay xung dot version.
- Khong check ket qua `NX` trong cooldown/rate limit.
- Nham kieu ket qua cua `SET NX EX` (co codebase dung bool, co codebase dung Option<String>).
- Truyen sai mutable ref vao `query_async`.

Ghi chu:
- Trong codebase nay, ca `&mut conn` va `&mut *conn` deu co the gap tuy context generic.

========================================
## 10. BEST PRACTICES
========================================

- Luon dung connection pool (khong mo ket noi Redis moi cho moi request).
- Dung TTL cho cache/cooldown de tu dong cleanup.
- Dat key ro rang theo namespace:
  - `url:{short_code}`
  - `session:{token}`
  - `cooldown:user:{user_id}`
- Redis khong phai source of truth.
  - PostgreSQL van la nguon du lieu chinh.
- Co fallback/log ro rang khi Redis su co.

========================================
## 11. TOM TAT
========================================

deadpool:
- Quan ly connection pool.

redis crate (thong qua deadpool_redis::redis):
- Gui command Redis.

pool.get():
- Lay connection async.

query_async / AsyncCommands:
- Thuc thi lenh Redis.

========================================
END
========================================
