# URL Shortener - Huong Dan Chi Tiet Cac Tinh Nang Thu Nghiem

Tai lieu nay giai thich 4 tinh nang moi theo cach de nguoi moi cung co the hieu:
1. Loc du lieu link nang cao
2. Co che cooldown khi tao link
3. Phan trang va sap xep danh sach link
4. Co che het han link (TTL)

Muc tieu cua tai lieu:
- Giai thich "vi sao can tinh nang nay"
- Giai thich "luong du lieu chay nhu the nao"
- Giai thich "code dang xu ly ra sao"
- Dua ra vi du test de de kiem tra lai

---

## 1) Tong quan kien truc lien quan

Duong di chung cua 1 request trong du an:

1. Route nhan request
2. Middleware (neu co) kiem tra auth/role
3. Handler doc input va validate
4. Service xu ly logic nghiep vu
5. Repository truy van DB
6. Handler tra ve response JSON hoac redirect

Cac file chinh lien quan:
- src/routes/link_route.rs
- src/routes/admin_route.rs
- src/handlers/link_handler.rs
- src/handlers/admin_handler.rs
- src/services/link_service.rs
- src/repositories/link_repository.rs
- src/models/link.rs
- src/dtos/link.rs
- src/state.rs

---

## 2) Tinh nang LOC DU LIEU LINK NANG CAO

### 2.1 Muc dich

Tinh nang nay giup admin loc nhanh danh sach link theo:
- So luot click toi thieu (min_clicks)
- Trang thai active/inactive (is_active)

Thay vi lay tat ca link roi loc bang tay, admin co the goi API loc truc tiep.

### 2.2 API va input

Endpoint:
- GET /admin/links/search?min_clicks=<so>&is_active=<true|false>

Input duoc map vao struct SearchQuery:
- min_clicks: i64
- is_active: bool

### 2.3 Luong xu ly

1. Request vao route admin (co middleware admin).
2. Handler search_links trong src/handlers/admin_handler.rs nhan Query<SearchQuery>.
3. Handler goi service get_links_with_min_clicks.
4. Service goi repository get_links_with_min_clicks.
5. Repository chay SQL:
   - WHERE click_count >= $1
   - AND is_active = $2
6. Ket qua map thanh LinkResponse va tra JSON array.

### 2.4 Vi du

Request:
- GET /admin/links/search?min_clicks=100&is_active=true

Y nghia:
- Chi lay link con active
- Va co click_count >= 100

---

## 3) Tinh nang COOLDOWN KHI TAO LINK

### 3.1 Muc dich

Khi user bam tao link lien tuc trong thoi gian rat ngan, he thong co the bi spam hoac tao qua nhieu ban ghi khong can thiet.

Cooldown giai quyet van de nay bang cach:
- Moi user chi duoc tao link moi sau mot khoang cho ngan (hien tai la 5 giay).

### 3.2 Du lieu luu cooldown

Trong src/state.rs, AppState co them:
- cooldown: Arc<Mutex<HashMap<i64, Instant>>>

Y nghia:
- Key: user_id
- Value: thoi diem user vua tao link gan nhat

### 3.3 Luong xu ly khi tao link

Trong handler create_link (src/handlers/link_handler.rs):

1. Parse user_id tu JWT claim.
2. Lay lock cooldown_map.
3. Neu user da co timestamp cu:
   - Tinh elapsed = now - last_request
   - Neu elapsed < COOLDOWN_SECONDS (5):
     - Tra loi 429 TooManyRequests
4. Neu hop le:
   - Ghi timestamp moi vao map
5. Tiep tuc xu ly tao link trong DB.
6. Neu DB loi:
   - Xoa user khoi cooldown_map de user co the retry ngay.

### 3.4 Luu y quan trong

Cooldown hien tai la in-memory:
- Uu diem: nhanh, don gian.
- Nhuoc diem:
  - Restart server se mat du lieu cooldown.
  - Neu scale nhieu instance, moi instance co map rieng.

Neu can production-level:
- Co the chuyen cooldown sang Redis de dung chung giua cac instance.

---

## 4) Tinh nang PHAN TRANG VA SAP XEP DANH SACH LINK

### 4.1 Muc dich

Khi so luong link lon, tra ve het 1 lan se nang payload va kho xu ly o frontend.

Phan trang + sap xep giup:
- Giam kich thuoc response
- Load du lieu theo trang
- Hien thi du lieu co thu tu ro rang

### 4.2 API va tham so

Endpoint:
- GET /admin/links?current_page=...&limit=...&sort_by=...

Tham so trong PaginationQuery:
- current_page: Option<u32>
- limit: Option<u32>
- sort_by: Option<String>

Gia tri mac dinh:
- current_page = 1
- limit = 10
- sort_by = "clicks_desc"

Gia tri sort_by hop le:
- clicks_desc
- clicks_asc

Neu sort_by sai:
- Tra ve 400 BadRequest

### 4.3 Cong thuc tinh offset

Cong thuc duoc dung:
- offset = (page - 1) * limit

Vi du:
- page = 3, limit = 10
- offset = (3 - 1) * 10 = 20

Y nghia:
- Bo qua 20 phan tu dau
- Lay tiep 10 phan tu cho trang 3

### 4.4 Luong xu ly

Trong src/handlers/admin_handler.rs:

