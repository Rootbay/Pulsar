---
name: auth-flow
description: How Pulsar's authentication flow works — master password unlock, TOTP verification, biometric support, vault selection, rate limiting, and the route-driven auth state machine.
---

# Authentication Flow

Pulsar uses a route-driven auth state machine controlled by `+layout.svelte`.

## Auth State Machine

```
No DB loaded  →  /select-vault
       ↓
No password   →  /setup  (first-time vault setup)
       ↓
Vault locked  →  /login  (master password entry)
       ↓
TOTP required →  /totp   (TOTP verification)
       ↓
Unlocked      →  /       (main app)
```

### State Variables (`appState.svelte.ts`)

```typescript
class AppState {
  isLocked = $state<boolean>(true);           // Vault is locked
  isDatabaseLoaded = $state<boolean>(false);   // DB file selected
  needsPasswordSetup = $state<boolean>(false); // No master password yet
  totpVerified = $state<boolean>(false);       // TOTP code verified
  totpRequired = $state<boolean>(false);       // TOTP enabled for vault
}
```

## Unlock Flow (Backend)

1. Frontend calls `callBackend('unlock', { password })` 
2. Rust backend (`auth/commands.rs`):
   - Acquires semaphore (concurrency limit = 1)
   - Checks rate limiting (exponential backoff after failures)
   - Derives key from password using Argon2id
   - Decrypts password check ciphertext
   - Constant-time compares with known plaintext
   - If TOTP configured: stores key in `pending_key` with TTL
   - If no TOTP: sets key in `state.key`, runs migrations, registers device
3. Returns `{ totp_required: bool }`

### Rate Limiting

```rust
// In auth/types.rs
pub const UNLOCK_BACKOFF_BASE_MS: u64 = 250;
pub const UNLOCK_BACKOFF_MAX_MS: u64 = 5000;
pub const UNLOCK_CONCURRENCY_LIMIT: usize = 1;
```

After failed attempts, delay = min(base * 2^failures, max).

## TOTP Flow

1. `unlock` returns `{ totp_required: true }`
2. Frontend navigates to `/totp`
3. User enters 6-digit TOTP code
4. Frontend calls `callBackend('verify_login_totp', { token })`
5. Backend verifies against stored TOTP secret
6. On success: promotes `pending_key` to `state.key`
7. Pending key expires after 120 seconds (`PENDING_TOTP_TTL`)
8. Max 5 TOTP attempts before key is wiped

## Biometric Flow

1. On setup: `callBackend('enable_biometrics', { password })`
   - Derives key, stores in OS keyring (Windows Credential Manager / macOS Keychain)
2. On unlock: `callBackend('unlock_with_biometrics')`
   - Retrieves key from keyring via `keyring` crate
   - Unlocks vault without password entry

## Lock Flow

```typescript
// Frontend
await appState.lock();
// Calls callBackend('lock') → clears key from memory, resets state

// Backend (auth/commands.rs)
// Zeroizes key, closes DB pool, clears pending_key
```

## Vault Selection

1. User picks `.db` file or uses recent vaults list
2. `callBackend('switch_database', { dbPath })` — opens SQLCipher DB
3. `callBackend('is_master_password_configured')` — checks if setup needed
4. Auth state machine takes over from there

## Security Features

- **Memory locking**: `lock_process_memory()` called at startup (pins working set)
- **Auto-lock**: `SecurityManager.svelte` monitors inactivity, suspend events
- **Clipboard clear**: Restores clipboard history settings on exit
- **Device registry**: Tracks which devices have accessed the vault
