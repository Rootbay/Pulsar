pub mod types;
pub mod crypto_utils;
pub mod metadata;
pub mod biometrics;
pub mod commands;

use std::path::{Path, PathBuf};
use tauri::State;
use crate::state::AppState;
use crate::error::{Error, Result};
use sqlx::Row;
use crate::auth::types::{PasswordMetadata};
use zeroize::Zeroize;

pub use types::*;
pub use crypto_utils::*;
pub use metadata::*;
pub use commands::*;

pub async fn get_db_pool(_state: &State<'_, AppState>) -> Result<sqlx::SqlitePool> {
    let guard = _state.db.lock().await;
    guard
        .clone()
        .ok_or(Error::VaultNotLoaded)
}

pub async fn get_db_path(_state: &State<'_, AppState>) -> Result<PathBuf> {
    _state
        .db_path
        .lock()
        .await
        .clone()
        .ok_or(Error::Internal("Database path is not set. Select a vault first.".to_string()))
}

pub async fn load_metadata_from_db(
    db_pool: &sqlx::SqlitePool,
) -> Result<Option<PasswordMetadata>> {
    let salt_b64: Option<String> = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_salt")
        .fetch_optional(db_pool)
        .await?
        .map(|row| row.get("value"));

    let Some(salt_b64) = salt_b64 else {
        return Ok(None);
    };

    let nonce_b64: String = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_check_nonce")
        .fetch_one(db_pool)
        .await?
        .get("value");

    let ciphertext_b64: String = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_check_ciphertext")
        .fetch_one(db_pool)
        .await?
        .get("value");

    let argon2_memory_kib: Option<u32> =
        sqlx::query("SELECT value FROM configuration WHERE key = ?")
            .bind("argon2_memory_kib")
            .fetch_optional(db_pool)
            .await?
            .and_then(|row| {
                let value: String = row.get("value");
                value.parse::<u32>().ok()
            });

    let argon2_time_cost: Option<u32> =
        sqlx::query("SELECT value FROM configuration WHERE key = ?")
            .bind("argon2_time_cost")
            .fetch_optional(db_pool)
            .await?
            .and_then(|row| {
                let value: String = row.get("value");
                value.parse::<u32>().ok()
            });

    let argon2_parallelism: Option<u32> =
        sqlx::query("SELECT value FROM configuration WHERE key = ?")
            .bind("argon2_parallelism")
            .fetch_optional(db_pool)
            .await?
            .and_then(|row| {
                let value: String = row.get("value");
                value.parse::<u32>().ok()
            });

    Ok(Some(PasswordMetadata {
        version: 1,
        salt_b64,
        nonce_b64,
        ciphertext_b64,
        argon2_memory_kib,
        argon2_time_cost,
        argon2_parallelism,
        mac_version: None,
        mac_nonce_b64: None,
        mac_tag_b64: None,
    }))
}

pub async fn load_existing_metadata(
    _state: &State<'_, AppState>,
    db_pool: &sqlx::SqlitePool,
    db_path: &Path,
) -> Result<PasswordMetadata> {
    let metadata = match read_password_metadata(db_path).await? {
        Some(meta) => Some(meta),
        None => load_metadata_from_db(db_pool).await?,
    };

    metadata.ok_or_else(|| Error::Internal("Vault is not initialised with a master password.".to_string()))
}

pub async fn verify_master_password_internal(
    state: &State<'_, AppState>,
    password: &str,
) -> Result<bool> {
    use zeroize::Zeroizing;
    use chacha20poly1305::{aead::{Aead, KeyInit}, Key, XChaCha20Poly1305, XNonce};
    use subtle::ConstantTimeEq;

    const PASSWORD_CHECK_PLAINTEXT: &[u8] = b"pulsar-password-check";

    if password.trim().is_empty() {
        return Err(Error::Validation("Master password is required.".to_string()));
    }

    let db_path = get_db_path(state).await?;
    let db_pool = get_db_pool(state).await?;
    let metadata = load_existing_metadata(state, &db_pool, &db_path).await?;
    let (salt, nonce, ciphertext) = decode_metadata(&metadata)?;
    let argon_params = metadata.argon2_params();

    let mut derived_key = derive_key(password, &salt, &argon_params)?;
    let key_z = Zeroizing::new(derived_key.to_vec());
    derived_key.zeroize();

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));
    let mut decrypted = match cipher.decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref()) {
        Ok(value) => value,
        Err(_) => return Ok(false),
    };

    let is_valid = decrypted.ct_eq(PASSWORD_CHECK_PLAINTEXT).unwrap_u8() == 1;
    decrypted.zeroize();
    Ok(is_valid)
}
