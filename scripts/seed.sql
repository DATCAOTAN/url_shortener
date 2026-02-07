-- 1. Dọn dẹp dữ liệu cũ trước khi nạp (Reset ID về 1)
TRUNCATE TABLE link_analytics, refresh_tokens, links, users RESTART IDENTITY CASCADE;

-- 2. Tạo User mẫu
-- Tất cả user dưới đây đều có mật khẩu là: "password123"
-- Hash: $2b$10$mLGLk2bUTMLvj.9HbdQ3MOyw.KMHebfnAIEyebI7inzORWzZu05Ve
INSERT INTO users (username, password_hash, email, role, is_active)
VALUES 
('admin', '$2b$10$mLGLk2bUTMLvj.9HbdQ3MOyw.KMHebfnAIEyebI7inzORWzZu05Ve', 'admin@system.com', 'admin', true),
('mod_user', '$2b$10$mLGLk2bUTMLvj.9HbdQ3MOyw.KMHebfnAIEyebI7inzORWzZu05Ve', 'mod@system.com', 'user', true),
('basic_user', '$2b$10$mLGLk2bUTMLvj.9HbdQ3MOyw.KMHebfnAIEyebI7inzORWzZu05Ve', 'user@system.com', 'user', true);

-- 3. Tạo Link mẫu
-- Lưu ý: ID user sẽ tự tăng: admin=1, mod_user=2, basic_user=3
INSERT INTO links (owner_id, original_url, short_code, title, click_count, is_active)
VALUES 
-- Link của Admin (ID 1)
(1, 'https://www.google.com', 'google', 'Google Search', 1500, true),
(1, 'https://www.rust-lang.org', 'rust', 'Rust Programming Language', 500, true),

-- Link của Basic User (ID 3)
(3, 'https://shopee.vn', 'shopee', 'Shopee Vietnam', 10, true),
(3, 'https://tiki.vn', 'tiki', 'Tiki E-commerce', 5, true),

-- Link ẩn danh (Không có chủ sở hữu - owner_id NULL)
(NULL, 'https://news.ycombinator.com', 'hacker', 'Hacker News', 100, true);

-- 4. Tạo dữ liệu thống kê mẫu (Analytics)
-- Giả sử hôm nay và hôm qua có click
INSERT INTO link_analytics (link_id, date, clicks)
VALUES
-- Thống kê cho link 'google' (ID 1)
(1, CURRENT_DATE, 100),       -- Hôm nay 100 click
(1, CURRENT_DATE - 1, 250),   -- Hôm qua 250 click
(1, CURRENT_DATE - 2, 120),   -- Hôm kia 120 click

-- Thống kê cho link 'rust' (ID 2)
(2, CURRENT_DATE, 50);