use crate::encryption::{decrypt, encrypt};
use crate::state::AppState;
use crate::security::register_device;
use crate::error::{Error, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    Key, XChaCha20Poly1305, XNonce,
};
use keyring::Entry;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{Connection, Row};
use std::path::{Path, PathBuf};
use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::fs;
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "pulsar-vault";
const PASSWORD_CHECK_PLAINTEXT: &[u8] = b"pulsar-password-check";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PasswordMetadata {
    version: u8,
    salt_b64: String,
    nonce_b64: String,
    ciphertext_b64: String,
    #[serde(default)]
    argon2_memory_kib: Option<u32>,
    #[serde(default)]
    argon2_time_cost: Option<u32>,
    #[serde(default)]
    argon2_parallelism: Option<u32>,
}

#[derive(Debug, Clone)]
struct Argon2ParamsConfig {
    memory_kib: u32,
    time_cost: u32,
    parallelism: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Argon2ParamsResponse {
    memory_kib: u32,
    time_cost: u32,
    parallelism: u32,
}

impl From<Argon2ParamsConfig> for Argon2ParamsResponse {
    fn from(value: Argon2ParamsConfig) -> Self {
        Self {
            memory_kib: value.memory_kib,
            time_cost: value.time_cost,
            parallelism: value.parallelism,
        }
    }
}

impl Default for Argon2ParamsConfig {
    fn default() -> Self {
        Self {
            memory_kib: 64 * 1024,
            time_cost: 3,
            parallelism: 4,
        }
    }
}

impl Argon2ParamsConfig {
    fn to_params(&self) -> Result<Params> {
        Params::new(self.memory_kib, self.time_cost, self.parallelism, None)
            .map_err(|e| Error::Internal(format!("Invalid Argon2 parameters: {}", e)))
    }
}

impl PasswordMetadata {
    fn argon2_params(&self) -> Argon2ParamsConfig {
        let defaults = Argon2ParamsConfig::default();
        Argon2ParamsConfig {
            memory_kib: self.argon2_memory_kib.unwrap_or(defaults.memory_kib),
            time_cost: self.argon2_time_cost.unwrap_or(defaults.time_cost),
            parallelism: self.argon2_parallelism.unwrap_or(defaults.parallelism),
        }
    }
}

#[derive(Serialize)]
pub struct UnlockResponse {
    pub totp_required: bool,
}

async fn get_db_pool(state: &State<'_, AppState>) -> Result<sqlx::SqlitePool> {
    let guard = state.db.lock().await;
    guard
        .clone()
        .ok_or(Error::VaultNotLoaded)
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

async fn read_password_metadata(db_path: &Path) -> Result<Option<PasswordMetadata>> {
    let path = metadata_path(db_path);
    match fs::read(&path).await {
        Ok(bytes) => {
            let meta: PasswordMetadata = serde_json::from_slice(&bytes)?;
            Ok(Some(meta))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(Error::Io(err)),
    }
}

async fn write_password_metadata(db_path: &Path, meta: &PasswordMetadata) -> Result<()> {
    let path = metadata_path(db_path);
    let bytes = serde_json::to_vec_pretty(meta)?;
    fs::write(&path, bytes).await?;
    Ok(())
}

async fn persist_metadata_to_db(
    db_pool: &sqlx::SqlitePool,
    meta: &PasswordMetadata,
) -> Result<()> {
    let mut tx = db_pool.begin().await?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_salt")
        .bind(&meta.salt_b64)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_check_nonce")
        .bind(&meta.nonce_b64)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_check_ciphertext")
        .bind(&meta.ciphertext_b64)
        .execute(&mut *tx)
        .await?;

    match meta.argon2_memory_kib {
        Some(value) => {
            sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
                .bind("argon2_memory_kib")
                .bind(value.to_string())
                .execute(&mut *tx)
                .await?;
        }
        None => {
            sqlx::query("DELETE FROM configuration WHERE key = ?")
                .bind("argon2_memory_kib")
                .execute(&mut *tx)
                .await?;
        }
    }

    match meta.argon2_time_cost {
        Some(value) => {
            sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
                .bind("argon2_time_cost")
                .bind(value.to_string())
                .execute(&mut *tx)
                .await?;
        }
        None => {
            sqlx::query("DELETE FROM configuration WHERE key = ?")
                .bind("argon2_time_cost")
                .execute(&mut *tx)
                .await?;
        }
    }

    match meta.argon2_parallelism {
        Some(value) => {
            sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
                .bind("argon2_parallelism")
                .bind(value.to_string())
                .execute(&mut *tx)
                .await?;
        }
        None => {
            sqlx::query("DELETE FROM configuration WHERE key = ?")
                .bind("argon2_parallelism")
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

fn decode_metadata(meta: &PasswordMetadata) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let salt = general_purpose::STANDARD
        .decode(&meta.salt_b64)
        .map_err(|e| Error::Internal(format!("Invalid salt encoding: {}", e)))?;
    let nonce = general_purpose::STANDARD
        .decode(&meta.nonce_b64)
        .map_err(|e| Error::Internal(format!("Invalid nonce encoding: {}", e)))?;
    let ciphertext = general_purpose::STANDARD
        .decode(&meta.ciphertext_b64)
        .map_err(|e| Error::Internal(format!("Invalid ciphertext encoding: {}", e)))?;
    Ok((salt, nonce, ciphertext))
}

async fn load_metadata_from_db(
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
    }))
}

fn derive_key(
    password: &str,
    salt: &[u8],
    params: &Argon2ParamsConfig,
) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    let params = params.to_params()?;
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| Error::Internal(format!("Failed to derive key: {}", e)))?;
    Ok(key)
}

