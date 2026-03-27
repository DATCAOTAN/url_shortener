DROP INDEX IF EXISTS idx_links_owner_url;
CREATE UNIQUE INDEX IF NOT EXISTS idx_links_owner_url_active
ON links (owner_id, original_url)
WHERE owner_id IS NOT NULL AND (is_active IS NULL OR is_active = TRUE);
