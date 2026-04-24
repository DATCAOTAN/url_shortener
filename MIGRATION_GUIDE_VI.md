# Migration Guide (VI) - URL Shortener

Tai lieu nay huong dan migration cho du an URL Shortener (PostgreSQL + SQLx), dua theo convention dang co trong thu muc migrations.

## 0. Can cai dat gi truoc khi chay migration
sudo apt install -y pkg-config libssl-dev

### 0.1 Bat buoc

- Docker + Docker Compose
- Source code day du (co thu muc migrations)

### 0.2 Neu chay migration tren may host (khong qua container app)

- Rust toolchain (cargo)
- sqlx-cli:
  - cargo install sqlx-cli --no-default-features --features postgres
- Bien moi truong DATABASE_URL, vi du:
  - export DATABASE_URL=postgresql://user:password@localhost:5433/shortener_db

### 0.3 Plugin/extension nen co (khong bat buoc)

- VS Code SQLTools
- VS Code PostgreSQL (ms-ossdata.vscode-postgresql)

Muc dich: de xem schema, test query, va kiem tra nhanh sau migration.

## 1. Convention dang dung trong repo

- Dat ten file: YYYYMMDDHHMMSS_ten_migration.up.sql va YYYYMMDDHHMMSS_ten_migration.down.sql
- Moi migration phai co du cap up/down de rollback an toan.
- Uu tien dung IF EXISTS, IF NOT EXISTS cho cac thay doi co the chay lai.
- Neu co rang buoc phu thuoc, rollback theo thu tu nguoc lai (drop con truoc, cha sau).

Tham khao migration hien co:
- migrations/20260207014058_init_db.up.sql
- migrations/20260207014823_add_refresh_tokens.up.sql
- migrations/20260326000000_extend_short_code.up.sql
- migrations/20260327001000_unique_owner_url_active.up.sql
- migrations/20260422010000_add_link_expires_at.up.sql

## 2. Quy trinh tao migration moi

### Cach 1: Tao file thu cong

1. Tao 2 file moi trong thu muc migrations:
- 20260424090000_example_change.up.sql
- 20260424090000_example_change.down.sql

2. Viet SQL cho up va down.

3. Chay migration:
- sqlx migrate run

4. Khi can rollback 1 buoc:
- sqlx migrate revert

### Cach 2: Neu ban co SQLx CLI helper

- sqlx migrate add ten_migration

Lenh tren se tao cap file up/down, sau do ban dien noi dung SQL.

## 3. Cap nhat 1 bang (table-level change)

### 3.1 Doi ten bang

Up:
```sql
ALTER TABLE users RENAME TO app_users;
```

Down:
```sql
ALTER TABLE app_users RENAME TO users;
```

### 3.2 Them cot moi vao bang

Up:
```sql
ALTER TABLE links
ADD COLUMN IF NOT EXISTS expires_at BIGINT;

CREATE INDEX IF NOT EXISTS idx_links_expires_at ON links(expires_at);
```

Down:
```sql
DROP INDEX IF EXISTS idx_links_expires_at;

ALTER TABLE links
DROP COLUMN IF EXISTS expires_at;
```

Mau nay giong migration dang co:
- migrations/20260422010000_add_link_expires_at.up.sql
- migrations/20260422010000_add_link_expires_at.down.sql

## 4. Them mot truong (column) moi

Vi du them cot status vao bang links.

Up:
```sql
ALTER TABLE links
ADD COLUMN IF NOT EXISTS status VARCHAR(20) NOT NULL DEFAULT 'active';

CREATE INDEX IF NOT EXISTS idx_links_status ON links(status);
```

Down:
```sql
DROP INDEX IF EXISTS idx_links_status;

ALTER TABLE links
DROP COLUMN IF EXISTS status;
```

Luu y:
- Neu bang lon, them cot NOT NULL can can nhac lock/thoi gian chay.
- Co the them nullable truoc, backfill, roi ALTER thanh NOT NULL.

## 5. Cap nhat 1 truong

Co 2 kieu cap nhat thuong gap:

### 5.1 Cap nhat schema cua truong (doi type/do dai)

Vi du doi do dai short_code tu 10 -> 16 (mau dang co trong repo).

Up:
```sql
ALTER TABLE links
ALTER COLUMN short_code TYPE VARCHAR(16);
```

Down:
```sql
ALTER TABLE links
ALTER COLUMN short_code TYPE VARCHAR(10);
```

Tham khao:
- migrations/20260326000000_extend_short_code.up.sql
- migrations/20260326000000_extend_short_code.down.sql

### 5.2 Cap nhat du lieu cua truong (UPDATE data)

Vi du chuan hoa role user null thanh user.

Up:
```sql
UPDATE users
SET role = 'user'
WHERE role IS NULL;
```

Down:
```sql
-- Khong phuc hoi chinh xac duoc gia tri cu, thuong de no-op hoac note ro.
-- SELECT 1;
```

Luu y:
- Migration du lieu can ghi ro tinh chat irreversable neu down khong phuc hoi duoc.
- Neu co the, tao bang backup tam hoac audit truoc khi UPDATE lon.

## 6. Khoa chinh (Primary Key)

### 6.1 Tao PK khi tao bang

```sql
CREATE TABLE example_items (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL
);
```

### 6.2 Them PK cho bang da ton tai

Up:
```sql
ALTER TABLE example_items
ADD CONSTRAINT pk_example_items PRIMARY KEY (id);
```