async fn get_db_path(state: &State<'_, AppState>) -> Result<PathBuf> {
    state
        .db_path
        .lock()
        .await
        .clone()
        .ok_or(Error::Internal("Database path is not set. Select a vault first.".to_string()))
}

async fn apply_key_to_conn(conn: &mut SqliteConnection, key_bytes: &[u8]) -> Result<()> {
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
                    .map_err(|_| Error::Internal("Failed to apply pragma key using fallback method.".to_string()))?;
            }
        } else {
            return Err(Error::Internal(format!("Failed to apply pragma key: {}", e)));
        }
    }

    Ok(())
}

async fn apply_rekey(conn: &mut SqliteConnection, key_bytes: &[u8]) -> Result<()> {
    let rekey_try = sqlx::query("PRAGMA rekey = ?;")
        .bind(key_bytes)
        .execute(&mut *conn)
        .await;

    if let Err(e) = rekey_try {
        let msg = e.to_string();
        if msg.contains("near \"?\": syntax error") || msg.contains("syntax error") {
            let hex_key: String = key_bytes.iter().map(|b| format!("{:02x}", b)).collect();
            let pragma_unquoted = format!("PRAGMA rekey = x'{}';", hex_key);
            let unq_try = sqlx::query(&pragma_unquoted).execute(&mut *conn).await;
            if unq_try.is_err() {
                let pragma_quoted = format!("PRAGMA rekey = \"x'{}'\";", hex_key);
                sqlx::query(&pragma_quoted)
                    .execute(&mut *conn)
                    .await
                    .map_err(|_| Error::Internal("Failed to rekey database using fallback method.".to_string()))?;
            }
        } else {
            return Err(Error::Internal(format!("Failed to rekey database: {}", e)));
        }
    }

    Ok(())
}

