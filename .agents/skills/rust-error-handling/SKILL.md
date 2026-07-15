---
name: rust-error-handling
description: How to handle errors in Pulsar's Rust backend using the crate::error module, thiserror derive, and Serialize impl for frontend consumption.
---

# Rust Error Handling

All Rust backend code uses the custom `Error` enum from `src-tauri/src/error.rs`.

## Error Enum

```rust
use crate::error::{Error, Result};

// Use the Result alias everywhere
pub async fn my_function() -> Result<String> {
    // Automatic conversion from sqlx::Error, io::Error, etc.
    let data = sqlx::query("SELECT ...").fetch_one(&pool).await?;

    // Manual error construction
    if invalid {
        return Err(Error::Validation("Invalid input".to_string()));
    }

    Ok("success".to_string())
}
```

## Error Variants

| Variant | Code | When to use |
|---------|------|-------------|
| `Error::Database(sqlx::Error)` | `"Database"` | Auto-converted from sqlx errors |
| `Error::Io(io::Error)` | `"Io"` | File system operations |
| `Error::Encryption(String)` | `"Encryption"` | Encryption failures |
| `Error::Decryption(String)` | `"Decryption"` | Decryption failures |
| `Error::Validation(String)` | `"Validation"` | Input validation errors |
| `Error::Internal(String)` | `"Internal"` | General internal errors |
| `Error::VaultLocked` | `"VaultLocked"` | Operation requires unlocked vault |
| `Error::VaultNotLoaded` | `"VaultNotLoaded"` | No database loaded |
| `Error::InvalidPassword` | `"InvalidPassword"` | Wrong master password |
| `Error::Totp(String)` | `"Totp"` | TOTP verification errors |
| `Error::Serialization(serde_json::Error)` | `"Serialization"` | JSON parse errors |
| `Error::Tauri(tauri::Error)` | `"Tauri"` | Tauri framework errors |

## Frontend Error Handling

Errors are serialized as `{ code: string, message: string }` and handled by `callBackend()`:

```typescript
// In callBackend() — errors are caught, logged, and shown as toasts
// Commands starting with "is_" or "check_" suppress toast errors

// Manual error handling
try {
  await callBackend('my_command');
} catch (error) {
  const e = error as { code: string; message: string };
  if (e.code === 'VaultLocked') {
    // Handle specific error
  }
}
```

## Common Patterns

### Accessing Vault State
```rust
// Get DB pool (errors if no vault loaded)
let pool = crate::db::utils::get_db_pool(&state).await?;

// Get encryption key (errors if vault locked)
let key = crate::db::utils::get_key(&state).await?;
```

### Validation Before Operations
```rust
if input.trim().is_empty() {
    return Err(Error::Validation("Input cannot be empty".to_string()));
}
```

### Converting External Errors
```rust
// For errors that don't have From impl
some_operation()
    .map_err(|e| Error::Internal(format!("Operation failed: {e}")))?;
```
