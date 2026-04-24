DROP INDEX IF EXISTS idx_links_expires_at;

ALTER TABLE links
DROP COLUMN IF EXISTS expires_at;
