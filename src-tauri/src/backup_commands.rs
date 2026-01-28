use crate::auth::verify_master_password_internal;
use crate::encryption::{decrypt, decrypt_bytes, encrypt, encrypt_bytes};
use crate::state::AppState;
use crate::types::{ExportPayload, VaultBackupAttachment, VaultBackupSnapshot};
use crate::db::{get_password_items_impl, get_buttons_impl, get_recipient_keys_impl};
use crate::error::{Error, Result};
use chrono::Utc;
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    Key, XChaCha20Poly1305, XNonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde_json::{self, Value};
use sqlx::{Error as SqlxError, Row, SqlitePool};
use std::path::Path;
use std::path::PathBuf;
use tauri::command;
use tauri::AppHandle;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;
use zeroize::{Zeroize, Zeroizing};

async fn get_key(state: &State<'_, AppState>) -> Result<Zeroizing<Vec<u8>>> {
    let guard = state.key.lock().await;
    guard.clone().ok_or(Error::VaultLocked)
}

async fn get_db_pool(state: &State<'_, AppState>) -> Result<SqlitePool> {
    let guard = state.db.lock().await;
    guard.clone().ok_or(Error::VaultNotLoaded)
}

async fn get_attachments_snapshot(db_pool: &SqlitePool, key: &[u8]) -> Result<Vec<VaultBackupAttachment>> {
    let rows = sqlx::query(
        "SELECT id, item_id, file_name, file_size, mime_type, data, created_at FROM attachments",
    )
    .fetch_all(db_pool)
    .await?;

    let mut attachments = Vec::with_capacity(rows.len());
    for row in rows {
        let name_enc: String = row.get("file_name");
        let mime_enc: String = row.get("mime_type");
        let data_blob: Vec<u8> = row.get("data");

        let data = decrypt_bytes(&data_blob, key)?;
        let data_b64 = general_purpose::STANDARD.encode(&data);

        attachments.push(VaultBackupAttachment {
            id: row.get("id"),
            item_id: row.get("item_id"),
            file_name: decrypt(&name_enc, key)?,
            file_size: row.get("file_size"),
            mime_type: decrypt(&mime_enc, key)?,
            created_at: row.get("created_at"),
            data_b64,
        });
    }

    Ok(attachments)
}

