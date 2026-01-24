# Pulsar Security Overview

This document describes Pulsar’s cryptographic design, authentication practices, and threat model. It is intended for developers, auditors, and security reviewers.

---

## Threat Model

- **Protected against:**
  - Offline attackers with access to the encrypted vault file.
  - Brute-force attempts against weak passwords (mitigated by Argon2id).
  - File tampering (AEAD integrity checks).

- **Not protected against:**
  - Malware/keyloggers on the host system (able to capture master password or 2FA codes).
  - Physical compromise of both the vault device and the user’s second-factor device simultaneously.
  - Attacks exploiting outdated OS or runtime vulnerabilities.

---

## Vault Encryption

1. **Master Password → Vault Key**
   - Argon2id with per-vault random salt (≥16 bytes).
   - Recommended defaults: memory ≥64 MiB, iterations ≥3, parallelism = 1 (tunable by platform).
   - Output: 256-bit vault key.

2. **Database Encryption**
   - AEAD: ChaCha20-Poly1305 (default).
   - AES-256-GCM if AES-NI available.
   - Each record encrypted individually with a 12–24 byte random nonce.
   - Associated Data (AAD): vault header (version, salt, KDF params) to ensure integrity.

3. **Vault Header**
   - Stores: version, KDF parameters, random salt, algorithm tag.
   - Enables future upgrades without re-encrypting the entire vault.

---

## Authentication Practices

- **Master Password:**
  - Always required. Root of trust for vault security.
  - Supports long passphrases; no arbitrary limits.
  - Users encouraged to select high-entropy passphrases (e.g., ≥128 bits).

- **Two-Factor Authentication (2FA):**
  - Optional TOTP or hardware-backed keys can be enabled.
  - 2FA is used as a \*secondary check\* during unlock.
  - Vault encryption key is \*\*never derived from TOTP\*\* (to avoid low-entropy exposure).
  - TOTP seeds are external (e.g., stored on a phone authenticator app) and not stored in the vault.

---

## Key Handling

- Sensitive keys are zeroed from memory as soon as possible.
- Secure memory APIs used where available.
- Clipboard contents cleared automatically after short timeouts.

---

## Recovery

- Vaults can be backed up safely since they remain encrypted.
- Optional recovery codes are generated during vault setup and should be stored securely offline.
- For advanced users: Shamir’s Secret Sharing may be configured to split recovery material across multiple trusted parties.

---

## Future Enhancements

- Optional TPM / Secure Enclave integration for device-bound keys.
- Metadata authentication with versioned headers for cryptographic agility.
- FIDO2/WebAuthn hardware key support for phishing-resistant unlocks.
