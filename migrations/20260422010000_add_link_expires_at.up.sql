ALTER TABLE links
ADD COLUMN IF NOT EXISTS expires_at BIGINT;

CREATE INDEX IF NOT EXISTS idx_links_expires_at ON links(expires_at);
