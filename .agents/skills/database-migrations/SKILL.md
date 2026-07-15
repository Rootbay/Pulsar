---
name: database-migrations
description: How to create SQLite database migrations for Pulsar's encrypted vault. Covers naming conventions, sequential numbering, and encryption-aware schema design.
---

# Creating Database Migrations

Pulsar uses SQLite with SQLCipher encryption. Migrations are plain SQL files that run automatically when a vault is opened.

## Location & Naming

Migrations live in `src-tauri/migrations/` with sequential numeric prefixes:

```
src-tauri/migrations/
├── 1_create_buttons_table.sql
├── 2_create_password_items_table.sql
├── 3_add_fields_to_password_items.sql
...
├── 19_add_search_indices_and_tables.sql
└── 20_your_new_migration.sql    ← next migration
```

**The next migration number is `20`.**

## Rules

1. **Sequential numbering**: Always use the next integer. Check existing files first.
2. **Descriptive suffix**: `{number}_{description}.sql` — use snake_case
3. **Idempotent when possible**: Use `IF NOT EXISTS` for CREATE statements
4. **No destructive changes**: Never DROP columns in production. Add new columns with defaults.
5. **Encrypted data**: Sensitive columns store Base64-encoded ciphertext (nonce:ciphertext format), not plaintext.

## Example Migration

```sql
-- 20_add_favorite_to_password_items.sql
ALTER TABLE password_items ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_password_items_favorite ON password_items(is_favorite);
```

## Existing Schema (key tables)

### `password_items`
Core table for vault entries. Sensitive fields (`title`, `description`, `password`, `username`, `url`, `notes`, `tags`, `totp_secret`, `custom_fields`) are encrypted with XChaCha20-Poly1305 per-record.

### `buttons`
Sidebar tags/categories with id, text, icon, color, position.

### `attachments`
File attachments stored encrypted on disk. Metadata in DB (file_name, file_size, mime_type, disk_path).

### `search_trigrams`
HMAC-SHA256 trigram hashes for encrypted search. Links to item_id.

### `activity_log`
Encrypted event log with event_type, item_id, item_title, details, created_at.

### `configuration`
Key-value store for vault metadata (password salt, argon2 params, device registry, profile settings).

### `item_tags`
Many-to-many relationship between password_items and buttons (tags).

### `recipient_keys`
X25519 public keys for encrypted sharing.

## After Creating a Migration

1. The migration runs automatically on next vault open via `sqlx::migrate!()` in the Rust backend
2. Update relevant Rust structs in `src-tauri/src/types/` if schema changed
3. Update queries in `src-tauri/src/db/passwords.rs` (or relevant db module)
4. Update TypeScript types in `src/lib/types/password.ts` if frontend-facing