async fn finalize_unlock(
    state: &State<'_, AppState>,
    key_z: Zeroizing<Vec<u8>>,
) -> Result<()> {
    let db_path = get_db_path(state).await?;

    {
        if let Some(pool) = state.db.lock().await.take() {
            pool.close().await;
        }
    }

    let new_pool = crate::db::init_db(db_path.as_path(), Some(key_z.as_slice())).await
        .map_err(|e| Error::Internal(e))?;

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

    // Register current device in the registry
    if let Err(e) = register_device(state).await {
        eprintln!("Failed to register device: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub async fn set_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<()> {
    let db_pool = get_db_pool(&state).await?;
    let db_path = get_db_path(&state).await?;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let argon_params = Argon2ParamsConfig::default();

    let mut derived_key = derive_key(&password, &salt, &argon_params)?;
    let key_z = Zeroizing::new(derived_key.to_vec());

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), PASSWORD_CHECK_PLAINTEXT)
        .map_err(|e| Error::Encryption(format!("Encryption failed: {}", e)))?;

    let salt_b64 = general_purpose::STANDARD.encode(&salt);
    let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
    let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);

    let metadata = PasswordMetadata {
        version: 1,
        salt_b64: salt_b64.clone(),
        nonce_b64: nonce_b64.clone(),
        ciphertext_b64: ciphertext_b64.clone(),
        argon2_memory_kib: Some(argon_params.memory_kib),
        argon2_time_cost: Some(argon_params.time_cost),
        argon2_parallelism: Some(argon_params.parallelism),
    };

    persist_metadata_to_db(&db_pool, &metadata).await?;
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
        .await?;

    apply_key_to_conn(&mut conn, key_z.as_slice()).await?;

    apply_rekey(&mut conn, key_z.as_slice()).await?;

    drop(conn);

    finalize_unlock(&state, key_z.clone()).await?;
    derived_key.zeroize();

    Ok(())
}

#[tauri::command]
pub async fn unlock(
    state: State<'_, AppState>,
    password: String,
) -> Result<UnlockResponse> {
    let _rekey_lock = state.rekey.lock().await;
    let db_path = get_db_path(&state).await?;

    let metadata = match read_password_metadata(db_path.as_path()).await? {
        Some(meta) => Some(meta),
        None => {
            let pool = get_db_pool(&state).await?;
            load_metadata_from_db(&pool).await?
        }
    };

    let meta =
        metadata.ok_or_else(|| Error::Internal("Vault is not initialised with a master password.".to_string()))?;
    let (salt, nonce, ciphertext) = decode_metadata(&meta)?;

    let argon_params = meta.argon2_params();

    let mut derived_key = derive_key(&password, &salt, &argon_params)?;
    let key_z = Zeroizing::new(derived_key.to_vec());
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));

    let decrypted = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| Error::InvalidPassword)?;

    if decrypted != PASSWORD_CHECK_PLAINTEXT {
        return Err(Error::InvalidPassword);
    }

    if read_password_metadata(db_path.as_path()).await?.is_none() {
        write_password_metadata(
            db_path.as_path(),
            &PasswordMetadata {
                version: meta.version,
                salt_b64: general_purpose::STANDARD.encode(&salt),
                nonce_b64: general_purpose::STANDARD.encode(&nonce),
                ciphertext_b64: general_purpose::STANDARD.encode(&ciphertext),
                argon2_memory_kib: meta.argon2_memory_kib,
                argon2_time_cost: meta.argon2_time_cost,
                argon2_parallelism: meta.argon2_parallelism,
            },
        )
        .await?;
    }

    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false);
    let mut conn = SqliteConnection::connect_with(&connect_options)
        .await?;

    apply_key_to_conn(&mut conn, key_z.as_slice()).await?;

    let totp_configured: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM configuration WHERE key = 'login_totp_secret'")
            .fetch_one(&mut conn)
            .await?;

    drop(conn);

    let totp_required = totp_configured > 0;

    if totp_required {
        {
            let mut pending_guard = state.pending_key.lock().await;
            *pending_guard = Some(key_z.clone());
        }
        derived_key.zeroize();
        Ok(UnlockResponse {
            totp_required: true,
        })
    } else {
        finalize_unlock(&state, key_z.clone()).await?;
        derived_key.zeroize();
        Ok(UnlockResponse {
            totp_required: false,
        })
    }
}