Down:
```sql
ALTER TABLE example_items
DROP CONSTRAINT IF EXISTS pk_example_items;
```

Luu y:
- Cot PK phai unique va NOT NULL truoc khi ADD PRIMARY KEY.

## 7. Khoa ngoai (Foreign Key)

### 7.1 Tao FK luc tao bang

Mau dang co trong repo:
```sql
CREATE TABLE refresh_tokens (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL
);
```

### 7.2 Them FK cho bang da ton tai

Up:
```sql
ALTER TABLE links
ADD CONSTRAINT fk_links_owner
FOREIGN KEY (owner_id)
REFERENCES users(id)
ON DELETE SET NULL;
```

Down:
```sql
ALTER TABLE links
DROP CONSTRAINT IF EXISTS fk_links_owner;
```

Luu y:
- ON DELETE CASCADE: xoa cha se xoa con.
- ON DELETE SET NULL: xoa cha se set cot FK ve NULL (cot FK phai cho phep NULL).
- Truoc khi add FK, du lieu hien tai phai khong vi pham rang buoc.

## 8. Unique va partial index (mau nang cao dang dung)

Repo dang dung partial unique index de tranh trung URL theo owner voi dieu kien active:

Up:
```sql
DROP INDEX IF EXISTS idx_links_owner_url;
CREATE UNIQUE INDEX IF NOT EXISTS idx_links_owner_url_active
ON links (owner_id, original_url)
WHERE owner_id IS NOT NULL AND (is_active IS NULL OR is_active = TRUE);
```

Down:
```sql
DROP INDEX IF EXISTS idx_links_owner_url_active;
CREATE UNIQUE INDEX IF NOT EXISTS idx_links_owner_url
ON links (owner_id, original_url)
WHERE owner_id IS NOT NULL;
```

Tham khao:
- migrations/20260327001000_unique_owner_url_active.up.sql
- migrations/20260327001000_unique_owner_url_active.down.sql

## 9. Checklist truoc khi merge migration

- Co du cap file up/down.
- Chay duoc tren DB moi (fresh) va DB da co du lieu.
- Down co the rollback toi thieu 1 buoc ma khong vo schema.
- Lenh CREATE INDEX lon can danh gia thoi gian downtime.
- Neu migration data, note ro down co reversable hay khong.

## 10. Lenh hay dung khi chay local Docker

- Chay app va DB:
  - docker compose up --build

- Kiem tra truoc khi chay migration:
  - docker compose ps
  - docker compose exec app sh -lc "which sqlx && sqlx --version"
  - docker compose exec app sh -lc "echo $DATABASE_URL"

- Chay migration tren DB Docker qua service app (khuyen nghi):
  - docker compose exec app sh -lc "sqlx migrate run"
  - docker compose exec app sh -lc "sqlx migrate revert"
  - docker compose exec app sh -lc "sqlx migrate info"

- Neu app chua chay, dung one-off container:
  - docker compose run --rm app sh -lc "sqlx migrate run"
  - docker compose run --rm app sh -lc "sqlx migrate revert"

- Chay SQL thu cong truc tiep vao container PostgreSQL (service db):
  - docker compose exec db psql -U user -d shortener_db -c "SELECT NOW();"
  - docker compose exec -T db psql -U user -d shortener_db < migrations/20260422010000_add_link_expires_at.up.sql

- Nap/cap nhat du lieu tu file scripts/seed.sql:
  - docker compose exec -T db psql -U user -d shortener_db < scripts/seed.sql
  - docker compose exec db psql -U user -d shortener_db -c "SELECT COUNT(*) AS users_count FROM users;"
  - docker compose exec db psql -U user -d shortener_db -c "SELECT COUNT(*) AS links_count FROM links;"
  - docker compose exec db psql -U user -d shortener_db -c "SELECT COUNT(*) AS analytics_count FROM link_analytics;"

Luu y:
- Trong docker-compose.yml, app da auto chay migration khi khoi dong qua command: sqlx migrate run && cargo watch -x run.
- Vi vay khi can chay bo sung migration moi, co the restart app hoac dung lenh exec/run ben tren de chay chu dong.
- scripts/seed.sql co TRUNCATE ... RESTART IDENTITY CASCADE, tuc la xoa du lieu cu truoc khi nap lai. Chi chay tren moi truong dev/test.

## 11. Troubleshooting nhanh

### Loi 127 khi chay: docker compose exec app sh -lc "sqlx migrate run"

Y nghia: shell khong tim thay lenh `sqlx` trong container app.

Xu ly theo thu tu:

1. Kiem tra binary co trong container khong:
  - docker compose exec app sh -lc "which sqlx && sqlx --version"

2. Neu khong co, rebuild image khong dung cache:
  - docker compose down
  - docker compose build --no-cache app
  - docker compose up -d

3. Chay lai migration:
  - docker compose exec app sh -lc "sqlx migrate run"

4. Neu van loi, chay migration bang SQL file truc tiep vao DB:
  - docker compose exec -T db psql -U user -d shortener_db < migrations/20260422010000_add_link_expires_at.up.sql

Ghi chu: Dockerfile cua repo da co lenh cai sqlx-cli, nen truong hop loi 127 thuong do image cu chua rebuild.

Neu ban muon, co the bo sung phan "mau migration cho them role moi" theo dung schema users hien tai de copy/paste dung ngay.