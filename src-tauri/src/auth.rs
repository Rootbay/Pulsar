use crate::encryption::{decrypt, encrypt};
use crate::state::AppState;
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{aead::{Aead, KeyInit}, Key, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{Connection, Row};
use std::path::{Path, PathBuf};
use tauri::State;
use tokio::fs;
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
use zeroize::{Zeroize, Zeroizing};

const PASSWORD_CHECK_PLAINTEXT: &[u8] = b"pulsar-password-check";

#[derive(Debug, Serialize, Deserialize)]
struct PasswordMetadata {
    version: u8,
    salt_b64: String,
    nonce_b64: String,
    ciphertext_b64: String,
}

#[derive(Serialize)]
pub struct UnlockResponse {
    pub totp_required: bool,
}

async fn get_db_pool(state: &State<'_, AppState>) -> Result<sqlx::SqlitePool, String> {
    let guard = state.db.lock().await;
    guard.clone().ok_or_else(|| "Database not loaded".to_string())
}

fn metadata_path(db_path: &Path) -> PathBuf {
    let file_name = db_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("vault.db");
    let meta_name = format!("{}.meta.json", file_name);
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(meta_name)
}

async fn read_password_metadata(db_path: &Path) -> Result<Option<PasswordMetadata>, String> {
    let path = metadata_path(db_path);
    match fs::read(&path).await {
        Ok(bytes) => {
            let meta: PasswordMetadata = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
            Ok(Some(meta))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("Failed to read {}: {}", path.display(), err)),
    }
}

async fn write_password_metadata(db_path: &Path, meta: &PasswordMetadata) -> Result<(), String> {
    let path = metadata_path(db_path);
    let bytes = serde_json::to_vec_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(&path, bytes)
        .await
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

fn decode_metadata(meta: &PasswordMetadata) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
    let salt = general_purpose::STANDARD
        .decode(&meta.salt_b64)
        .map_err(|e| format!("Invalid salt encoding: {}", e))?;
    let nonce = general_purpose::STANDARD
        .decode(&meta.nonce_b64)
        .map_err(|e| format!("Invalid nonce encoding: {}", e))?;
    let ciphertext = general_purpose::STANDARD
        .decode(&meta.ciphertext_b64)
        .map_err(|e| format!("Invalid ciphertext encoding: {}", e))?;
    Ok((salt, nonce, ciphertext))
}

async fn load_metadata_from_db(db_pool: &sqlx::SqlitePool) -> Result<Option<PasswordMetadata>, String> {
    let salt_b64: Option<String> = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_salt")
        .fetch_optional(db_pool)
        .await
        .map_err(|e| e.to_string())?
        .map(|row| row.get("value"));

    let Some(salt_b64) = salt_b64 else {
        return Ok(None);
    };

    let nonce_b64: String = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_check_nonce")
        .fetch_one(db_pool)
        .await
        .map_err(|e| e.to_string())?
        .get("value");

    let ciphertext_b64: String = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_check_ciphertext")
        .fetch_one(db_pool)
        .await
        .map_err(|e| e.to_string())?
        .get("value");

    Ok(Some(PasswordMetadata {
        version: 1,
        salt_b64,
        nonce_b64,
        ciphertext_b64,
    }))
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(64 * 1024, 3, 1, None).unwrap())
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Failed to derive key: {}", e))?;
    Ok(key)
}

async fn get_db_path(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    state
        .db_path
        .lock()
        .await
        .clone()
        .ok_or_else(|| "Database path is not set. Select a vault first.".to_string())
}

async fn apply_key_to_conn(conn: &mut SqliteConnection, key_bytes: &[u8]) -> Result<(), String> {
    let param_try = sqlx::query("PRAGMA key = ?;")
        .bind(key_bytes)
        .execute(&mut *conn)
        .await;

    if let Err(e) = param_try {
        let msg = e.to_string();
        if msg.contains("near \"?\": syntax error") || msg.contains("syntax error") {
            let hex_key: String = key_bytes.iter().map(|b| format!("{:02x}", b)).collect();
            let pragma_unquoted = format!("PRAGMA key = x'{}';", hex_key);
            let unq_try = sqlx::query(&pragma_unquoted).execute(&mut *conn).await;
            if unq_try.is_err() {
                let pragma_quoted = format!("PRAGMA key = \"x'{}'\";", hex_key);
                sqlx::query(&pragma_quoted)
                    .execute(&mut *conn)
                    .await
                    .map_err(|_| "Failed to apply pragma key using fallback method.".to_string())?;
            }
        } else {
            return Err(format!("Failed to apply pragma key: {}", e));
        }
    }

    Ok(())
}