#[tauri::command]
pub async fn verify_login_totp(state: State<'_, AppState>, token: String) -> Result<()> {
    let pending_key_opt = {
        let guard = state.pending_key.lock().await;
        guard.clone()
    };

    let pending_key = pending_key_opt.ok_or_else(|| Error::Internal("No pending unlock operation".to_string()))?;
    let trimmed = token.trim();
    if trimmed.len() < 6 {
        return Err(Error::Validation("Invalid TOTP token".to_string()));
    }

    let db_path = get_db_path(&state).await?;
    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false);
    let mut conn = SqliteConnection::connect_with(&connect_options)
        .await?;

    apply_key_to_conn(&mut conn, pending_key.as_slice()).await?;

    let secret_enc: Option<String> =
        sqlx::query_scalar("SELECT value FROM configuration WHERE key = 'login_totp_secret'")
            .fetch_optional(&mut conn)
            .await?;

    let secret_enc = secret_enc.ok_or_else(|| Error::Internal("Login TOTP is not configured.".to_string()))?;
    let secret_b32 = decrypt(&secret_enc, pending_key.as_slice())?;

    let secret = Secret::Encoded(secret_b32.clone());
    let secret_bytes = secret.to_bytes().map_err(|e| Error::Totp(e.to_string()))?;

    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Pulsar".to_string()),
        "vault".to_string(),
    )
    .map_err(|e| Error::Totp(e.to_string()))?;

    let is_valid = totp.check_current(trimmed).unwrap_or(false);
    if !is_valid {
        return Err(Error::Validation("Invalid TOTP token".to_string()));
    }

    drop(conn);
    finalize_unlock(&state, pending_key.clone()).await?;
    Ok(())
}

fn validate_password_inputs(current: &str, new_password: &str) -> Result<()> {
    if current.trim().is_empty() {
        return Err(Error::Validation("Current password is required.".to_string()));
    }

    if new_password.trim().len() < 8 {
        return Err(Error::Validation("New password must be at least 8 characters.".to_string()));
    }

    if new_password.trim() == current.trim() {
        return Err(Error::Validation("New password must be different from the current password.".to_string()));
    }

    Ok(())
}

