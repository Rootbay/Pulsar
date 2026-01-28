-- Create search_indices table if it doesn't exist (fixing failed migration 13)
CREATE TABLE IF NOT EXISTS search_indices (
    item_id INTEGER NOT NULL,
    field_name TEXT NOT NULL,
    token BLOB NOT NULL,
    FOREIGN KEY (item_id) REFERENCES password_items (id) ON DELETE CASCADE
);

-- Add indices for performance
CREATE INDEX IF NOT EXISTS idx_search_indices_token ON search_indices(token);
CREATE INDEX IF NOT EXISTS idx_search_trigrams_hash ON search_trigrams(trigram_hash);
CREATE INDEX IF NOT EXISTS idx_item_tags_reverse ON item_tags(tag_id, item_id);
CREATE INDEX IF NOT EXISTS idx_password_items_updated_at ON password_items(updated_at);