async fn write_sensitive_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp_path = path.with_extension("tmp");
    if tokio::fs::try_exists(&tmp_path).await.unwrap_or(false) {
        let _ = tokio::fs::remove_file(&tmp_path).await;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    #[cfg(windows)]
    {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }
    
    #[cfg(not(any(unix, windows)))]
    {
         let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    if let Err(err) = tokio::fs::rename(&tmp_path, path).await {
        if err.kind() == std::io::ErrorKind::Other || err.raw_os_error() == Some(17) || err.raw_os_error() == Some(18) {
            tokio::fs::copy(&tmp_path, path).await?;
            let _ = tokio::fs::remove_file(&tmp_path).await;
        } else {
            return Err(Error::Io(err));
        }
    }
    Ok(())
}

#[command]
pub async fn export_vault_backend(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    passphrase: Option<String>,
    is_plaintext: Option<bool>,
    destination: Option<String>,
    reauth_password: Option<String>,
) -> Result<String> {
    let is_plaintext = is_plaintext.unwrap_or(false);
    if is_plaintext && !cfg!(debug_assertions) {
        return Err(Error::Validation(
            "Plaintext export is disabled in production builds.".to_string(),
        ));
    }
    let reauth_password = Zeroizing::new(reauth_password.unwrap_or_default());
    let reauth_ok = verify_master_password_internal(&state, reauth_password.as_str()).await?;
    if !reauth_ok {
        return Err(Error::InvalidPassword);
    }
    let passphrase_value = Zeroizing::new(passphrase.unwrap_or_default());

    if !is_plaintext && passphrase_value.is_empty() {
        return Err(Error::Validation("A passphrase is required to export the vault.".to_string()));
    }

    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let password_items = get_password_items_impl(&db_pool, key.as_slice()).await?;
    let buttons = get_buttons_impl(&db_pool, key.as_slice()).await?;
    let recipient_keys = get_recipient_keys_impl(&db_pool, key.as_slice()).await?;
    let attachments = get_attachments_snapshot(&db_pool, key.as_slice()).await?;

    let snapshot = VaultBackupSnapshot {
        version: 1,
        exported_at: Utc::now().to_rfc3339(),
        password_items,
        buttons,
        recipient_keys,
        attachments,
    };

    let vault_data = serde_json::to_string(&snapshot)?;

    let file_extension = if is_plaintext { "json" } else { "pulsar" };
    let default_file_name = format!("vault_backup.{}", file_extension);
    let path = if let Some(destination) = destination {
        PathBuf::from(destination)
    } else {
        let (tx, rx) = oneshot::channel();
        app_handle
            .dialog()
            .file()
            .set_file_name(&default_file_name)
            .add_filter("Pulsar Vault", &[file_extension])
            .save_file(move |file_path| {
                let _ = tx.send(file_path);
            });

        let path_option = rx.await.map_err(|e| Error::Internal(e.to_string()))?;
        match path_option {
            Some(p) => p.into_path().map_err(|e| Error::Internal(e.to_string()))?,
            None => return Err(Error::Internal("File save dialog was cancelled.".into())),
        }
    };

    if is_plaintext {
        let pretty_bytes = serde_json::to_vec_pretty(&snapshot)?;
        write_sensitive_bytes(&path, &pretty_bytes).await?;
        return Ok(format!("Vault exported successfully to {}", path.display()));
    }

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let salt_clone = salt.to_vec();
    let passphrase_clone = passphrase_value.clone();
    
    let export_key = tauri::async_runtime::spawn_blocking(move || {
        let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| Error::Internal(e.to_string()))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(passphrase_clone.as_bytes(), &salt_clone, &mut key)
            .map_err(|e| Error::Internal(format!("KDF failed: {}", e)))?;
        Ok::<[u8; 32], Error>(key)
    })
    .await
    .map_err(|e| Error::Internal(format!("Runtime error: {}", e)))??;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&export_key));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), vault_data.as_bytes())
        .map_err(|e| Error::Encryption(format!("Encryption failed: {}", e)))?;

    let export = ExportPayload {
        version: 2,
        salt_b64: general_purpose::STANDARD.encode(salt),
        nonce_b64: general_purpose::STANDARD.encode(nonce),
        ciphertext_b64: general_purpose::STANDARD.encode(&ciphertext),
    };

    let export_bytes = serde_json::to_vec_pretty(&export)?;

    write_sensitive_bytes(&path, &export_bytes).await?;

    let _ = crate::db::activity::log_activity_impl(
        &db_pool,
        key.as_slice(),
        "vault_exported",
        None,
        None,
        Some(&format!("Vault exported to {}", path.display())),
    ).await;

    Ok(format!("Vault exported successfully to {}", path.display()))
}

#[command]
pub async fn export_vault(
    app_handle: AppHandle,
    vault_data: String,
    passphrase: Option<String>,
    is_plaintext: Option<bool>,
    destination: Option<String>,
) -> Result<String> {
    let is_plaintext = is_plaintext.unwrap_or(false);
    if is_plaintext && !cfg!(debug_assertions) {
        return Err(Error::Validation(
            "Plaintext export is disabled in production builds.".to_string(),
        ));
    }
    let passphrase_value = Zeroizing::new(passphrase.unwrap_or_default());

    if !is_plaintext && passphrase_value.is_empty() {
        return Err(Error::Validation("A passphrase is required to export the vault.".to_string()));
    }

    let file_extension = if is_plaintext { "json" } else { "pulsar" };
    let default_file_name = format!("vault_backup.{file_extension}");

    let path = if let Some(destination) = destination {
        PathBuf::from(destination)
    } else {
        let (tx, rx) = oneshot::channel();
        app_handle
            .dialog()
            .file()
            .set_file_name(&default_file_name)
            .add_filter("Pulsar Vault", &[file_extension])
            .save_file(move |file_path| {
                let _ = tx.send(file_path);
            });

        let path_option = rx.await.map_err(|e| Error::Internal(e.to_string()))?;
        match path_option {
            Some(p) => p.into_path().map_err(|e| Error::Internal(e.to_string()))?,
            None => return Err(Error::Internal("File save dialog was cancelled.".into())),
        }
    };

    if is_plaintext {
        let pretty_bytes = match serde_json::from_str::<Value>(&vault_data) {
            Ok(value) => {
                serde_json::to_vec_pretty(&value).unwrap_or_else(|_| vault_data.as_bytes().to_vec())
            }
            Err(_) => vault_data.as_bytes().to_vec(),
        };

        write_sensitive_bytes(&path, &pretty_bytes).await?;
        return Ok(format!("Vault exported successfully to {}", path.display()));
    }

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let salt_clone = salt.to_vec();
    let passphrase_clone = passphrase_value.clone();

    let mut key = tauri::async_runtime::spawn_blocking(move || {
        let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| Error::Internal(e.to_string()))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(passphrase_clone.as_bytes(), &salt_clone, &mut key)
            .map_err(|e| Error::Internal(format!("KDF failed: {e}")))?;
        Ok::<[u8; 32], Error>(key)
    })
    .await
    .map_err(|e| Error::Internal(format!("Runtime error: {}", e)))??;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), vault_data.as_bytes())
        .map_err(|e| Error::Encryption(format!("Encryption failed: {e}")))?;

    key.zeroize();

    let export = ExportPayload {
        version: 2,
        salt_b64: general_purpose::STANDARD.encode(salt),
        nonce_b64: general_purpose::STANDARD.encode(nonce),
        ciphertext_b64: general_purpose::STANDARD.encode(&ciphertext),
    };

    let export_bytes = serde_json::to_vec_pretty(&export)?;

    write_sensitive_bytes(&path, &export_bytes).await?;
    Ok(format!("Vault exported successfully to {}", path.display()))
}