async fn load_existing_metadata(
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

fn validate_argon_params(params: &Argon2ParamsConfig) -> Result<()> {
    if params.memory_kib < 8 * 1024 {
        return Err(Error::Validation("Argon2 memory must be at least 8 MiB.".to_string()));
    }

    if params.time_cost == 0 {
        return Err(Error::Validation("Argon2 time cost must be at least 1.".to_string()));
    }

    if params.parallelism == 0 {
        return Err(Error::Validation("Argon2 parallelism must be at least 1.".to_string()));
    }

    Ok(())
}

#[tauri::command]
pub async fn get_argon2_params(state: State<'_, AppState>) -> Result<Argon2ParamsResponse> {
    let db_pool = get_db_pool(&state).await?;
    let db_path = get_db_path(&state).await?;

    let metadata = load_existing_metadata(&state, &db_pool, db_path.as_path()).await;

    match metadata {
        Ok(meta) => Ok(meta.argon2_params().into()),
        Err(_) => Ok(Argon2ParamsConfig::default().into()),
    }
}

#[tauri::command]
pub async fn rotate_master_password(
    state: State<'_, AppState>,
    current_password: String,
    new_password: String,
) -> Result<()> {
    validate_password_inputs(&current_password, &new_password)?;

    let _rekey_lock = state.rekey.lock().await;
    let db_pool = get_db_pool(&state).await?;
    let db_path = get_db_path(&state).await?;

    let mut metadata = load_existing_metadata(&state, &db_pool, db_path.as_path()).await?;
    let (salt, nonce, ciphertext) = decode_metadata(&metadata)?;
    let argon_params = metadata.argon2_params();

    let mut current_key_bytes = derive_key(&current_password, &salt, &argon_params)?;
    let current_key_z = Zeroizing::new(current_key_bytes.to_vec());
    current_key_bytes.zeroize();

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&current_key_z));
    cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| Error::Validation("Invalid current password".to_string()))?;

    let mut new_salt = [0u8; 16];
    OsRng.fill_bytes(&mut new_salt);

    let mut new_key_bytes = derive_key(&new_password, &new_salt, &argon_params)?;
    let new_key_z = Zeroizing::new(new_key_bytes.to_vec());
    new_key_bytes.zeroize();

    let mut new_nonce = [0u8; 24];
    OsRng.fill_bytes(&mut new_nonce);

    let new_cipher = XChaCha20Poly1305::new(Key::from_slice(&new_key_z));
    let new_ciphertext = new_cipher
        .encrypt(XNonce::from_slice(&new_nonce), PASSWORD_CHECK_PLAINTEXT)
        .map_err(|e| Error::Encryption(format!("Encryption failed: {}", e)))?;

    metadata.salt_b64 = general_purpose::STANDARD.encode(&new_salt);
    metadata.nonce_b64 = general_purpose::STANDARD.encode(&new_nonce);
    metadata.ciphertext_b64 = general_purpose::STANDARD.encode(&new_ciphertext);
    metadata.argon2_memory_kib = Some(argon_params.memory_kib);
    metadata.argon2_time_cost = Some(argon_params.time_cost);
    metadata.argon2_parallelism = Some(argon_params.parallelism);

    persist_metadata_to_db(&db_pool, &metadata).await?;
    write_password_metadata(db_path.as_path(), &metadata).await?;

    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false);
    let mut conn = SqliteConnection::connect_with(&connect_options)
        .await?;

    apply_key_to_conn(&mut conn, current_key_z.as_slice()).await?;
    apply_rekey(&mut conn, new_key_z.as_slice()).await?;
    drop(conn);

    finalize_unlock(&state, new_key_z.clone()).await?;

    Ok(())
}

#[tauri::command]
pub async fn update_argon2_params(
    state: State<'_, AppState>,
    current_password: String,
    memory_kib: u32,
    time_cost: u32,
    parallelism: u32,
) -> Result<()> {
    if current_password.trim().is_empty() {
        return Err(Error::Validation("Current password is required.".to_string()));
    }

    let new_params = Argon2ParamsConfig {
        memory_kib,
        time_cost,
        parallelism,
    };
    validate_argon_params(&new_params)?;

    let _rekey_lock = state.rekey.lock().await;
    let db_pool = get_db_pool(&state).await?;
    let db_path = get_db_path(&state).await?;

    let mut metadata = load_existing_metadata(&state, &db_pool, db_path.as_path()).await?;
    let (salt, nonce, ciphertext) = decode_metadata(&metadata)?;
    let current_params = metadata.argon2_params();

    let mut current_key_bytes = derive_key(&current_password, &salt, &current_params)?;
    let current_key_z = Zeroizing::new(current_key_bytes.to_vec());
    current_key_bytes.zeroize();

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&current_key_z));
    cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| Error::Validation("Invalid current password".to_string()))?;

    let mut new_salt = [0u8; 16];
    OsRng.fill_bytes(&mut new_salt);

    let mut new_key_bytes = derive_key(&current_password, &new_salt, &new_params)?;
    let new_key_z = Zeroizing::new(new_key_bytes.to_vec());
    new_key_bytes.zeroize();

    let mut new_nonce = [0u8; 24];
    OsRng.fill_bytes(&mut new_nonce);

    let new_cipher = XChaCha20Poly1305::new(Key::from_slice(&new_key_z));
    let new_ciphertext = new_cipher
        .encrypt(XNonce::from_slice(&new_nonce), PASSWORD_CHECK_PLAINTEXT)
        .map_err(|e| Error::Encryption(format!("Encryption failed: {}", e)))?;

    metadata.salt_b64 = general_purpose::STANDARD.encode(&new_salt);
    metadata.nonce_b64 = general_purpose::STANDARD.encode(&new_nonce);
    metadata.ciphertext_b64 = general_purpose::STANDARD.encode(&new_ciphertext);
    metadata.argon2_memory_kib = Some(new_params.memory_kib);
    metadata.argon2_time_cost = Some(new_params.time_cost);
    metadata.argon2_parallelism = Some(new_params.parallelism);

    persist_metadata_to_db(&db_pool, &metadata).await?;
    write_password_metadata(db_path.as_path(), &metadata).await?;

    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false);
    let mut conn = SqliteConnection::connect_with(&connect_options)
        .await?;

    apply_key_to_conn(&mut conn, current_key_z.as_slice()).await?;
    apply_rekey(&mut conn, new_key_z.as_slice()).await?;
    drop(conn);

    finalize_unlock(&state, new_key_z.clone()).await?;

    Ok(())
}