async fn finalize_unlock(state: &State<'_, AppState>, key_z: Zeroizing<Vec<u8>>) -> Result<(), String> {
    let db_path = get_db_path(state).await?;

    {
        if let Some(pool) = state.db.lock().await.take() {
            pool.close().await;
        }
    }

    let new_pool = crate::db::init_db(db_path.as_path(), Some(key_z.as_slice())).await?;

    {
        let mut db_guard = state.db.lock().await;
        *db_guard = Some(new_pool);
    }

    {
        let mut key_guard = state.key.lock().await;
        *key_guard = Some(key_z.clone());
    }

    {
        let mut pending_guard = state.pending_key.lock().await;
        *pending_guard = None;
    }

    Ok(())
}

#[tauri::command]
pub async fn set_master_password(state: State<'_, AppState>, password: String) -> Result<(), String> {
    let db_pool = get_db_pool(&state).await?;
    let db_path = get_db_path(&state).await?;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let mut derived_key = derive_key(&password, &salt)?;
    let key_z = Zeroizing::new(derived_key.to_vec());

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), PASSWORD_CHECK_PLAINTEXT)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let salt_b64 = general_purpose::STANDARD.encode(&salt);
    let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
    let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);

    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_salt")
        .bind(&salt_b64)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_check_nonce")
        .bind(&nonce_b64)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_check_ciphertext")
        .bind(&ciphertext_b64)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    let metadata = PasswordMetadata {
        version: 1,
        salt_b64: salt_b64.clone(),
        nonce_b64: nonce_b64.clone(),
        ciphertext_b64: ciphertext_b64.clone(),
    };
    write_password_metadata(db_path.as_path(), &metadata).await?;

    {
        let mut path_guard = state.db_path.lock().await;
        *path_guard = Some(db_path.clone());
    }

    {
        let mut key_guard = state.key.lock().await;
        *key_guard = Some(key_z.clone());
    }

    let _rekey_lock = state.rekey.lock().await;
    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false);
    let mut conn = SqliteConnection::connect_with(&connect_options)
        .await
        .map_err(|e| format!("Failed to open exclusive connection for rekey: {}", e))?;

    apply_key_to_conn(&mut conn, key_z.as_slice()).await?;

    let rekey_try = sqlx::query("PRAGMA rekey = ?;")
        .bind(key_z.as_slice())
        .execute(&mut conn)
        .await;

    if let Err(e) = rekey_try {
        let msg = e.to_string();
        if msg.contains("near \"?\": syntax error") || msg.contains("syntax error") {
            let hex_key: String = key_z.as_slice().iter().map(|b| format!("{:02x}", b)).collect();
            let pragma_unquoted = format!("PRAGMA rekey = x'{}';", hex_key);
            let unq_try = sqlx::query(&pragma_unquoted).execute(&mut conn).await;
            if unq_try.is_err() {
                let pragma_quoted = format!("PRAGMA rekey = \"x'{}'\";", hex_key);
                sqlx::query(&pragma_quoted)
                    .execute(&mut conn)
                    .await
                    .map_err(|_| "Failed to rekey database using fallback method.".to_string())?;
            }
        } else {
            return Err(format!("Failed to rekey database: {}", e));
        }
    }

    drop(conn);

    finalize_unlock(&state, key_z.clone()).await?;
    derived_key.zeroize();

    Ok(())
}

#[tauri::command]
pub async fn unlock(state: State<'_, AppState>, password: String) -> Result<UnlockResponse, String> {
    let _rekey_lock = state.rekey.lock().await;
    let db_path = get_db_path(&state).await?;

    let metadata = match read_password_metadata(db_path.as_path()).await? {
        Some(meta) => Some(meta),
        None => {
            let pool = get_db_pool(&state).await?;
            load_metadata_from_db(&pool).await?
        }
    };

    let meta = metadata.ok_or_else(|| "Vault is not initialised with a master password.".to_string())?;
    let (salt, nonce, ciphertext) = decode_metadata(&meta)?;

    let mut derived_key = derive_key(&password, &salt)?;
    let key_z = Zeroizing::new(derived_key.to_vec());
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));

    let decrypted = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| "Invalid password".to_string())?;

    if decrypted != PASSWORD_CHECK_PLAINTEXT {
        return Err("Invalid password".to_string());
    }

    // Ensure metadata file exists for legacy databases.
    if read_password_metadata(db_path.as_path()).await?.is_none() {
        write_password_metadata(
            db_path.as_path(),
            &PasswordMetadata {
                version: meta.version,
                salt_b64: general_purpose::STANDARD.encode(&salt),
                nonce_b64: general_purpose::STANDARD.encode(&nonce),
                ciphertext_b64: general_purpose::STANDARD.encode(&ciphertext),
            },
        )
        .await?;
    }

    // Determine whether login TOTP is configured by opening a temporary connection with the derived key.
    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false);
    let mut conn = SqliteConnection::connect_with(&connect_options)
        .await
        .map_err(|e| format!("Failed to open database for verification: {}", e))?;

    apply_key_to_conn(&mut conn, key_z.as_slice()).await?;

    let totp_configured: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM configuration WHERE key = 'login_totp_secret'",
    )
    .fetch_one(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    drop(conn);

    let totp_required = totp_configured > 0;

    if totp_required {
        {
            let mut pending_guard = state.pending_key.lock().await;
            *pending_guard = Some(key_z.clone());
        }
        derived_key.zeroize();
        Ok(UnlockResponse { totp_required: true })
    } else {
        finalize_unlock(&state, key_z.clone()).await?;
        derived_key.zeroize();
        Ok(UnlockResponse { totp_required: false })
    }
}