#[command]
pub async fn import_vault(
    app_handle: AppHandle,
    passphrase: Option<String>,
    path: Option<String>,
) -> Result<String> {
    let passphrase_value = Zeroizing::new(passphrase.unwrap_or_default());
    if passphrase_value.is_empty() {
        return Err(Error::Validation("A passphrase is required to import the vault.".to_string()));
    }

    let path = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        let (tx, rx) = oneshot::channel();
        app_handle
            .dialog()
            .file()
            .add_filter("Pulsar Vault", &["pulsar"])
            .add_filter("JSON", &["json"])
            .pick_file(move |file_path| {
                let _ = tx.send(file_path);
            });

        let path_option = rx.await.map_err(|e| Error::Internal(e.to_string()))?;
        match path_option {
            Some(p) => p.into_path().map_err(|e| Error::Internal(e.to_string()))?,
            None => return Err(Error::Internal("File open dialog was cancelled.".into())),
        }
    };

    if path.extension().and_then(|s| s.to_str()) == Some("json") && !cfg!(debug_assertions) {
        return Err(Error::Validation(
            "Plaintext backups are not supported in production builds.".to_string(),
        ));
    }

    let file_content_bytes = tokio::fs::read(&path).await?;

    let payload: ExportPayload = serde_json::from_slice(&file_content_bytes).map_err(|_| {
        Error::Validation("Failed to parse backup file. It might be invalid or not a Pulsar backup.".to_string())
    })?;

    let salt = general_purpose::STANDARD
        .decode(payload.salt_b64)
        .map_err(|e| Error::Internal(e.to_string()))?;
    let nonce = general_purpose::STANDARD
        .decode(payload.nonce_b64)
        .map_err(|e| Error::Internal(e.to_string()))?;
    let ciphertext = general_purpose::STANDARD
        .decode(payload.ciphertext_b64)
        .map_err(|e| Error::Internal(e.to_string()))?;

    let salt_clone = salt.clone();
    let passphrase_clone = passphrase_value.clone();

    let mut key = tauri::async_runtime::spawn_blocking(move || {
        let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| Error::Internal(e.to_string()))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(passphrase_clone.as_bytes(), &salt_clone, &mut key)
            .map_err(|e| Error::Internal(format!("KDF failed: {e}")))?;
        Ok::<[u8; 32], Error>(key)
    })
    .await
    .map_err(|e| Error::Internal(format!("Runtime error: {}", e)))??;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let decrypted_bytes = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| {
            Error::Validation("Decryption failed. The password may be incorrect or the file is corrupt.".to_string())
        })?;

    key.zeroize();

    String::from_utf8(decrypted_bytes).map_err(|e| Error::Internal(format!("UTF-8 conversion failed: {e}")))
}

