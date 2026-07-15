---
name: testing
description: How to test Pulsar — Rust unit tests, frontend type-checking, linting, and formatting workflows.
---

# Testing & Verification

## Quick Reference

```bash
# Frontend
bun check           # svelte-check + TypeScript
bun lint            # ESLint with auto-fix
bun prettier --write .  # Format all files

# File-scoped
bun tsc --noEmit <file>              # Type-check single file
bun eslint --fix <file>              # Lint single file
bun prettier --write <file>          # Format single file

# Rust
cd src-tauri && cargo test           # Run all Rust tests
cd src-tauri && cargo test <name>    # Run specific test
cd src-tauri && cargo check          # Fast type-check without building
cd src-tauri && cargo clippy         # Lint Rust code

# Full app
bun tauri dev       # Development build + hot reload
bun tauri build     # Production build
```

## Existing Rust Tests

### `encryption.rs`
- `test_encrypt_decrypt_roundtrip` — XChaCha20-Poly1305 encrypt/decrypt cycle
- `test_decrypt_invalid_format` — Rejects malformed payloads
- `test_decrypt_bad_base64` — Rejects invalid Base64
- `test_invalid_key_length_errors` — Rejects short keys

### `secmem.rs`
- `test_locked_buffer_lifecycle` — LockedBuffer create/clone/compare
- `test_locked_string_lifecycle` — LockedString create/clone/compare

## Writing New Tests

### Rust Tests

Place tests in the same file using `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function("input");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "expected");
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = my_async_function().await;
        assert!(result.is_ok());
    }
}
```

### Frontend Tests

Use Vitest (compatible with Bun):

```typescript
// src/lib/utils/__tests__/generator.test.ts
import { describe, it, expect } from 'vitest';
import { GeneratorService } from '../generator';

describe('GeneratorService', () => {
  it('generates password of correct length', () => {
    const password = GeneratorService.generate(20);
    expect(password.length).toBe(20);
  });
});
```

## Pre-Commit Checklist

1. `bun prettier --write .` — format
2. `bun lint` — lint
3. `bun check` — type-check
4. `cd src-tauri && cargo test` — Rust tests
5. `cd src-tauri && cargo clippy` — Rust lints