#[tauri::command]
pub async fn verify_login_totp(state: State<'_, AppState>, token: String) -> Result<(), String> {
    let pending_key_opt = {
        let guard = state.pending_key.lock().await;
        guard.clone()
    };

    let pending_key = pending_key_opt.ok_or_else(|| "No pending unlock operation".to_string())?;
    let trimmed = token.trim();
    if trimmed.len() < 6 {
        return Err("Invalid TOTP token".to_string());
    }

    let db_path = get_db_path(&state).await?;
    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false);
    let mut conn = SqliteConnection::connect_with(&connect_options)
    .await
    .map_err(|e| format!("Failed to open database for TOTP verification: {}", e))?;

    apply_key_to_conn(&mut conn, pending_key.as_slice()).await?;

    let secret_enc: Option<String> = sqlx::query_scalar(
        "SELECT value FROM configuration WHERE key = 'login_totp_secret'",
    )
    .fetch_optional(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    let secret_enc = secret_enc.ok_or_else(|| "Login TOTP is not configured.".to_string())?;
    let secret_b32 = decrypt(&secret_enc, pending_key.as_slice())?;

    let secret = Secret::Encoded(secret_b32.clone());
    let secret_bytes = secret.to_bytes().map_err(|e| e.to_string())?;

    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Pulsar".to_string()),
        "vault".to_string(),
    )
    .map_err(|e| e.to_string())?;

    let is_valid = totp.check_current(trimmed).unwrap_or(false);
    if !is_valid {
        return Err("Invalid TOTP token".to_string());
    }

    drop(conn);
    finalize_unlock(&state, pending_key.clone()).await?;
    Ok(())
}

#[tauri::command]
pub async fn configure_login_totp(state: State<'_, AppState>, secret_b32: String) -> Result<(), String> {
    let key_opt = {
        let guard = state.key.lock().await;
        guard.clone()
    };

    let key_z = key_opt.ok_or_else(|| "Vault must be unlocked to configure TOTP.".to_string())?;

    // Validate TOTP secret
    Secret::Encoded(secret_b32.clone())
        .to_bytes()
        .map_err(|e| format!("Invalid TOTP secret: {}", e))?;

    let encrypted = encrypt(&secret_b32, key_z.as_slice())?;
    let db_pool = get_db_pool(&state).await?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES ('login_totp_secret', ?)")
        .bind(encrypted)
        .execute(&db_pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn disable_login_totp(state: State<'_, AppState>) -> Result<(), String> {
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("DELETE FROM configuration WHERE key = 'login_totp_secret'")
        .execute(&db_pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn is_login_totp_configured(state: State<'_, AppState>) -> Result<bool, String> {
    let db_pool = get_db_pool(&state).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM configuration WHERE key = 'login_totp_secret'")
        .fetch_one(&db_pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

#[tauri::command]
pub async fn lock(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut key_guard = state.key.lock().await;
        *key_guard = None;
    }
    {
        let mut pending = state.pending_key.lock().await;
        *pending = None;
    }
    Ok(())
}

#[tauri::command]
pub async fn is_locked(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.key.lock().await.is_none())
}

#[tauri::command]
pub async fn is_master_password_configured(state: State<'_, AppState>) -> Result<bool, String> {
    if let Ok(db_path) = get_db_path(&state).await {
        if read_password_metadata(db_path.as_path()).await?.is_some() {
            return Ok(true);
        }
    }

    let db_pool = get_db_pool(&state).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM configuration WHERE key = 'password_salt'")
        .fetch_one(&db_pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(count > 0)
}