#[command]
pub async fn restore_vault_backend(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    passphrase: Option<String>,
    path: Option<String>,
    reauth_password: Option<String>,
) -> Result<VaultBackupSnapshot> {
    let passphrase_value = Zeroizing::new(passphrase.unwrap_or_default());
    if passphrase_value.is_empty() {
        return Err(Error::Validation("A passphrase is required to import the vault.".to_string()));
    }
    let reauth_password = Zeroizing::new(reauth_password.unwrap_or_default());
    let reauth_ok = verify_master_password_internal(&state, reauth_password.as_str()).await?;
    if !reauth_ok {
        return Err(Error::InvalidPassword);
    }

    let path = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        let (tx, rx) = oneshot::channel();
        app_handle
            .dialog()
            .file()
            .add_filter("Pulsar Vault", &["pulsar"])
            .add_filter("JSON", &["json"])
            .pick_file(move |file_path| {
                let _ = tx.send(file_path);
            });

        let path_option = rx.await.map_err(|e| Error::Internal(e.to_string()))?;
        match path_option {
            Some(p) => p.into_path().map_err(|e| Error::Internal(e.to_string()))?,
            None => return Err(Error::Internal("File open dialog was cancelled.".into())),
        }
    };

    let file_content_bytes = tokio::fs::read(&path).await?;

    let decrypted_json = if path.extension().and_then(|s| s.to_str()) == Some("json") {
        if !cfg!(debug_assertions) {
            return Err(Error::Validation(
                "Plaintext backups are not supported in production builds.".to_string(),
            ));
        }
        String::from_utf8(file_content_bytes).map_err(|e| Error::Internal(format!("UTF-8 conversion failed: {e}")))?
    } else {
        let payload: ExportPayload = serde_json::from_slice(&file_content_bytes).map_err(|_| {
            Error::Validation("Failed to parse backup file. It might be invalid or not a Pulsar backup.".to_string())
        })?;

        let salt = general_purpose::STANDARD
            .decode(payload.salt_b64)
            .map_err(|e| Error::Internal(e.to_string()))?;
        let nonce = general_purpose::STANDARD
            .decode(payload.nonce_b64)
            .map_err(|e| Error::Internal(e.to_string()))?;
        let ciphertext = general_purpose::STANDARD
            .decode(payload.ciphertext_b64)
            .map_err(|e| Error::Internal(e.to_string()))?;

        let salt_clone = salt.clone();
        let passphrase_clone = passphrase_value.clone();

        let mut key = tauri::async_runtime::spawn_blocking(move || {
            let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| Error::Internal(e.to_string()))?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let mut key = [0u8; 32];
                    argon2
                        .hash_password_into(passphrase_clone.as_bytes(), &salt_clone, &mut key)
                        .map_err(|e| Error::Internal(format!("KDF failed: {e}")))?;
                    Ok::<[u8; 32], Error>(key)
                })
                .await
            
        .map_err(|e| Error::Internal(format!("Runtime error: {}", e)))??;

        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let decrypted_bytes = cipher
            .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
            .map_err(|_| {
                Error::Validation("Decryption failed. The password may be incorrect or the file is corrupt.".to_string())
            })?;

        key.zeroize();
        String::from_utf8(decrypted_bytes).map_err(|e| Error::Internal(format!("UTF-8 conversion failed: {e}")))?
    };

    let snapshot: VaultBackupSnapshot = serde_json::from_str(&decrypted_json)?;

    restore_vault_snapshot(state.clone(), snapshot.clone()).await?;

    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let _ = crate::db::activity::log_activity_impl(
        &db_pool,
        key.as_slice(),
        "vault_restored",
        None,
        None,
        Some("Vault data was restored from a backup"),
    ).await;

    Ok(snapshot)
}

