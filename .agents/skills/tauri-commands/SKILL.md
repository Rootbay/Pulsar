---
name: tauri-commands
description: How to create new Tauri v2 commands — Rust function, registration in main.rs, capabilities permissions, and frontend callBackend() integration.
---

# Creating Tauri v2 Commands

This skill covers the end-to-end process for adding a new Tauri command that connects the Svelte frontend to the Rust backend.

## Steps

### 1. Create the Rust Command Function

Create or add to an existing module in `src-tauri/src/`. Commands must be annotated with `#[tauri::command]`.

```rust
use crate::error::Result;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn my_new_command(
    state: State<'_, AppState>,
    some_arg: String,
) -> Result<String> {
    // Access DB pool and encryption key from state:
    let key = crate::db::utils::get_key(&state).await?;
    let pool = crate::db::utils::get_db_pool(&state).await?;

    // Your logic here...
    Ok("result".to_string())
}
```

### 2. Export from Module

If the command is in a submodule (e.g., `db/my_module.rs`), re-export it from the parent `mod.rs`:

```rust
// In db/mod.rs
pub mod my_module;
pub use my_module::*;
```

### 3. Register in `main.rs`

Add the command to the `generate_handler![]` macro in `src-tauri/src/main.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    db::my_new_command,  // Add here
])
```

### 4. Add Capabilities Permission

Update `src-tauri/capabilities/default.json` to allow the command:

```json
{
  "permissions": [
    "core:default",
    // ... existing permissions ...
  ]
}
```

For custom commands, the permission is usually auto-granted via `core:default`. Only plugin-specific commands need explicit permission entries.

### 5. Call from Frontend

Use `callBackend()` from `$lib/utils/backend.ts` — **never call `invoke()` directly**:

```typescript
import { callBackend } from '$lib/utils/backend';

const result = await callBackend<string>('my_new_command', {
  someArg: 'value'  // camelCase on frontend → snake_case in Rust
});
```

## Important Conventions

- **Error handling**: Always return `crate::error::Result<T>`. The `Error` enum serializes with `code` and `message` fields.
- **Serde naming**: Use `#[serde(rename_all = "camelCase")]` on all structs exposed to the frontend.
- **State access**: Use `State<'_, AppState>` parameter. Access DB via `get_db_pool()`, encryption key via `get_key()`.
- **Sensitive data**: Use `LockedBuffer` for keys, `Zeroizing<>` for temporary secrets, always `zeroize()` after use.
- **Async**: Most commands should be `async` to avoid blocking the main thread.
- **Silent errors**: Commands prefixed with `is_` or `check_` won't show toast errors on the frontend.

## Existing Command Patterns

Look at these files for reference implementations:
- `src-tauri/src/db/passwords.rs` — CRUD with encryption
- `src-tauri/src/auth/commands.rs` — Auth flow with rate limiting
- `src-tauri/src/security.rs` — Device registry with encrypted storage
- `src-tauri/src/backup_commands.rs` — Export/import with file dialogs
