# Pulsar — AI Agent Rules

## Project Identity

**Pulsar** is an offline-first, security-focused desktop password manager built with **Tauri v2** (Rust backend) and **SvelteKit** (Svelte 5 runes mode) frontend. It stores credentials in an encrypted SQLite database using ChaCha20-Poly1305 / AES-256-GCM with Argon2id key derivation. The app identifier is `com.rootbay.pulsar`.

---

## Tech Stack (exact versions)

| Layer | Technology | Version |
|---|---|---|
| Frontend framework | SvelteKit + Svelte 5 (runes) | svelte ^5.48, kit ^2.50 |
| Language | TypeScript (strict mode) | ^5.9 |
| CSS | TailwindCSS v4 (CSS-first) | ^4.1 |
| UI components | shadcn-svelte (bits-ui primitives) | bits-ui ^2.15 |
| Icons | lucide-svelte / @lucide/svelte | ^0.544 / ^0.561 |
| Desktop runtime | Tauri v2 | tauri 2.3 |
| Backend | Rust (edition 2024) | — |
| Database | SQLite via sqlx + bundled-sqlcipher | sqlx 0.8 |
| Encryption | chacha20poly1305, argon2, x25519-dalek | — |
| Package manager / runtime | Bun | — |
| Vite | Vite 7 | ^7.3 |
| Adapter | @sveltejs/adapter-static (SPA) | ^3.0 |

---

## Architecture Overview

### Frontend (`src/`)

```
src/
├── app.html              # HTML shell (title: "Pulsar Pass")
├── app.css               # TailwindCSS v4 theme (oklch, @theme inline, @custom-variant dark)
├── lib/
│   ├── components/       # Svelte components
│   │   ├── layout/       # Sidebar, PasswordList, PasswordDetail (main three-panel layout)
│   │   ├── password/     # PasswordDetailHeader, PasswordGenerator, PasswordStrength, TagList
│   │   ├── ui/           # shadcn-svelte primitives + custom wrappers (Popup, FieldInput, etc.)
│   │   ├── CreatePasswordPopup.svelte
│   │   ├── DeletePasswordPopup.svelte
│   │   ├── ImportManagerPopup.svelte
│   │   ├── KeyboardShortcutsPopup.svelte
│   │   ├── SecurityManager.svelte    # Auto-lock, inactivity timer, lock-on-suspend
│   │   └── ...
│   ├── config/
│   │   ├── settings.ts    # AllSettings interface + all default*Settings constants
│   │   └── keybinds.ts    # Keybind interface + defaultKeybinds array
│   ├── hooks/
│   │   └── is-mobile.svelte.ts
│   ├── i18n/              # 20 language JSON files (en, sv, es, fr, de, pt-BR, zh, ru, ja, hi, ko, ar, it, tr, nl, pl, id, th, vi, el)
│   ├── i18n.svelte.ts     # i18n system: t(locale, key, vars), currentLocale, detectSystemLocale()
│   ├── icons.ts           # Barrel export of lucide icons
│   ├── stores/            # Svelte 5 rune-based stores (class pattern with $state/$derived/$effect)
│   │   ├── appState.svelte.ts          # AppState class: isLocked, isDatabaseLoaded, selectedTag, filterCategory
│   │   ├── appSettings.svelte.ts       # SettingsStore class: loads/saves AllSettings via Rust backend
│   │   ├── vault.svelte.ts             # VaultStore class: items, search, pagination, CRUD
│   │   ├── tags.svelte.ts              # TagStore
│   │   ├── sync.svelte.ts             # SyncStore: WebDAV sync with encrypted payloads
│   │   ├── security-dashboard.svelte.ts
│   │   ├── profile.svelte.ts
│   │   ├── general.svelte.ts
│   │   ├── recentDatabases.svelte.ts
│   │   └── totp.svelte.ts
│   ├── stores.ts          # Barrel re-exports of all stores
│   ├── types/
│   │   ├── password.ts          # PasswordItem, PasswordItemOverview, Attachment, ActivityEntry
│   │   └── password-fields.ts   # DisplayField, TotpDisplayField, BaseDisplayField
│   ├── utils/
│   │   ├── backend.ts           # callBackend<T>(command, args, retries) — wraps invoke() with retry + error toast
│   │   ├── generator.ts         # GeneratorService: password/passphrase generation
│   │   ├── clipboardService.svelte.ts  # Clipboard policy management
│   │   ├── passwordFields.ts    # Field extraction logic
│   │   ├── security.ts          # Frontend security helpers
│   │   ├── copyHelper.ts        # Copy-to-clipboard helpers
│   │   ├── backup.ts            # Backup/restore utilities
│   │   ├── error.ts             # parseError utility
│   │   └── wordlist.ts          # EFF large wordlist for passphrases
│   └── utils.ts           # cn() (clsx+twMerge), WithoutChild, WithElementRef type helpers
├── routes/
│   ├── +layout.svelte     # Root layout: theme/appearance effects, clipboard init, autotype events, auth routing
│   ├── +layout.ts         # SSR=false, loads settings, checks vault state, auto-opens default vault
│   ├── (app)/             # Main app layout (three-panel: sidebar + password list + detail)
│   │   ├── +layout.svelte
│   │   └── +page.svelte
│   ├── (auth)/            # Auth routes
│   │   ├── login/         # Master password entry
│   │   ├── setup/         # Initial vault setup
│   │   ├── select-vault/  # Vault selector
│   │   └── totp/          # TOTP verification
│   └── settings/          # 13 settings sub-pages (about, advanced, appearance, autofill, backup, clipboard, general, generator, presets, security, sessions, site-rules, vault)
│       ├── +layout.svelte
│       └── [section]/+page.svelte
```

