ALTER TABLE password_items ADD COLUMN created_at TEXT;
ALTER TABLE password_items ADD COLUMN updated_at TEXT;

UPDATE password_items SET created_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE created_at IS NULL;
UPDATE password_items SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE updated_at IS NULL;