#[command]
pub async fn restore_vault_snapshot(
    state: State<'_, AppState>,
    snapshot: VaultBackupSnapshot,
) -> Result<()> {
    if snapshot.version != 1 {
        return Err(Error::Validation(format!(
            "Unsupported backup version {}. Please upgrade Pulsar to restore this backup.",
            snapshot.version
        )));
    }

    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let mut tx = db_pool.begin().await?;

    sqlx::query("DELETE FROM password_items")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM buttons")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM recipient_keys")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM attachments")
        .execute(&mut *tx)
        .await?;
    if let Err(e) = sqlx::query(
        "DELETE FROM sqlite_sequence WHERE name IN ('password_items', 'buttons', 'recipient_keys', 'attachments')",
    )
    .execute(&mut *tx)
    .await
    {
        match &e {
            SqlxError::Database(db_err) if db_err.message().contains("no such table") => {}
            _ => return Err(Error::Database(e)),
        }
    }

    for item in &snapshot.password_items {
        let title_enc = encrypt(&item.title, key.as_slice())?;
        let description_enc = item
            .description
            .as_deref()
            .map(|value| encrypt(value, key.as_slice()))
            .transpose()?;
        let img_enc = item
            .img
            .as_deref()
            .map(|value| encrypt(value, key.as_slice()))
            .transpose()?;
        let tags_enc = item
            .tags
            .as_deref()
            .map(|value| encrypt(value, key.as_slice()))
            .transpose()?;
        let username_enc = item
            .username
            .as_deref()
            .map(|value| encrypt(value, key.as_slice()))
            .transpose()?;
        let url_enc = item
            .url
            .as_deref()
            .map(|value| encrypt(value, key.as_slice()))
            .transpose()?;
        let notes_enc = item
            .notes
            .as_deref()
            .map(|value| encrypt(value, key.as_slice()))
            .transpose()?;
        let password_enc = encrypt(item.password.as_str(), key.as_slice())?;
        let totp_secret_enc = item
            .totp_secret
            .as_ref()
            .map(|value| encrypt(value.as_str(), key.as_slice()))
            .transpose()?;
        let custom_fields_json =
            serde_json::to_string(&item.custom_fields)?;
        let custom_fields_enc = encrypt(&custom_fields_json, key.as_slice())?;
        let field_order_json = item
            .field_order
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let field_order_enc = field_order_json
            .map(|value| encrypt(&value, key.as_slice()))
            .transpose()?;

        let category_enc = encrypt(&item.category, key.as_slice())?;

        sqlx::query("INSERT INTO password_items (id, category, title, description, img, tags, username, url, notes, password, created_at, updated_at, color, totp_secret, custom_fields, field_order) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(item.id)
            .bind(category_enc)
            .bind(title_enc)
            .bind(description_enc)
            .bind(img_enc)
            .bind(tags_enc)
            .bind(username_enc)
            .bind(url_enc)
            .bind(notes_enc)
            .bind(password_enc)
            .bind(&item.created_at)
            .bind(&item.updated_at)
            .bind(item.color.clone())
            .bind(totp_secret_enc)
            .bind(custom_fields_enc)
            .bind(field_order_enc)
            .execute(&mut *tx)
            .await?;
    }

    for button in &snapshot.buttons {
        let text_enc = encrypt(&button.text, key.as_slice())?;
        let icon_enc = encrypt(&button.icon, key.as_slice())?;
        let color_enc = encrypt(&button.color, key.as_slice())?;

        sqlx::query("INSERT INTO buttons (id, text, icon, color) VALUES (?, ?, ?, ?)")
            .bind(button.id)
            .bind(text_enc)
            .bind(icon_enc)
            .bind(color_enc)
            .execute(&mut *tx)
            .await?;
    }

    for recipient in &snapshot.recipient_keys {
        let name_enc = encrypt(&recipient.name, key.as_slice())?;
        let public_key_enc = encrypt(&recipient.public_key, key.as_slice())?;
        let private_key_enc = encrypt(&recipient.private_key, key.as_slice())?;

        sqlx::query("INSERT INTO recipient_keys (id, name, public_key, private_key) VALUES (?, ?, ?, ?)")
            .bind(recipient.id)
            .bind(name_enc)
            .bind(public_key_enc)
            .bind(private_key_enc)
            .execute(&mut *tx)
            .await?;
    }

    for attachment in &snapshot.attachments {
        let file_name_enc = encrypt(&attachment.file_name, key.as_slice())?;
        let mime_type_enc = encrypt(&attachment.mime_type, key.as_slice())?;
        let data = general_purpose::STANDARD
            .decode(&attachment.data_b64)
            .map_err(|e| Error::Internal(format!("Invalid attachment data: {}", e)))?;
        let data_enc = encrypt_bytes(&data, key.as_slice())?;

        sqlx::query("INSERT INTO attachments (id, item_id, file_name, file_size, mime_type, data, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(attachment.id)
            .bind(attachment.item_id)
            .bind(file_name_enc)
            .bind(attachment.file_size)
            .bind(mime_type_enc)
            .bind(data_enc)
            .bind(&attachment.created_at)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

