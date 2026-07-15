---
name: encryption
description: How to properly encrypt and decrypt data in Pulsar using XChaCha20-Poly1305, CipherSession, LockedBuffer, and Argon2id key derivation. Security-critical patterns.
---

# Encryption Patterns

Pulsar uses a multi-layer encryption architecture. This skill covers how to correctly handle cryptographic operations.

## Layer 1: Database-Level Encryption (SQLCipher)

The entire SQLite database file is encrypted via SQLCipher. This is handled automatically in `db/core.rs`:

```rust
// Key is passed as a hex PRAGMA on connection
opts = opts.pragma("key", format!("\"x'{hex_key}'\""));
```

**You don't need to manage this directly** — it's handled by `init_db_lazy()`.

## Layer 2: Record-Level Encryption (CipherSession)

Individual fields are encrypted using `CipherSession` from `encryption.rs`:

```rust
use crate::encryption::CipherSession;
use crate::db::utils::{get_key, get_db_pool};

// Get key and create cipher session
let key = get_key(&state).await?;
let session = CipherSession::new(key.as_slice())?;

// Encrypt a string field
let encrypted_title = session.encrypt("My Secret Title")?;

// Decrypt a string field
let decrypted_title = session.decrypt(&encrypted_title)?;

// For zeroized decryption (sensitive data like passwords)
let decrypted_password = session.decrypt_zeroized(&encrypted_password)?;
```

### Encrypted Payload Format

All encrypted strings use the format: `{base64_nonce}:{base64_ciphertext}`

### Standalone Encryption (without CipherSession)

```rust
use crate::encryption::{encrypt, decrypt, encrypt_bytes, decrypt_bytes};

let encrypted = encrypt("plaintext", key.as_slice())?;
let decrypted = decrypt(&encrypted, key.as_slice())?;

// For binary data (attachments)
let encrypted_bytes = encrypt_bytes(&data, key.as_slice())?;
let decrypted_bytes = decrypt_bytes(&encrypted_bytes, key.as_slice())?;
```

## Layer 3: Secure Memory (LockedBuffer)

All cryptographic keys MUST use `LockedBuffer` which provides:
- Page-aligned allocation
- OS memory locking (VirtualLock on Windows, mlock on Unix)
- Automatic zeroization on drop
- Constant-time comparison

```rust
use crate::secmem::LockedBuffer;

// Create from slice
let locked_key = LockedBuffer::from_slice(&key_bytes);

// Access key material
let key_slice = locked_key.as_slice();

// Clone if needed (creates independent locked copy)
let key_copy = locked_key.clone();

// LockedBuffer auto-zeroizes on drop — no manual cleanup needed
```

## Key Derivation (Argon2id)

Master password → vault key derivation:

```rust
use crate::auth::crypto_utils::derive_key;

let key = derive_key(password, &salt, &argon2_params)?;
// Returns LockedBuffer with 256-bit key
```

Default Argon2id parameters:
- Memory: 64 MiB (64 * 1024 KiB)
- Iterations: 3
- Parallelism: 4
- Output: 32 bytes (256-bit key)

## Encrypted Search (Trigram Hashing)

For searchable encrypted fields, use HMAC-SHA256 trigram tokens:

```rust
let session = CipherSession::new(key.as_slice())?;

// Generate search token for exact match
let token = session.generate_search_token("search text");

// Generate trigram hashes for fuzzy search
let trigrams = session.generate_trigram_hashes("searchable text");
```

Tokens are truncated to 8 bytes and stored in the `search_trigrams` table.

## Critical Security Rules

1. **NEVER log or print key material** — LockedBuffer's Debug impl shows `<redacted>`
2. **ALWAYS zeroize temporary key buffers** — use `Zeroizing<>` wrapper or call `.zeroize()` 
3. **NEVER store plaintext secrets** in the database
4. **Use constant-time comparison** for authentication checks (`subtle::ConstantTimeEq`)
5. **Generate random nonces** for each encryption operation — never reuse nonces
6. **Use `OsRng`** for all random generation — never `thread_rng()` for crypto
