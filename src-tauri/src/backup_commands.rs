use tauri::command;
use std::fs;
use tauri::AppHandle;
use tokio::sync::oneshot;
use tauri_plugin_dialog::DialogExt;
use argon2::{Argon2, Algorithm, Version, Params};
use chacha20poly1305::{aead::{Aead, KeyInit}, Key, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};
use zeroize::Zeroize;
use crate::types::ExportPayload;

#[command]
pub async fn export_vault(
    app_handle: AppHandle,
    vault_data: String,
    passphrase: String,
) -> Result<String, String> {
    if passphrase.is_empty() {
        return Err("A passphrase is required to export the vault.".to_string());
    }

    let file_extension = "pulsar";
    let default_file_name = format!("vault_backup.{}", file_extension);

    let (tx, rx) = oneshot::channel();
    app_handle.dialog()
        .file()
        .set_file_name(&default_file_name)
        .add_filter("Pulsar Vault", &[file_extension])
        .save_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    let path_option = rx.await.map_err(|e| e.to_string())?;
    let path = match path_option {
        Some(p) => p.into_path().map_err(|e| e.to_string())?,
        None => return Err("File save dialog was cancelled.".into()),
    };

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| format!("KDF failed: {}", e))?;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), vault_data.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    key.zeroize();

    let export = ExportPayload {
        version: 2, // Using version 2 to signify the vault backup format
        salt_b64: general_purpose::STANDARD.encode(&salt),
        nonce_b64: general_purpose::STANDARD.encode(&nonce),
        ciphertext_b64: general_purpose::STANDARD.encode(&ciphertext),
    };

    let export_bytes = serde_json::to_vec_pretty(&export).map_err(|e| e.to_string())?;
    
    match fs::write(&path, export_bytes) {
        Ok(_) => Ok(format!("Vault exported successfully to {}", path.display())),
        Err(e) => Err(format!("Failed to write encrypted vault: {}", e)),
    }
}

#[command]
pub async fn import_vault(
    app_handle: AppHandle,
    passphrase: String,
) -> Result<String, String> {
    if passphrase.is_empty() {
        return Err("A passphrase is required to import the vault.".to_string());
    }

    let (tx, rx) = oneshot::channel();
    app_handle.dialog()
        .file()
        .add_filter("Pulsar Vault", &["pulsar"])
        .add_filter("JSON", &["json"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    let path_option = rx.await.map_err(|e| e.to_string())?;
    let path = match path_option {
        Some(p) => p.into_path().map_err(|e| e.to_string())?,
        None => return Err("File open dialog was cancelled.".into()),
    };

    let file_content_bytes = fs::read(&path)
        .map_err(|e| format!("Failed to read vault file: {}", e))?;
    
    let payload: ExportPayload = serde_json::from_slice(&file_content_bytes)
        .map_err(|_| "Failed to parse backup file. It might be invalid or not a Pulsar backup.".to_string())?;

    let salt = general_purpose::STANDARD.decode(payload.salt_b64).map_err(|e| e.to_string())?;
    let nonce = general_purpose::STANDARD.decode(payload.nonce_b64).map_err(|e| e.to_string())?;
    let ciphertext = general_purpose::STANDARD.decode(payload.ciphertext_b64).map_err(|e| e.to_string())?;

    let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| format!("KDF failed: {}", e))?;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let decrypted_bytes = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| "Decryption failed. The password may be incorrect or the file is corrupt.".to_string())?;

    key.zeroize();

    String::from_utf8(decrypted_bytes).map_err(|e| format!("UTF-8 conversion failed: {}", e))
}