1. Lay links tu service.
2. Map sang LinkResponse.
3. Sort in-memory theo sort_by.
4. Tinh total_items, total_pages.
5. Tinh offset theo cong thuc tren.
6. Cat mang:
   - skip(offset)
   - take(limit)
7. Tra ve PaginationResponse gom:
   - data: du lieu trang hien tai
   - metadata: current_page, limit, offset, total_items, total_pages, sort_by

### 4.5 Vi du response

```json
{
  "data": [
    {
      "id": 1,
      "short_code": "abc",
      "original_url": "https://example.com",
      "title": "Example",
      "click_count": 120,
      "is_active": true,
      "expires_at": null
    }
  ],
  "metadata": {
    "limit": 10,
    "offset": 0,
    "sort_by": "clicks_desc",
    "total_items": 24,
    "total_pages": 3,
    "current_page": 1
  }
}
```

---

## 5) Tinh nang CO CHE HET HAN LINK (TTL)

### 5.1 Muc dich

TTL giup link chi co hieu luc trong mot khoang thoi gian.
Sau khi het han:
- Khong redirect nua
- Tra ve thong bao loi ro rang

### 5.2 Cau truc du lieu

Trong model Link (src/models/link.rs):
- expires_at: Option<u64>

Y nghia:
- Some(unix_timestamp): link co han
- None: link vo thoi han

DB migration them cot:
- links.expires_at BIGINT

File migration:
- migrations/20260422010000_add_link_expires_at.up.sql
- migrations/20260422010000_add_link_expires_at.down.sql

### 5.3 Tao link co TTL

Endpoint tao link (thu nghiem):
- POST /api/links/create

Body co the co:
- expires_in_seconds (optional)

Logic trong create_link:

1. Neu expires_in_seconds duoc truyen:
   - Bat buoc > 0
   - current_timestamp = now (unix giay)
   - expires_at = current_timestamp + expires_in_seconds
   - Kiem tra overflow va mien i64 (de luu BIGINT an toan)
2. Neu khong truyen:
   - expires_at = None (vo thoi han)
3. Goi service -> repository de insert DB.
4. Response tra ve ca expires_at.

### 5.4 Redirect va kiem tra het han

Endpoint redirect:
- GET /{short_code}

Service resolve_redirect_target xu ly:

1. Tim link active theo short_code.
2. Neu khong tim thay -> NotFound (404).
3. Neu tim thay va co expires_at:
   - So sanh expires_at voi current timestamp.
   - Neu expires_at < now -> Expired.
4. Neu chua het han:
   - Tang click_count + ghi analytics async
   - Tra ve Found(url).

Handler redirect_link:
- Found -> redirect (307/303 tuy theo client)
- Expired -> tra 410 Gone + text don gian
- NotFound -> 404

### 5.5 Vi sao dung 410 Gone

410 Gone la ma HTTP chuan cho tai nguyen da khong con hieu luc.
So voi 404:
- 404: co the khong ton tai hoac sai duong dan
- 410: tung ton tai, nhung hien tai khong dung nua

---

## 6) Tinh tuong tac giua cac tinh nang

Cac tinh nang nay khong tach roi ma bo tro cho nhau:

1. Tao link co cooldown
   - Giam spam ngay tu dau vao.
2. Link co TTL
   - Tu dong ket thuc hieu luc theo thoi gian.
3. Admin co bo loc + phan trang + sap xep
   - Quan tri du lieu lon de dang hon.
4. Response co metadata
   - Frontend de ve table va pager.

---

## 7) Kich ban test de xac minh nhanh

Co the dung file test_api.http de test:

1. Login user/admin lay token.
2. Tao link co TTL:
   - POST /api/links/create voi expires_in_seconds = 3600
3. Tao link khong TTL:
   - POST /api/links/create khong truyen expires_in_seconds
4. Test redirect link het han (seed san short_code = expired1):
   - GET /expired1 -> expect 410
5. Test redirect link con han (seed san short_code = futurettl):
   - GET /futurettl -> expect redirect
6. Test phan trang/sap xep:
   - GET /admin/links?current_page=2&limit=5&sort_by=clicks_asc
7. Test loc nang cao:
   - GET /admin/links/search?min_clicks=100&is_active=true

---

## 8) Gioi han hien tai va huong nang cap

### 8.1 Cooldown

Hien tai la in-memory, nen:
- Mat du lieu khi restart
- Khong dong bo da instance

Huong nang cap:
- Chuyen sang Redis key theo user_id + TTL.

### 8.2 Phan trang

Dang sort + paginate in-memory sau khi lay tat ca link.

Huong nang cap:
- Day ORDER BY / LIMIT / OFFSET xuong SQL de toi uu khi du lieu rat lon.

### 8.3 TTL va cache

Dang uu tien do chinh xac nghiep vu (check TTL truoc redirect), tranh redirect nham link da het han.

Huong nang cap:
- Co the cache them metadata TTL (hoac cache object) neu can toi uu them.

---

## 9) Ket luan

Bo tinh nang nay phu hop cho bai tap ung dung va moi truong thu nghiem:
- Co quan tri admin de nhin du lieu tot hon
- Co bao ve tao link khoi spam nhanh
- Co co che song/chet cua link ro rang theo thoi gian

Neu can dua len production, nen uu tien:
1. Dua cooldown sang Redis
2. Day pagination/sorting xuong DB
3. Bo sung metric/log cho cac case 410 va 429
4. Viet them integration test tu dong cho toan bo flow
