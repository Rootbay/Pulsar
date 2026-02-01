# Pulsar

**Pulsar** is a lightweight, modern, and fully offline password manager built with Tauri. Designed for low resource usage while providing high security, Pulsar combines a sleek interface with advanced encryption and organizational features to keep your credentials safe and easily accessible.

<img width="1148" height="669" alt="Pulsar" src="https://github.com/user-attachments/assets/3939baa0-a987-4dc1-8016-6db70d91bbe8" />

---

## Features

- **Offline-first & Lightweight:** Built with Tauri, Pulsar runs efficiently across Windows, macOS, and Linux with minimal RAM usage.

- **Secure Encryption:**
  - Vault key derived from the **master password** via **Argon2id** (memory-hard) with per-vault random 16+ byte salt.
  - Default encryption: **ChaCha20-Poly1305** (authenticated encryption).  
    If AES-NI is detected, automatically uses **AES-256-GCM** for faster encryption.
  - Each record is encrypted individually (per-record AEAD), ensuring compromise of one record does not reveal others.
  - Keys are cleared from memory after use; vault header stores salt and KDF parameters.

- **Vault Management:** All passwords stored locally in a secure SQLite database.

- **Intuitive Filtering & Search:**
  - Filter passwords by categories/tags.
  - Quick search, “All” or “Recently Added” views.

- **Password Creation & Strength Tools:**
  - Generate strong passwords with customizable parameters.
  - Visual password strength indicators.
  - Leak check integration linking to trusted databases.

- **Record Management:**
  - Clickable list of passwords to view, edit, or copy credentials.
  - Tag management for organized storage.
  - Drag & drop for reordering or categorizing.

- **Keyboard Shortcuts & CLI Commands:** Navigate and manage vaults efficiently without leaving the keyboard.

---

## Security Model

- The **master password** is the single root of trust. Vaults are always encrypted with a key derived from the master password using Argon2id.
- Protects against **offline attacks**: vaults are resistant to brute-force via Argon2id + AEAD.
- Assumes the user’s system is not already compromised by malware (no password manager can protect against active host compromise).
- Optional **2FA** (e.g., TOTP or hardware tokens) can be configured as an additional check before unlocking, but never replaces the master password.
- Supports long passphrases; no arbitrary maximum length is imposed.

See [SECURITY.md](./docs/SECURITY.md) for full technical details, KDF parameters, and recovery considerations.

---

## Tech Stack

- **Frontend:** SvelteKit, Svelte 5 (Runes), TypeScript 5, Vite 6, Bun
- **Backend / Storage:** Rust with SQLite database
- **Security:** Argon2id, ChaCha20-Poly1305 / AES-256-GCM

---

## Installation & Development

### Dev Mode

```bash
bun tauri dev
```

### Build for Release

```bash
bun tauri build
```

This will generate platform-specific binaries for distribution.

---

### License

Pulsar is open-source under a permissive license: you are free to use, modify, and distribute it as long as the original creators are credited.