#[tauri::command]
pub async fn configure_login_totp(
    state: State<'_, AppState>,
    secret_b32: String,
) -> Result<()> {
    let key_opt = {
        let guard = state.key.lock().await;
        guard.clone()
    };

    let key_z = key_opt.ok_or(Error::VaultLocked)?;

    Secret::Encoded(secret_b32.clone())
        .to_bytes()
        .map_err(|e| Error::Validation(format!("Invalid TOTP secret: {}", e)))?;

    let encrypted = encrypt(&secret_b32, key_z.as_slice())?;
    let db_pool = get_db_pool(&state).await?;

    sqlx::query(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES ('login_totp_secret', ?)",
    )
    .bind(encrypted)
    .execute(&db_pool)
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn disable_login_totp(state: State<'_, AppState>) -> Result<()> {
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("DELETE FROM configuration WHERE key = 'login_totp_secret'")
        .execute(&db_pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn is_login_totp_configured(state: State<'_, AppState>) -> Result<bool> {
    let db_pool = get_db_pool(&state).await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM configuration WHERE key = 'login_totp_secret'")
            .fetch_one(&db_pool)
            .await?;
    Ok(count > 0)
}

#[tauri::command]
pub async fn lock(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    {
        let mut key_guard = state.key.lock().await;
        *key_guard = None;
    }
    {
        let mut pending = state.pending_key.lock().await;
        *pending = None;
    }

    let should_clear = {
        let policy = state.clipboard_policy.lock().await;
        policy.integration_enabled && policy.only_unlocked
    };

    if should_clear {
        if let Err(error) = app.clipboard().clear() {
            eprintln!("Failed to clear clipboard on lock: {}", error);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn is_locked(state: State<'_, AppState>) -> Result<bool> {
    Ok(state.key.lock().await.is_none())
}

#[tauri::command]
pub async fn is_master_password_configured(state: State<'_, AppState>) -> Result<bool> {
    if let Ok(db_path) = get_db_path(&state).await {
        if let Ok(Some(_)) = read_password_metadata(db_path.as_path()).await {
            return Ok(true);
        }
    }

    if let Ok(db_pool) = get_db_pool(&state).await {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM configuration WHERE key = 'password_salt'")
            .fetch_one(&db_pool)
            .await
            .unwrap_or(0);
        return Ok(count > 0);
    }

    Ok(false)
}

fn get_vault_id(db_path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(db_path.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    format!("vault-{}", hex::encode(&result[..8]))
}

#[tauri::command]
pub async fn enable_biometrics(
    state: State<'_, AppState>,
    password: String,
) -> Result<()> {
    // 1. Verify password first
    let db_path = get_db_path(&state).await?;
    let metadata = match read_password_metadata(db_path.as_path()).await? {
        Some(meta) => Some(meta),
        None => {
            let pool = get_db_pool(&state).await?;
            load_metadata_from_db(&pool).await?
        }
    };

    let meta = metadata.ok_or_else(|| Error::Internal("Vault is not initialised.".to_string()))?;
    let (salt, nonce, ciphertext) = decode_metadata(&meta)?;
    let argon_params = meta.argon2_params();

    let derived_key = derive_key(&password, &salt, &argon_params)?;
    let key_z = Zeroizing::new(derived_key.to_vec());
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));

    let decrypted = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| Error::Validation("Invalid password".to_string()))?;

    if decrypted != PASSWORD_CHECK_PLAINTEXT {
        return Err(Error::Validation("Invalid password".to_string()));
    }

    // 2. Generate Biometric Key (High Entropy)
    let mut bio_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bio_key_bytes);
    let bio_key_b64 = general_purpose::STANDARD.encode(bio_key_bytes);

    // 3. Encrypt Master Password with Biometric Key
    let encrypted_password_blob = encrypt(&password, &bio_key_bytes)?;

    // 4. Store Encrypted Password in Database
    let db_pool = get_db_pool(&state).await?;
    sqlx::query(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES ('biometric_encrypted_password', ?)",
    )
    .bind(encrypted_password_blob)
    .execute(&db_pool)
    .await?;

    // 5. Store Biometric Key in Keyring (Vault Specific)
    let vault_user = get_vault_id(&db_path);
    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    entry.set_password(&bio_key_b64).map_err(|e| Error::Internal(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn disable_biometrics(state: State<'_, AppState>) -> Result<()> {
    let db_path = get_db_path(&state).await?;
    let vault_user = get_vault_id(&db_path);

    // Remove from Keyring
    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    let _ = entry.delete_credential(); 

    // Remove from Database
    if let Ok(db_pool) = get_db_pool(&state).await {
        let _ = sqlx::query("DELETE FROM configuration WHERE key = 'biometric_encrypted_password'")
            .execute(&db_pool)
            .await;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn is_biometrics_enabled(state: State<'_, AppState>) -> Result<bool> {
    let db_path = get_db_path(&state).await?;
    let vault_user = get_vault_id(&db_path);
    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(Error::Internal(e.to_string())),
    }
}

#[tauri::command]
pub async fn unlock_with_biometrics(
    state: State<'_, AppState>,
) -> Result<UnlockResponse> {
    let db_path = get_db_path(&state).await?;
    let vault_user = get_vault_id(&db_path);

    // 1. Retrieve Biometric Key from Keyring
    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    let bio_key_b64 = entry.get_password().map_err(|e| {
        if matches!(e, keyring::Error::NoEntry) {
            Error::Internal("Biometrics not configured for this vault".to_string())
        } else {
            Error::Internal(e.to_string())
        }
    })?;

    let bio_key_bytes = general_purpose::STANDARD
        .decode(&bio_key_b64)
        .map_err(|_| Error::Internal("Invalid biometric key format".to_string()))?;

    // 2. Retrieve Encrypted Password from DB
    let db_pool = get_db_pool(&state).await?;
    let row = sqlx::query("SELECT value FROM configuration WHERE key = 'biometric_encrypted_password'")
        .fetch_optional(&db_pool)
        .await?;

    let encrypted_password_blob: String = match row {
        Some(r) => r.get("value"),
        None => return Err(Error::Internal("Biometric configuration corrupted (DB entry missing)".to_string())),
    };

    // 3. Decrypt Master Password
    let master_password = decrypt(&encrypted_password_blob, &bio_key_bytes)
        .map_err(|_| Error::Internal("Biometric decryption failed".to_string()))?;

    // 4. Unlock
    unlock(state, master_password).await
}


