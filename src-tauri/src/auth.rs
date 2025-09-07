use crate::state::AppState;
use argon2::{Argon2, Algorithm, Version, Params};
use chacha20poly1305::{aead::{Aead, KeyInit}, Key, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use tauri::State;
use base64::{engine::general_purpose, Engine as _};
use sqlx::Row;
use sqlx::Connection;

use zeroize::Zeroize;
use zeroize::Zeroizing;
use std::path::Path;

const PASSWORD_CHECK_PLAINTEXT: &[u8] = b"pulsar-password-check";

async fn get_db_pool(state: &State<'_, AppState>) -> Result<sqlx::SqlitePool, String> {
    let guard = state.db.lock().await;
    guard.clone().ok_or_else(|| "Database not loaded".to_string())
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(64 * 1024, 3, 1, None).unwrap())
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Failed to derive key: {}", e))?;
    Ok(key)
}

#[tauri::command]
pub async fn set_master_password(state: State<'_, AppState>, password: String) -> Result<(), String> {
    let db_pool = get_db_pool(&state).await?;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let mut derived_key = derive_key(&password, &salt)?;
    let key_z = Zeroizing::new(derived_key.to_vec());

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher.encrypt(XNonce::from_slice(&nonce), PASSWORD_CHECK_PLAINTEXT)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let salt_b64 = general_purpose::STANDARD.encode(&salt);
    let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
    let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);

    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_salt")
        .bind(salt_b64)
        .execute(&mut *tx)
        .await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_check_nonce")
        .bind(nonce_b64)
        .execute(&mut *tx)
        .await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_check_ciphertext")
        .bind(ciphertext_b64)
        .execute(&mut *tx)
        .await.map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    {
        let mut key_guard = state.key.lock().await;
        *key_guard = Some(key_z.clone());
    }
    derived_key.zeroize();

    {
        let param_try = sqlx::query("PRAGMA key = ?;")
            .bind(key_z.as_slice())
            .execute(&db_pool)
            .await;
        if let Err(e) = param_try {
            let msg = e.to_string();
                if msg.contains("near \"?\": syntax error") || msg.contains("syntax error") {
                let hex_key: String = key_z.as_slice().iter().map(|b| format!("{:02x}", b)).collect();
                let pragma_unquoted = format!("PRAGMA key = x'{}';", hex_key);
                let unq_try = sqlx::query(&pragma_unquoted).execute(&db_pool).await;
                if let Err(_) = unq_try {
                    let pragma_quoted = format!("PRAGMA key = \"x'{}'\";", hex_key);
                    sqlx::query(&pragma_quoted)
                        .execute(&db_pool)
                        .await
                        .map_err(|_| "Failed to apply pragma key using fallback method.".to_string())?;
                }
            } else {
                return Err(format!("Failed to apply pragma key before database_list: {}", e));
            }
        }
    }
    let rows = sqlx::query("PRAGMA database_list;")
        .fetch_all(&db_pool)
        .await
        .map_err(|e| format!("Failed to query database_list: {}", e))?;
    let mut db_file_opt: Option<String> = None;
    for row in rows {
        let name: String = row.get("name");
        if name == "main" {
            let file: String = row.get("file");
            db_file_opt = Some(file);
            break;
        }
    }

    if let Some(db_file) = db_file_opt {
        if !db_file.is_empty() {
            let _rekey_lock = state.rekey.lock().await;
            use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};

            drop(db_pool);

            if let Some(pool) = state.db.lock().await.take() {
                pool.close().await;
            }

            let opts = SqliteConnectOptions::new()
                .filename(db_file.as_str())
                .create_if_missing(false);
            
            let mut conn = SqliteConnection::connect_with(&opts)
                .await
                .map_err(|e| format!("Failed to open exclusive connection for rekey: {}", e))?;

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
                    if let Err(_) = unq_try {
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

            let tmp_key_opt = { let guard = state.key.lock().await; guard.clone() };
            if let Some(tmp_key) = tmp_key_opt {
                let key_slice: &[u8] = tmp_key.as_slice();
                let new_pool = crate::db::init_db(Path::new(&db_file), Some(key_slice)).await
                    .map_err(|e| format!("Failed to recreate pool after rekey: {}", e))?;
                *state.db.lock().await = Some(new_pool);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn unlock(state: State<'_, AppState>, password: String) -> Result<(), String> {
    let _rekey_lock = state.rekey.lock().await;

    let db_pool = get_db_pool(&state).await?;

    let salt_b64: String = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_salt")
        .fetch_one(&db_pool).await.map_err(|_| "Authentication failed".to_string())?
        .get("value");

    let nonce_b64: String = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_check_nonce")
        .fetch_one(&db_pool).await.map_err(|_| "Authentication failed".to_string())?
        .get("value");

    let ciphertext_b64: String = sqlx::query("SELECT value FROM configuration WHERE key = ?")
        .bind("password_check_ciphertext")
        .fetch_one(&db_pool).await.map_err(|_| "Authentication failed".to_string())?
        .get("value");

    let salt = general_purpose::STANDARD.decode(&salt_b64).map_err(|e| e.to_string())?;
    let nonce = general_purpose::STANDARD.decode(&nonce_b64).map_err(|e| e.to_string())?;
    let ciphertext = general_purpose::STANDARD.decode(&ciphertext_b64).map_err(|e| e.to_string())?;

    let mut derived_key = derive_key(&password, &salt)?;
    let key_z = Zeroizing::new(derived_key.to_vec());
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));

    let decrypted = cipher.decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| "Invalid password".to_string())?;

    if decrypted != PASSWORD_CHECK_PLAINTEXT {
        return Err("Invalid password".to_string());
    }

    {
        let mut key_guard = state.key.lock().await;
        *key_guard = Some(key_z.clone());
    }
    derived_key.zeroize();

    let rows = sqlx::query("PRAGMA database_list;")
        .fetch_all(&db_pool)
        .await
        .map_err(|e| format!("Failed to query database_list: {}", e))?;
    let mut db_file_opt: Option<String> = None;
    for row in rows {
        let name: String = row.get("name");
        if name == "main" {
            let file: String = row.get("file");
            db_file_opt = Some(file);
            break;
        }
    }

    if let Some(db_file) = db_file_opt {
        if !db_file.is_empty() {
            let tmp_key_opt = { let guard = state.key.lock().await; guard.clone() };
            if let Some(tmp_key) = tmp_key_opt {
                let key_slice: &[u8] = tmp_key.as_slice();

                drop(db_pool);

                if let Some(pool) = state.db.lock().await.take() {
                    pool.close().await;
                }

                let new_pool = crate::db::init_db(Path::new(&db_file), Some(key_slice)).await
                    .map_err(|e| format!("Failed to recreate pool after unlock: {}", e))?;
                *state.db.lock().await = Some(new_pool);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn lock(state: State<'_, AppState>) -> Result<(), String> {
    let mut key_guard = state.key.lock().await;
    *key_guard = None;
    Ok(())
}

#[tauri::command]
pub async fn is_locked(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.key.lock().await.is_none())
}

#[tauri::command]
pub async fn is_master_password_configured(state: State<'_, AppState>) -> Result<bool, String> {
    let db_pool = get_db_pool(&state).await?;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM configuration WHERE key = 'password_salt'")
        .fetch_one(&db_pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(count > 0)
}
