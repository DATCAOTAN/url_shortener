CREATE UNIQUE INDEX IF NOT EXISTS idx_links_owner_url
ON links (owner_id, original_url)
WHERE owner_id IS NOT NULL;