### Backend (`src-tauri/`)

```
src-tauri/
├── tauri.conf.json       # App config: identifier com.rootbay.pulsar, CSP, updater endpoints
├── Cargo.toml            # Rust deps (tauri 2.3, sqlx, chacha20poly1305, argon2, x25519-dalek, etc.)
├── capabilities/         # Tauri v2 permissions (default.json, biometric.json)
├── migrations/           # 19 SQL migration files (sequential: buttons → passwords → attachments → search → activity)
├── src/
│   ├── main.rs           # Entry: locks process memory, creates AppState, registers all commands, sets up clipboard exit handler
│   ├── state.rs          # AppState struct: db, key (LockedBuffer), pending_key, db_path, clipboard_policy, rate_limit, unlock_guard
│   ├── error.rs          # Error enum with code() method + Serialize impl for frontend
│   ├── auth/
│   │   ├── mod.rs        # get_db_pool, get_db_path, verify_master_password_internal, load_metadata
│   │   ├── commands.rs   # Tauri commands: unlock, lock, set_master_password, rotate_master_password, configure/disable_login_totp, biometrics
│   │   ├── types.rs      # PasswordMetadata, Argon2ParamsConfig, constants (PENDING_TOTP_TTL, rate limits)
│   │   ├── crypto_utils.rs  # derive_key (Argon2id), decode_metadata
│   │   ├── metadata.rs   # Read/write password metadata to .meta.json sidecar
│   │   └── biometrics.rs # Windows Hello / macOS biometric support via keyring
│   ├── db/
│   │   ├── mod.rs        # Re-exports all db submodules
│   │   ├── core.rs       # init_db_lazy: SQLite pool with WAL, sqlcipher PRAGMA key, connection pragmas
│   │   ├── passwords.rs  # CRUD for password_items with per-record encryption via CipherSession
│   │   ├── buttons.rs    # Sidebar "buttons" (tags/categories) CRUD
│   │   ├── attachments.rs # File attachments (encrypted, stored on disk)
│   │   ├── activity.rs   # Activity log (encrypted events)
│   │   ├── config.rs     # configuration table KV store (profile settings)
│   │   ├── recipient_keys.rs  # X25519 recipient public keys
│   │   ├── utils.rs      # Helper: get_db_pool, get_key from State
│   │   └── validation.rs # Input validation
│   ├── encryption.rs     # XChaCha20-Poly1305: encrypt/decrypt strings & bytes, CipherSession (reusable cipher + HMAC search tokens + trigram hashes)
│   ├── secmem.rs         # LockedBuffer: page-aligned, VirtualLock/mlock pinned, zeroize-on-drop memory; LockedString wrapper
│   ├── crypto.rs         # Export/import entries: passphrase-based (Argon2id+XChaCha20) and X25519 ephemeral key exchange
│   ├── security.rs       # Device registry, security report (reused/weak/breached passwords), integrity check
│   ├── clipboard.rs      # Windows clipboard history blocking (registry), timed clipboard clear, policy management
│   ├── backup_commands.rs # Vault export/import/restore, WebDAV sync encrypt/decrypt/merge
│   ├── vault_commands.rs # list_vaults, switch_database, is_database_loaded
│   ├── file_dialog.rs    # Native file dialogs, elevated_copy, open_app_data_folder
│   ├── settings/
│   │   ├── mod.rs        # get_all_settings, set_all_settings, apply_system_settings (startup, tray)
│   │   └── system.rs     # Global hotkey listener (Windows), autotype simulation, system tray setup
│   ├── totp.rs           # TOTP generation/verification (SHA1, 6 digits, 30s period)
│   ├── tray.rs           # System tray setup: Show/Quit menu items
│   ├── utils.rs          # Misc helpers
│   └── types/            # Rust types: PasswordItem, ExportPayload, PubKeyExportPayload, VaultEntry, etc.
```

---

## Critical Patterns & Conventions

### Frontend-Backend Communication

All Rust commands are called via `callBackend<T>(command, args)` from `$lib/utils/backend.ts`. This wrapper:
- Uses `@tauri-apps/api/core` `invoke()`
- Retries transient errors (busy/timeout/locked) up to 3 times with exponential backoff
- Shows toast errors via `svelte-sonner` (suppressed for `is_*` / `check_*` commands)
- Centralizes error handling — **never call `invoke()` directly**

### Svelte 5 Rune Stores

