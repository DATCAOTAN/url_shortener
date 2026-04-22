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
-- expires_at lưu Unix timestamp (giây). NULL nghĩa là link vô thời hạn.
INSERT INTO links (owner_id, original_url, short_code, title, click_count, is_active, expires_at)
VALUES 
-- Link của Admin (ID 1)
(1, 'https://www.google.com', 'google', 'Google Search', 1500, true, NULL),
(1, 'https://www.rust-lang.org', 'rust', 'Rust Programming Language', 500, true, NULL),
(1, 'https://www.youtube.com', 'youtube', 'YouTube', 2200, true, NULL),
(1, 'https://www.github.com', 'github', 'GitHub', 1200, true, NULL),
(1, 'https://www.stackoverflow.com', 'stackovf', 'Stack Overflow', 900, true, NULL),
(1, 'https://www.medium.com', 'medium', 'Medium', 250, true, NULL),
(1, 'https://www.cnn.com', 'cnn', 'CNN', 80, true, NULL),

-- Link của Mod User (ID 2)
(2, 'https://www.reddit.com', 'reddit', 'Reddit', 300, true, NULL),
(2, 'https://azure.microsoft.com', 'azure', 'Azure', 450, true, NULL),
(2, 'https://aws.amazon.com', 'aws', 'AWS', 430, true, NULL),
(2, 'https://www.bbc.com', 'bbc', 'BBC', 60, true, NULL),

-- Link của Basic User (ID 3)
(3, 'https://shopee.vn', 'shopee', 'Shopee Vietnam', 10, true, NULL),
(3, 'https://tiki.vn', 'tiki', 'Tiki E-commerce', 5, true, NULL),
(3, 'https://docs.rs', 'docsrs', 'Rust Docs', 70, true, NULL),
(3, 'https://dev.to', 'devto', 'Dev Community', 40, true, NULL),
(3, 'https://www.notion.so', 'notion', 'Notion', 20, true, NULL),
(3, 'https://www.atlassian.com/software/jira', 'jira', 'Jira', 15, true, NULL),
(3, 'https://trello.com', 'trello', 'Trello', 12, true, NULL),
(3, 'https://www.figma.com', 'figma', 'Figma', 8, true, NULL),
(3, 'https://www.canva.com', 'canva', 'Canva', 3, true, NULL),

-- Link ẩn danh (Không có chủ sở hữu - owner_id NULL)
(NULL, 'https://news.ycombinator.com', 'hacker', 'Hacker News', 100, true, NULL),
(NULL, 'https://example.org/old', 'oldlink', 'Old Inactive Link', 999, false, NULL),

-- Link mẫu đã hết hạn (dùng để test API redirect trả 410 Gone)
(NULL, 'https://expired.example.com', 'expired1', 'Expired Link Sample', 77, true, EXTRACT(EPOCH FROM NOW())::BIGINT - 3600),

-- Link mẫu còn hạn (dùng để test redirect hợp lệ)
(NULL, 'https://future.example.com', 'futurettl', 'Future TTL Link', 88, true, EXTRACT(EPOCH FROM NOW())::BIGINT + 86400);

-- 4. Tạo dữ liệu thống kê mẫu (Analytics)
-- Giả sử hôm nay và hôm qua có click
INSERT INTO link_analytics (link_id, date, clicks)
VALUES
-- Thống kê cho link 'google' (ID 1)
(1, CURRENT_DATE, 100),       -- Hôm nay 100 click
(1, CURRENT_DATE - 1, 250),   -- Hôm qua 250 click
(1, CURRENT_DATE - 2, 120),   -- Hôm kia 120 click

-- Thống kê cho link 'rust' (ID 2)
(2, CURRENT_DATE, 50),

-- Thống kê cho link 'youtube' (ID 3)
(3, CURRENT_DATE, 400),
(3, CURRENT_DATE - 1, 320),

-- Thống kê cho link 'github' (ID 4)
(4, CURRENT_DATE, 180),

-- Thống kê cho link 'reddit' (ID 8)
(8, CURRENT_DATE, 90),

-- Thống kê cho link 'azure' (ID 9)
(9, CURRENT_DATE, 130),

-- Thống kê cho link 'aws' (ID 10)
(10, CURRENT_DATE, 125);