ALTER TABLE links
ADD COLUMN expires_at TIMESTAMPTZ;

CREATE INDEX idx_links_expires_at ON links(expires_at);