All stores use the **class-based rune pattern** (NOT legacy `writable()`/`readable()`):
```typescript
// File must use .svelte.ts extension
class MyStore {
  value = $state<string>('');
  computed = $derived(this.value.toUpperCase());

  constructor() {
    $effect.root(() => {
      $effect(() => { /* reactive side effects */ });
    });
  }
}
export const myStore = new MyStore();
```

### Encryption Architecture

1. **Database-level**: SQLCipher (bundled via `libsqlite3-sys`) encrypts the entire `.db` file
2. **Record-level**: Each password field is individually encrypted with `CipherSession` (XChaCha20-Poly1305) using the vault key
3. **Key derivation**: Master password → Argon2id (64 MiB, 3 iterations, 4 parallelism) → 256-bit key
4. **Secure memory**: All keys use `LockedBuffer` (page-aligned, VirtualLock/mlock, zeroize-on-drop)
5. **Search**: Encrypted trigram search using HMAC-SHA256 tokens (truncated to 8 bytes)

### Settings System

Settings are managed by `SettingsStore` (`appSettings.svelte.ts`):
- Persisted via Rust backend (`get_all_settings` / `set_all_settings`) which uses `tauri-plugin-store`
- Shape defined in `$lib/config/settings.ts` as `AllSettings` interface
- Deep-merged with defaults on load to handle schema evolution
- Debounced save (500ms) via `settings.save()`
- To add a new setting: update the interface, add default, the store auto-merges

### i18n

- 20 languages with English as fallback
- Key-based: `t(locale, 'key_name', { var: value })`
- Translation files: `src/lib/i18n/*.json`
- System locale auto-detection with fallback chain

### Auth Flow (route-driven)

The root `+layout.svelte` determines which auth route to show:
1. No database loaded → `/select-vault`
2. No master password set → `/setup`
3. Vault locked → `/login`
4. TOTP required but not verified → `/totp`
5. All good → `/` (main app)

### Three-Panel Layout

The main app (`(app)/+layout.svelte`) uses a three-panel layout:
- **Sidebar** (left): Navigation, tag filters, categories — width controlled by `--sidebar-width`
- **Password List** (center): Scrollable list with search — width controlled by `--passwordList-width`
- **Password Detail** (right): Full item detail with edit mode, custom fields, attachments

---

## Do's & Don'ts

### Do:
- Use `callBackend()` for ALL Rust IPC calls
- Use `.svelte.ts` extension for files containing runes outside components
- Use `$state`, `$derived`, `$effect` (Svelte 5 runes) — never legacy `$:` or `writable()`
- Use shadcn-svelte primitives from `$lib/components/ui/`
- Use `cn()` from `$lib/utils.ts` for conditional class merging
- Use `t(locale, key)` for all user-facing strings
- Use `LockedBuffer`/`Zeroizing<>` in Rust for any sensitive data
- Use `CipherSession` for encrypting/decrypting record fields
- Use per-record AEAD — never store plaintext secrets
- Follow the existing error pattern: return `crate::error::Result<T>`
- Run `bun prettier --write .` before commits
- Derive Serialize/Deserialize with `#[serde(rename_all = "camelCase")]` for Rust types exposed to frontend
- Register all new Tauri commands in `main.rs` `generate_handler![]`
- Add Tauri permissions in `capabilities/default.json` for new commands

### Don't:
- Don't call `invoke()` directly — use `callBackend()`
- Don't use `any` in TypeScript unless unavoidable (add `TODO`)
- Don't use legacy Svelte stores (`writable()`, `readable()`)
- Don't use `$:` reactive declarations (Svelte 4 syntax)
- Don't hardcode secrets in frontend code
- Don't skip `zeroize()` for any key material in Rust
- Don't add CSS when TailwindCSS utilities exist
- Don't create new UI primitives — use shadcn-svelte components
- Don't skip error handling — all backend calls can fail
- Don't store sensitive data in `tauri-plugin-store` — only non-secret settings

---

## Database Migrations

Migrations are plain SQL files in `src-tauri/migrations/` named with sequential numeric prefixes:
```
1_create_buttons_table.sql
2_create_password_items_table.sql
...
19_add_search_indices_and_tables.sql
```

The next migration should be numbered `20_*.sql`. Migrations run automatically on vault open.

---

## Testing

- **Rust tests**: `cargo test` in `src-tauri/` (encryption roundtrip, LockedBuffer lifecycle tests exist)
- **Frontend checks**: `bun check` (svelte-check + TypeScript), `bun lint` (ESLint)
- **Type-check**: `bun tsc --noEmit`
- **Format**: `bun prettier --write .`

---

## Build & Run

```bash
bun install          # Install dependencies
bun tauri dev        # Development (hot-reload frontend + Rust backend)
bun tauri build      # Production build (platform-specific binaries)
```

---

## External References

- Tauri v2: https://tauri.app
- Svelte 5: https://svelte.dev
- TailwindCSS v4: https://tailwindcss.com/docs/v4
- shadcn-svelte: https://shadcn-svelte.com
- bits-ui: https://bits-ui.com
- lucide icons: https://lucide.dev
- sqlx: https://docs.rs/sqlx
- chacha20poly1305: https://docs.rs/chacha20poly1305
- argon2: https://docs.rs/argon2
