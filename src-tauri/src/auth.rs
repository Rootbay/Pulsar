use crate::encryption::{decrypt, encrypt};
use crate::state::{AppState, PendingUnlock};
use crate::security::register_device;
use crate::error::{Error, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    Key, XChaCha20Poly1305, XNonce,
};
use hkdf::Hkdf;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use keyring::Entry;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{Connection, Row};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::fs;
use tokio::time::Duration;
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "pulsar-vault";
const PASSWORD_CHECK_PLAINTEXT: &[u8] = b"pulsar-password-check";
const PENDING_TOTP_TTL: Duration = Duration::from_secs(120);
const MAX_TOTP_ATTEMPTS: u8 = 5;
const UNLOCK_BACKOFF_BASE_MS: u64 = 250;
const UNLOCK_BACKOFF_MAX_MS: u64 = 5000;
const ARGON2_MIN_MEMORY_KIB: u32 = 8 * 1024;
const ARGON2_MAX_MEMORY_KIB: u32 = 1024 * 1024;
const ARGON2_MAX_TIME_COST: u32 = 10;
const ARGON2_MAX_PARALLELISM: u32 = 16;
pub const UNLOCK_CONCURRENCY_LIMIT: usize = 1;
const SQLCIPHER_PAGE_SIZE: i64 = 4096;
const SQLCIPHER_KDF_ITER: i64 = 256_000;
const SQLCIPHER_HMAC_ALG: &str = "HMAC_SHA512";
const SQLCIPHER_KDF_ALG: &str = "PBKDF2_HMAC_SHA512";

#[cfg(target_os = "macos")]
#[link(name = "LocalAuthentication", kind = "framework")]
extern "C" {}

#[cfg(mobile)]
use tauri_plugin_biometric::{AuthOptions, BiometricExt, BiometryType};

#[cfg(target_os = "windows")]
use windows::{
    core::HSTRING,
    Security::Credentials::UI::{
        UserConsentVerifier, UserConsentVerifierAvailability, UserConsentVerificationResult,
    },
};

#[cfg(target_os = "macos")]
use {
    block::ConcreteBlock,
    dispatch::Semaphore,
    objc::{class, msg_send, sel, sel_impl},
    objc::runtime::Object,
    objc_foundation::NSString,
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[cfg(mobile)]
fn ensure_biometric_available(app: &AppHandle) -> Result<()> {
    let status = app
        .biometric()
        .status()
        .map_err(|e| Error::Internal(format!("Biometric status check failed: {}", e)))?;

    if !status.is_available || matches!(status.biometry_type, BiometryType::None) {
        let reason = status
            .error
            .unwrap_or_else(|| "Biometric authentication is unavailable.".to_string());
        return Err(Error::Internal(reason));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn ensure_biometric_available(_app: &AppHandle) -> Result<()> {
    let availability = UserConsentVerifier::CheckAvailabilityAsync()
        .map_err(|e| Error::Internal(format!("Biometric availability check failed: {}", e)))?
        .get()
        .map_err(|e| Error::Internal(format!("Biometric availability check failed: {}", e)))?;

    match availability {
        UserConsentVerifierAvailability::Available => Ok(()),
        UserConsentVerifierAvailability::DeviceNotPresent => Err(Error::Internal(
            "No biometric device is present.".to_string(),
        )),
        UserConsentVerifierAvailability::NotConfiguredForUser => Err(Error::Internal(
            "Biometrics are not configured for this user.".to_string(),
        )),
        UserConsentVerifierAvailability::DisabledByPolicy => Err(Error::Internal(
            "Biometrics are disabled by policy.".to_string(),
        )),
        _ => Err(Error::Internal(
            "Biometric authentication is unavailable.".to_string(),
        )),
    }
}

#[cfg(target_os = "macos")]
fn ensure_biometric_available(_app: &AppHandle) -> Result<()> {
    unsafe {
        let context: *mut Object = msg_send![class!(LAContext), new];
        let mut error: *mut Object = std::ptr::null_mut();
        let policy: i64 = 2; // LAPolicyDeviceOwnerAuthentication (biometrics or passcode)
        let can: bool = msg_send![context, canEvaluatePolicy:policy error:&mut error];
        if !can {
            return Err(Error::Internal(
                "Biometric authentication is unavailable.".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(not(any(mobile, target_os = "windows", target_os = "macos")))]
fn ensure_biometric_available(_app: &AppHandle) -> Result<()> {
    Err(Error::Internal(
        "Biometric authentication is not supported on this platform.".to_string(),
    ))
}

#[cfg(mobile)]
fn authenticate_biometric(app: &AppHandle, reason: &str) -> Result<()> {
    ensure_biometric_available(app)?;
    let options = AuthOptions {
        allow_device_credential: true,
        title: Some("Unlock Pulsar".to_string()),
        subtitle: Some("Confirm your identity to access the vault".to_string()),
        confirmation_required: Some(true),
        ..Default::default()
    };

    app.biometric()
        .authenticate(reason.to_string(), options)
        .map_err(|e| Error::Internal(format!("Biometric authentication failed: {}", e)))
}

#[cfg(target_os = "windows")]
fn authenticate_biometric(_app: &AppHandle, reason: &str) -> Result<()> {
    ensure_biometric_available(_app)?;

    let reason = HSTRING::from(reason);
    let result = UserConsentVerifier::RequestVerificationAsync(&reason)
        .map_err(|e| Error::Internal(format!("Biometric authentication failed: {}", e)))?
        .get()
        .map_err(|e| Error::Internal(format!("Biometric authentication failed: {}", e)))?;

    match result {
        UserConsentVerificationResult::Verified => Ok(()),
        UserConsentVerificationResult::DeviceBusy => Err(Error::Internal(
            "Biometric device is busy.".to_string(),
        )),
        UserConsentVerificationResult::DeviceNotPresent => Err(Error::Internal(
            "No biometric device is present.".to_string(),
        )),
        UserConsentVerificationResult::DisabledByPolicy => Err(Error::Internal(
            "Biometrics are disabled by policy.".to_string(),
        )),
        UserConsentVerificationResult::NotConfiguredForUser => Err(Error::Internal(
            "Biometrics are not configured for this user.".to_string(),
        )),
        UserConsentVerificationResult::Canceled => Err(Error::Internal(
            "Biometric verification was canceled.".to_string(),
        )),
        _ => Err(Error::Internal(
            "Biometric verification failed.".to_string(),
        )),
    }
}

#[cfg(target_os = "macos")]
fn authenticate_biometric(app: &AppHandle, reason: &str) -> Result<()> {
    ensure_biometric_available(app)?;

    unsafe {
        let context: *mut Object = msg_send![class!(LAContext), new];
        let policy: i64 = 2; // LAPolicyDeviceOwnerAuthentication (biometrics or passcode)
        let reason = NSString::from_str(reason);
        let semaphore = Arc::new(Semaphore::new(0));
        let success = Arc::new(AtomicBool::new(false));
        let success_clone = success.clone();
        let semaphore_clone = semaphore.clone();

        let handler = ConcreteBlock::new(move |ok: bool, _err: *mut Object| {
            success_clone.store(ok, Ordering::SeqCst);
            semaphore_clone.signal();
        })
        .copy();

        let _: () = msg_send![context, evaluatePolicy:policy localizedReason:reason reply:handler];
        semaphore.wait();
        if success.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(Error::Internal("Biometric verification failed.".to_string()))
        }
    }
}

#[cfg(not(any(mobile, target_os = "windows", target_os = "macos")))]
fn authenticate_biometric(_app: &AppHandle, _reason: &str) -> Result<()> {
    Err(Error::Internal(
        "Biometric authentication is not supported on this platform.".to_string(),
    ))
}

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
    #[serde(default)]
    mac_version: Option<u8>,
    #[serde(default)]
    mac_nonce_b64: Option<String>,
    #[serde(default)]
    mac_tag_b64: Option<String>,
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

async fn write_password_metadata(
    db_path: &Path,
    meta: &PasswordMetadata,
    mac_key: Option<&[u8]>,
) -> Result<()> {
    let path = metadata_path(db_path);
    let tmp_path = path.with_extension("meta.json.tmp");
    let mut meta = meta.clone();
    if let Some(key) = mac_key {
        let vault_id = get_vault_id(db_path);
        let (nonce_b64, tag_b64) = compute_metadata_mac(&meta, &vault_id, key)?;
        meta.mac_version = Some(1);
        meta.mac_nonce_b64 = Some(nonce_b64);
        meta.mac_tag_b64 = Some(tag_b64);
    }

    let bytes = serde_json::to_vec_pretty(&meta)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path)
            .await?;
        let _ = file.set_permissions(std::fs::Permissions::from_mode(0o600)).await;
        file.write_all(&bytes).await?;
        file.sync_all().await?;
        fs::rename(&tmp_path, &path).await?;
        if let Ok(dir) = fs::File::open(
            path.parent().unwrap_or_else(|| Path::new(".")),
        )
        .await
        {
            let _ = dir.sync_all().await;
        }
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        fs::write(&tmp_path, bytes).await?;
        fs::rename(&tmp_path, &path).await?;
        Ok(())
    }
}

fn decode_metadata(meta: &PasswordMetadata) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let salt = general_purpose::STANDARD
        .decode(&meta.salt_b64)
        .map_err(|e| Error::Internal(format!("Invalid salt encoding: {}", e)))?;
    let nonce = general_purpose::STANDARD
        .decode(&meta.nonce_b64)
        .map_err(|e| Error::Internal(format!("Invalid nonce encoding: {}", e)))?;
    if nonce.len() != 24 {
        return Err(Error::Validation("Invalid nonce length".to_string()));
    }
    let ciphertext = general_purpose::STANDARD
        .decode(&meta.ciphertext_b64)
        .map_err(|e| Error::Internal(format!("Invalid ciphertext encoding: {}", e)))?;
    Ok((salt, nonce, ciphertext))
}

fn unlock_backoff_duration(failures: u32) -> Duration {
    if failures == 0 {
        return Duration::from_millis(0);
    }
    let exp = failures.min(6);
    let base = UNLOCK_BACKOFF_BASE_MS.saturating_mul(1u64 << exp);
    Duration::from_millis(base.min(UNLOCK_BACKOFF_MAX_MS))
}

async fn ensure_unlock_not_throttled(state: &State<'_, AppState>) -> Result<()> {
    let guard = state.unlock_rate_limit.lock().await;
    let Some(last_failure) = guard.last_failure else {
        return Ok(());
    };
    let backoff = unlock_backoff_duration(guard.failures);
    if last_failure.elapsed() < backoff {
        return Err(Error::Validation(
            "Too many attempts. Please wait and try again.".to_string(),
        ));
    }
    Ok(())
}

async fn register_unlock_failure(state: &State<'_, AppState>) {
    let mut guard = state.unlock_rate_limit.lock().await;
    guard.failures = guard.failures.saturating_add(1);
    guard.last_failure = Some(Instant::now());
}

async fn reset_unlock_failures(state: &State<'_, AppState>) {
    let mut guard = state.unlock_rate_limit.lock().await;
    guard.failures = 0;
    guard.last_failure = None;
}

#[derive(Serialize)]
struct MetadataMacPayload<'a> {
    vault_id: &'a str,
    version: u8,
    salt_b64: &'a str,
    nonce_b64: &'a str,
    ciphertext_b64: &'a str,
    argon2_memory_kib: Option<u32>,
    argon2_time_cost: Option<u32>,
    argon2_parallelism: Option<u32>,
}

fn metadata_mac_payload(meta: &PasswordMetadata, vault_id: &str) -> Result<Vec<u8>> {
    serde_json::to_vec(&MetadataMacPayload {
        vault_id,
        version: meta.version,
        salt_b64: &meta.salt_b64,
        nonce_b64: &meta.nonce_b64,
        ciphertext_b64: &meta.ciphertext_b64,
        argon2_memory_kib: meta.argon2_memory_kib,
        argon2_time_cost: meta.argon2_time_cost,
        argon2_parallelism: meta.argon2_parallelism,
    })
    .map_err(|e| Error::Internal(format!("Failed to serialize metadata MAC payload: {}", e)))
}

fn derive_metadata_mac_key(master_key: &[u8]) -> Result<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(None, master_key);
    let mut out = [0u8; 32];
    hk.expand(b"pulsar:meta", &mut out)
        .map_err(|_| Error::Internal("Failed to derive metadata MAC key".to_string()))?;
    Ok(out)
}

fn compute_metadata_mac(
    meta: &PasswordMetadata,
    vault_id: &str,
    master_key: &[u8],
) -> Result<(String, String)> {
    let mac_key = derive_metadata_mac_key(master_key)?;
    let payload = metadata_mac_payload(meta, vault_id)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&mac_key));

    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let tag = cipher
        .encrypt(nonce, Payload { msg: b"", aad: &payload })
        .map_err(|e| Error::Encryption(format!("Metadata MAC failed: {}", e)))?;

    let nonce_b64 = general_purpose::STANDARD.encode(nonce_bytes);
    let tag_b64 = general_purpose::STANDARD.encode(tag);
    Ok((nonce_b64, tag_b64))
}

fn verify_metadata_mac(meta: &PasswordMetadata, vault_id: &str, master_key: &[u8]) -> Result<()> {
    let mac_key = derive_metadata_mac_key(master_key)?;
    if meta.mac_version.unwrap_or(1) != 1 {
        return Err(Error::Validation("Unsupported metadata MAC version".to_string()));
    }
    let nonce_b64 = meta
        .mac_nonce_b64
        .as_deref()
        .ok_or_else(|| Error::Validation("Missing metadata MAC nonce".to_string()))?;
    let tag_b64 = meta
        .mac_tag_b64
        .as_deref()
        .ok_or_else(|| Error::Validation("Missing metadata MAC tag".to_string()))?;

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|e| Error::Validation(format!("Invalid metadata MAC nonce: {}", e)))?;
    if nonce_bytes.len() != 24 {
        return Err(Error::Validation("Invalid metadata MAC nonce length".to_string()));
    }

    let tag_bytes = general_purpose::STANDARD
        .decode(tag_b64)
        .map_err(|e| Error::Validation(format!("Invalid metadata MAC tag: {}", e)))?;

    let payload = metadata_mac_payload(meta, vault_id)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&mac_key));
    let expected_tag = cipher
        .encrypt(XNonce::from_slice(&nonce_bytes), Payload { msg: b"", aad: &payload })
        .map_err(|e| Error::Encryption(format!("Metadata MAC failed: {}", e)))?;

    if expected_tag.len() != tag_bytes.len()
        || expected_tag.ct_eq(&tag_bytes).unwrap_u8() != 1
    {
        return Err(Error::Validation(
            "Vault metadata integrity check failed.".to_string(),
        ));
    }

    Ok(())
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
        mac_version: None,
        mac_nonce_b64: None,
        mac_tag_b64: None,
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

async fn connect_with_key(db_path: &Path, key_bytes: &[u8]) -> Result<SqliteConnection> {
    let hex_key = hex::encode(key_bytes);
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(10))
        .pragma("key", format!("\"x'{}'\"", hex_key));

    SqliteConnection::connect_with(&connect_options).await.map_err(Error::Database)
}

async fn connect_plaintext(db_path: &Path) -> Result<SqliteConnection> {
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(10))
        .pragma("key", "''");

    SqliteConnection::connect_with(&connect_options).await.map_err(Error::Database)
}

async fn connect_plaintext_raw(db_path: &Path) -> Result<SqliteConnection> {
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(10));

    SqliteConnection::connect_with(&connect_options).await.map_err(Error::Database)
}

fn is_not_a_database_error(err: &sqlx::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("file is not a database") || msg.contains("code 26")
}

fn is_unable_to_open_db_error(err: &sqlx::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("unable to open database") || msg.contains("code 14") || msg.contains("code: 14")
}

async fn replace_db_with_backup(
    db_path: &Path,
    temp_db_path: &Path,
    context: &str,
) -> Result<()> {
    let backup_path = db_path.with_extension("psec_backup");
    if backup_path.exists() {
        let _ = fs::remove_file(&backup_path).await;
    }

    fs::rename(db_path, &backup_path).await?;
    if let Err(err) = fs::rename(temp_db_path, db_path).await {
        let _ = fs::rename(&backup_path, db_path).await;
        return Err(Error::Internal(format!(
            "Failed to replace vault database during {}: {}",
            context, err
        )));
    }

    let _ = fs::remove_file(&backup_path).await;
    Ok(())
}

fn build_attach_cmd(path: &Path, hex_key: &str) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    let raw_sql = raw.replace("'", "''");
    format!(
        "ATTACH DATABASE '{}' AS encrypted KEY \"x'{}'\"",
        raw_sql, hex_key
    )
}

async fn apply_sqlcipher_pragmas(
    conn: &mut SqliteConnection,
    db_name: Option<&str>,
) -> Result<()> {
    let prefix = db_name.map(|name| format!("{}.", name)).unwrap_or_default();
    let statements = [
        format!("PRAGMA {}cipher_page_size = {}", prefix, SQLCIPHER_PAGE_SIZE),
        format!("PRAGMA {}kdf_iter = {}", prefix, SQLCIPHER_KDF_ITER),
        format!("PRAGMA {}cipher_hmac_algorithm = {}", prefix, SQLCIPHER_HMAC_ALG),
        format!("PRAGMA {}cipher_kdf_algorithm = {}", prefix, SQLCIPHER_KDF_ALG),
    ];

    for stmt in statements {
        let _ = sqlx::query(&stmt).execute(&mut *conn).await;
    }

    Ok(())
}

async fn attach_encrypted_db(
    conn: &mut SqliteConnection,
    path: &Path,
    hex_key: &str,
) -> Result<()> {
    let attach_cmd = build_attach_cmd(path, hex_key);
    match sqlx::query(&attach_cmd).execute(&mut *conn).await {
        Ok(_) => {
            let _ = apply_sqlcipher_pragmas(conn, Some("encrypted")).await;
            Ok(())
        }
        Err(err) => {
            if is_unable_to_open_db_error(&err) {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).await.map_err(Error::Io)?;
                }
                let _ = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(path)
                    .await;
                sqlx::query(&attach_cmd).execute(&mut *conn).await?;
                let _ = apply_sqlcipher_pragmas(conn, Some("encrypted")).await;
                Ok(())
            } else {
                Err(Error::Database(err))
            }
        }
    }
}

async fn write_password_metadata_to_db(
    db_path: &Path,
    key_bytes: &[u8],
    metadata: &PasswordMetadata,
) -> Result<()> {
    let hex_key = hex::encode(key_bytes);
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(30))
        .pragma("key", format!("\"x'{}'\"", hex_key));

    let mut conn = connect_with_timeout(&connect_options, Duration::from_secs(15))
        .await
        .map_err(Error::Database)?;

    sqlx::query("BEGIN").execute(&mut conn).await?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_salt")
        .bind(&metadata.salt_b64)
        .execute(&mut conn)
        .await?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_check_nonce")
        .bind(&metadata.nonce_b64)
        .execute(&mut conn)
        .await?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
        .bind("password_check_ciphertext")
        .bind(&metadata.ciphertext_b64)
        .execute(&mut conn)
        .await?;

    if let Some(val) = metadata.argon2_memory_kib {
        sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
            .bind("argon2_memory_kib")
            .bind(val.to_string())
            .execute(&mut conn)
            .await?;
    }

    if let Some(val) = metadata.argon2_time_cost {
        sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
            .bind("argon2_time_cost")
            .bind(val.to_string())
            .execute(&mut conn)
            .await?;
    }

    if let Some(val) = metadata.argon2_parallelism {
        sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES (?, ?)")
            .bind("argon2_parallelism")
            .bind(val.to_string())
            .execute(&mut conn)
            .await?;
    }

    sqlx::query("COMMIT").execute(&mut conn).await?;
    conn.close().await?;

    Ok(())
}

async fn is_plaintext_sqlite(db_path: &Path) -> Result<bool> {
    for use_empty_key in [true, false] {
        let conn_result = if use_empty_key {
            connect_plaintext(db_path).await
        } else {
            connect_plaintext_raw(db_path).await
        };

        let mut conn = match conn_result {
            Ok(conn) => conn,
            Err(err) => {
                if let Error::Database(db_err) = &err {
                    if is_not_a_database_error(db_err) {
                        continue;
                    }
                }
                return Err(err);
            }
        };

        let result = sqlx::query("SELECT count(*) FROM sqlite_master")
            .execute(&mut conn)
            .await;
        conn.close().await?;

        match result {
            Ok(_) => return Ok(true),
            Err(err) => {
                if is_not_a_database_error(&err) {
                    continue;
                }
                return Err(Error::Database(err));
            }
        }
    }

    Ok(false)
}

async fn close_pool_with_timeout(pool: sqlx::SqlitePool, timeout: Duration) -> Result<()> {
    tokio::time::timeout(timeout, pool.close())
        .await
        .map_err(|_| {
            Error::Internal(
                "Timed out while closing the database. Please try again.".to_string(),
            )
        })?;
    Ok(())
}

async fn log_cipher_version(conn: &mut SqliteConnection, context: &str) {
    match sqlx::query_scalar::<_, String>("PRAGMA cipher_version")
        .fetch_optional(&mut *conn)
        .await
    {
        Ok(Some(version)) => eprintln!("[{}] cipher_version={}", context, version),
        Ok(None) => eprintln!("[{}] cipher_version=<none>", context),
        Err(err) => eprintln!("[{}] cipher_version query failed: {}", context, err),
    }
}

async fn validate_encrypted_db(db_path: &Path, key_bytes: &[u8]) -> Result<()> {
    match connect_with_key(db_path, key_bytes).await {
        Ok(mut conn) => {
            let result = sqlx::query("SELECT count(*) FROM sqlite_master")
                .execute(&mut conn)
                .await;
            let _ = conn.close().await;
            match result {
                Ok(_) => Ok(()),
                Err(err) => Err(Error::Database(err)),
            }
        }
        Err(err) => Err(err),
    }
}

async fn rekey_plaintext_db(db_path: &Path, key_bytes: &[u8]) -> Result<()> {
    let temp_db_path = db_path.with_extension("tmp_rekey_psec");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).await.map_err(Error::Io)?;
    }
    let hex_key = hex::encode(key_bytes);

    let mut last_err: Option<Error> = None;
    for _ in 0..10 {
        if temp_db_path.exists() {
            let _ = fs::remove_file(&temp_db_path).await;
        }

        let mut conn = match connect_plaintext_raw(db_path).await {
            Ok(conn) => conn,
            Err(err) => match connect_plaintext(db_path).await {
                Ok(conn) => conn,
                Err(_) => {
                    last_err = Some(err);
                    tokio::time::sleep(Duration::from_millis(750)).await;
                    continue;
                }
            },
        };

        log_cipher_version(&mut conn, "rekey/plaintext").await;

        let export_result: Result<()> = async {
            attach_encrypted_db(&mut conn, &temp_db_path, &hex_key).await?;
            sqlx::query("SELECT sqlcipher_export('encrypted')")
                .execute(&mut conn)
                .await?;
            sqlx::query("DETACH DATABASE encrypted").execute(&mut conn).await?;
            Ok(())
        }
        .await;

        let _ = conn.close().await;

        match export_result {
            Ok(()) => {
                tokio::time::sleep(Duration::from_millis(500)).await;
                if let Err(err) = fs::remove_file(db_path).await {
                    last_err = Some(Error::Io(err));
                } else if let Err(err) = fs::rename(&temp_db_path, db_path).await {
                    last_err = Some(Error::Io(err));
                } else if let Err(err) = validate_encrypted_db(db_path, key_bytes).await {
                    last_err = Some(err);
                } else {
                    return Ok(());
                }
            }
            Err(err) => last_err = Some(err),
        }

        tokio::time::sleep(Duration::from_millis(750)).await;
    }

    Err(last_err.unwrap_or_else(|| {
        Error::Internal("Failed to encrypt database after multiple attempts.".to_string())
    }))
}

async fn connect_with_timeout(
    connect_options: &SqliteConnectOptions,
    timeout: Duration,
) -> std::result::Result<SqliteConnection, sqlx::Error> {
    match tokio::time::timeout(timeout, SqliteConnection::connect_with(connect_options)).await {
        Ok(result) => result,
        Err(_) => Err(sqlx::Error::PoolTimedOut),
    }
}

async fn finalize_unlock(
    state: &State<'_, AppState>,
    key_z: Zeroizing<Vec<u8>>,
) -> Result<()> {
    let db_path = get_db_path(state).await?;

    {
        let mut db_guard = state.db.lock().await;
        if let Some(pool) = db_guard.take() {
            pool.close().await;
        }
    }

    tokio::time::sleep(Duration::from_millis(2000)).await;

    let new_pool = crate::db::init_db_lazy(db_path.as_path(), Some(key_z.as_slice()))
        .await
        .map_err(Error::Internal)?;

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
        if let Some(mut key) = pending_guard.take() {
            key.key.zeroize();
        }
    }

    match tokio::time::timeout(Duration::from_secs(5), register_device(state)).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            eprintln!("Failed to register device: {}", e);
        }
        Err(_) => {
            eprintln!("Failed to register device: timed out");
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<()> {
    let password = Zeroizing::new(password);
    validate_new_password(password.as_str())?;
    let _rekey_lock = tokio::time::timeout(Duration::from_secs(15), state.rekey.lock())
        .await
        .map_err(|_| Error::Internal("Vault is busy. Please try again.".to_string()))?;
    let db_path = get_db_path(&state).await?;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let argon_params = Argon2ParamsConfig::default();

    let mut derived_key = derive_key(password.as_str(), &salt, &argon_params)?;
    drop(password);
    let key_z = Zeroizing::new(derived_key.to_vec());
    derived_key.zeroize();

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
        mac_version: None,
        mac_nonce_b64: None,
        mac_tag_b64: None,
    };

    if let Some(pool) = { state.db.lock().await.take() } {
        close_pool_with_timeout(pool, Duration::from_secs(15)).await?;
    }

    tokio::time::sleep(Duration::from_millis(1500)).await;

    let hex_key = hex::encode(key_z.as_slice());

    let temp_db_path = db_path.with_extension("tmp_psec");
    if temp_db_path.exists() {
        let _ = fs::remove_file(&temp_db_path).await;
    }

    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(30));
    
    let mut last_err: Option<Error> = None;
    for _ in 0..10 {
        match connect_with_timeout(&connect_options, Duration::from_secs(15)).await {
            Ok(mut conn) => {
                attach_encrypted_db(&mut conn, &temp_db_path, &hex_key).await?;
                sqlx::query("SELECT sqlcipher_export('encrypted')").execute(&mut conn).await?;
                sqlx::query("DETACH DATABASE encrypted").execute(&mut conn).await?;

                conn.close().await?;

                write_password_metadata_to_db(&temp_db_path, key_z.as_slice(), &metadata).await?;
                
                tokio::time::sleep(Duration::from_millis(1000)).await;
                fs::remove_file(&db_path).await?;
                fs::rename(&temp_db_path, &db_path).await?;
                
                write_password_metadata(db_path.as_path(), &metadata, Some(key_z.as_slice()))
                    .await?;
                last_err = None;
                break;
            }
            Err(e) => {
                last_err = Some(Error::Database(e));
                tokio::time::sleep(Duration::from_millis(1500)).await;
            }
        }
    }
    
    if let Some(e) = last_err {
        return Err(Error::Internal(format!("Failed to connect for rekeying after retries: {}", e)));
    }

    finalize_unlock(&state, key_z.clone()).await?;
    derived_key.zeroize();

    Ok(())
}

#[tauri::command]
pub async fn unlock(
    state: State<'_, AppState>,
    password: String,
) -> Result<UnlockResponse> {
    let password = Zeroizing::new(password);
    let _unlock_permit = state
        .unlock_guard
        .acquire()
        .await
        .map_err(|_| Error::Internal("Unlock guard closed".to_string()))?;
    let db_path = get_db_path(&state).await?;
    ensure_unlock_not_throttled(&state).await?;

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
    validate_argon_params(&argon_params)?;

    let mut derived_key = derive_key(password.as_str(), &salt, &argon_params)?;
    drop(password);
    let key_z = Zeroizing::new(derived_key.to_vec());
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));

    let mut decrypted = match cipher.decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref()) {
        Ok(value) => value,
        Err(_) => {
            register_unlock_failure(&state).await;
            return Err(Error::InvalidPassword);
        }
    };

    let is_valid = decrypted.ct_eq(PASSWORD_CHECK_PLAINTEXT).unwrap_u8() == 1;
    decrypted.zeroize();
    if !is_valid {
        register_unlock_failure(&state).await;
        return Err(Error::InvalidPassword);
    }
    reset_unlock_failures(&state).await;

    if meta.mac_tag_b64.is_some() {
        let vault_id = get_vault_id(db_path.as_path());
        verify_metadata_mac(&meta, &vault_id, key_z.as_slice())?;
    }

    let should_refresh_meta = meta.mac_tag_b64.is_none();
    if should_refresh_meta || read_password_metadata(db_path.as_path()).await?.is_none() {
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
                mac_version: None,
                mac_nonce_b64: None,
                mac_tag_b64: None,
            },
            Some(key_z.as_slice()),
        )
        .await?;
    }

    let is_plaintext = is_plaintext_sqlite(db_path.as_path()).await?;
    if is_plaintext {
        if let Some(pool) = { state.db.lock().await.take() } {
            if let Err(err) = close_pool_with_timeout(pool, Duration::from_secs(15)).await {
                let _ = err;
            }
        }
        tokio::time::sleep(Duration::from_millis(1500)).await;
        rekey_plaintext_db(db_path.as_path(), key_z.as_slice()).await?;
    }

    let mut conn = match connect_with_key(db_path.as_path(), key_z.as_slice()).await {
        Ok(conn) => conn,
        Err(err) => {
            if let Error::Database(sqlx_err) = &err {
                let _ = sqlx_err;
                if is_not_a_database_error(sqlx_err) {
                    let is_plaintext_retry = is_plaintext_sqlite(db_path.as_path()).await?;
                    if is_plaintext_retry {
                        if let Some(pool) = { state.db.lock().await.take() } {
                            if let Err(err) = close_pool_with_timeout(pool, Duration::from_secs(15)).await {
                                let _ = err;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(1500)).await;
                        rekey_plaintext_db(db_path.as_path(), key_z.as_slice()).await?;
                        connect_with_key(db_path.as_path(), key_z.as_slice()).await?
                    } else {
                        return Err(err);
                    }
                } else {
                    return Err(err);
                }
            } else {
                return Err(err);
            }
        }
    };

    let totp_query =
        "SELECT COUNT(*) FROM configuration WHERE key = 'login_totp_secret'";
    let totp_configured: i64 =
        match sqlx::query_scalar(totp_query).fetch_one(&mut conn).await {
            Ok(value) => value,
            Err(err) => {
                if is_not_a_database_error(&err) {
                    let _ = err;
                    conn.close().await?;
                    let is_plaintext_retry = is_plaintext_sqlite(db_path.as_path()).await?;
                    if is_plaintext_retry {
                        if let Some(pool) = { state.db.lock().await.take() } {
                            if let Err(err) = close_pool_with_timeout(pool, Duration::from_secs(15)).await {
                                let _ = err;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(1500)).await;
                        rekey_plaintext_db(db_path.as_path(), key_z.as_slice()).await?;
                        let mut retry_conn =
                            connect_with_key(db_path.as_path(), key_z.as_slice()).await?;
                        let value = sqlx::query_scalar(totp_query)
                            .fetch_one(&mut retry_conn)
                            .await?;
                        conn = retry_conn;
                        value
                    } else {
                        return Err(Error::Database(err));
                    }
                } else {
                    return Err(Error::Database(err));
                }
            }
        };

    conn.close().await?;

    let totp_required = totp_configured > 0;

    if totp_required {
        {
            let mut pending_guard = state.pending_key.lock().await;
            *pending_guard = Some(PendingUnlock {
                key: key_z.clone(),
                created_at: Instant::now(),
                attempts: 0,
            });
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
    let pending_key = {
        let mut guard = state.pending_key.lock().await;
        let pending = guard
            .as_mut()
            .ok_or_else(|| Error::Internal("No pending unlock operation".to_string()))?;

        if pending.created_at.elapsed() > PENDING_TOTP_TTL {
            if let Some(mut expired) = guard.take() {
                expired.key.zeroize();
            }
            return Err(Error::Validation(
                "TOTP session expired. Please unlock again.".to_string(),
            ));
        }

        if pending.attempts >= MAX_TOTP_ATTEMPTS {
            if let Some(mut exhausted) = guard.take() {
                exhausted.key.zeroize();
            }
            return Err(Error::Validation(
                "Too many invalid attempts. Please unlock again.".to_string(),
            ));
        }

        pending.key.clone()
    };
    let trimmed = token.trim();
    if trimmed.len() < 6 {
        let mut guard = state.pending_key.lock().await;
        if let Some(pending) = guard.as_mut() {
            pending.attempts = pending.attempts.saturating_add(1);
            if pending.attempts >= MAX_TOTP_ATTEMPTS {
                if let Some(mut exhausted) = guard.take() {
                    exhausted.key.zeroize();
                }
                return Err(Error::Validation(
                    "Too many invalid attempts. Please unlock again.".to_string(),
                ));
            }
        }
        return Err(Error::Validation("Invalid TOTP token".to_string()));
    }

    let db_path = get_db_path(&state).await?;
    let mut conn = connect_with_key(db_path.as_path(), pending_key.as_slice()).await?;

    let secret_enc: Option<String> =
        sqlx::query_scalar("SELECT value FROM configuration WHERE key = 'login_totp_secret'")
            .fetch_optional(&mut conn)
            .await?;

    let secret_enc = secret_enc.ok_or_else(|| Error::Internal("Login TOTP is not configured.".to_string()))?;
    let secret_b32 = Zeroizing::new(decrypt(&secret_enc, pending_key.as_slice())?);

    let secret = Secret::Encoded(secret_b32.to_string());
    let mut secret_bytes = secret.to_bytes().map_err(|e| Error::Totp(e.to_string()))?;

    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes.clone(),
        Some("Pulsar".to_string()),
        "vault".to_string(),
    )
    .map_err(|e| Error::Totp(e.to_string()))?;

    let is_valid = totp.check_current(trimmed).unwrap_or(false);
    secret_bytes.zeroize();
    if !is_valid {
        let mut guard = state.pending_key.lock().await;
        if let Some(pending) = guard.as_mut() {
            pending.attempts = pending.attempts.saturating_add(1);
            if pending.attempts >= MAX_TOTP_ATTEMPTS {
                if let Some(mut exhausted) = guard.take() {
                    exhausted.key.zeroize();
                }
                return Err(Error::Validation(
                    "Too many invalid attempts. Please unlock again.".to_string(),
                ));
            }
        }
        return Err(Error::Validation("Invalid TOTP token".to_string()));
    }

    conn.close().await?;
    finalize_unlock(&state, pending_key.clone()).await?;
    Ok(())
}

fn validate_password_inputs(current: &str, new_password: &str) -> Result<()> {
    if current.trim().is_empty() {
        return Err(Error::Validation("Current password is required.".to_string()));
    }

    validate_new_password(new_password)?;

    if new_password.trim() == current.trim() {
        return Err(Error::Validation("New password must be different from the current password.".to_string()));
    }

    Ok(())
}

fn validate_new_password(new_password: &str) -> Result<()> {
    let trimmed = new_password.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation("Password is required.".to_string()));
    }
    if trimmed.len() < 12 {
        return Err(Error::Validation("Password must be at least 12 characters.".to_string()));
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
    if params.memory_kib < ARGON2_MIN_MEMORY_KIB {
        return Err(Error::Validation("Argon2 memory must be at least 8 MiB.".to_string()));
    }
    if params.memory_kib > ARGON2_MAX_MEMORY_KIB {
        return Err(Error::Validation("Argon2 memory is too high.".to_string()));
    }

    if params.time_cost == 0 {
        return Err(Error::Validation("Argon2 time cost must be at least 1.".to_string()));
    }
    if params.time_cost > ARGON2_MAX_TIME_COST {
        return Err(Error::Validation("Argon2 time cost is too high.".to_string()));
    }

    if params.parallelism == 0 {
        return Err(Error::Validation("Argon2 parallelism must be at least 1.".to_string()));
    }
    if params.parallelism > ARGON2_MAX_PARALLELISM {
        return Err(Error::Validation("Argon2 parallelism is too high.".to_string()));
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
    let current_password = Zeroizing::new(current_password);
    let new_password = Zeroizing::new(new_password);
    validate_password_inputs(current_password.as_str(), new_password.as_str())?;

    let _rekey_lock = state.rekey.lock().await;
    let db_pool = get_db_pool(&state).await?;
    let db_path = get_db_path(&state).await?;

    let mut metadata = load_existing_metadata(&state, &db_pool, db_path.as_path()).await?;
    let (salt, nonce, ciphertext) = decode_metadata(&metadata)?;
    let argon_params = metadata.argon2_params();
    validate_argon_params(&argon_params)?;

    let mut current_key_bytes = derive_key(current_password.as_str(), &salt, &argon_params)?;
    let current_key_z = Zeroizing::new(current_key_bytes.to_vec());
    current_key_bytes.zeroize();

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&current_key_z));
    let mut decrypted = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| Error::Validation("Invalid current password".to_string()))?;
    let is_valid = decrypted.ct_eq(PASSWORD_CHECK_PLAINTEXT).unwrap_u8() == 1;
    decrypted.zeroize();
    if !is_valid {
        return Err(Error::Validation("Invalid current password".to_string()));
    }

    let mut new_salt = [0u8; 16];
    OsRng.fill_bytes(&mut new_salt);

    let mut new_key_bytes = derive_key(new_password.as_str(), &new_salt, &argon_params)?;
    drop(current_password);
    drop(new_password);
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

    if let Some(pool) = { state.db.lock().await.take() } {
        close_pool_with_timeout(pool, Duration::from_secs(15)).await?;
    }

    tokio::time::sleep(Duration::from_millis(1500)).await;

    let hex_old_key = hex::encode(current_key_z.as_slice());
    let hex_new_key = hex::encode(new_key_z.as_slice());

    let temp_db_path = db_path.with_extension("tmp_rotate_psec");
    if temp_db_path.exists() {
        let _ = fs::remove_file(&temp_db_path).await;
    }

    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(30))
        .pragma("key", format!("\"x'{}'\"", hex_old_key));
    
    let mut last_err: Option<Error> = None;
    for _ in 0..10 {
        match connect_with_timeout(&connect_options, Duration::from_secs(15)).await {
            Ok(mut conn) => {
                attach_encrypted_db(&mut conn, &temp_db_path, &hex_new_key).await?;
                sqlx::query("SELECT sqlcipher_export('encrypted')").execute(&mut conn).await?;
                sqlx::query("DETACH DATABASE encrypted").execute(&mut conn).await?;

                conn.close().await?;

                write_password_metadata_to_db(&temp_db_path, new_key_z.as_slice(), &metadata).await?;
                
                tokio::time::sleep(Duration::from_millis(1000)).await;
                replace_db_with_backup(&db_path, &temp_db_path, "master password rotation").await?;
                
                write_password_metadata(db_path.as_path(), &metadata, Some(new_key_z.as_slice()))
                    .await?;
                last_err = None;
                break;
            }
            Err(e) => {
                last_err = Some(Error::Database(e));
                tokio::time::sleep(Duration::from_millis(1500)).await;
            }
        }
    }
    
    if let Some(e) = last_err {
        return Err(Error::Internal(format!("Failed to connect for master password rotation: {}", e)));
    }

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
    let current_password = Zeroizing::new(current_password);
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
    validate_argon_params(&current_params)?;

    let mut current_key_bytes = derive_key(current_password.as_str(), &salt, &current_params)?;
    let current_key_z = Zeroizing::new(current_key_bytes.to_vec());
    current_key_bytes.zeroize();

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&current_key_z));
    let mut decrypted = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| Error::Validation("Invalid current password".to_string()))?;
    let is_valid = decrypted.ct_eq(PASSWORD_CHECK_PLAINTEXT).unwrap_u8() == 1;
    decrypted.zeroize();
    if !is_valid {
        return Err(Error::Validation("Invalid current password".to_string()));
    }

    let mut new_salt = [0u8; 16];
    OsRng.fill_bytes(&mut new_salt);

    let mut new_key_bytes = derive_key(current_password.as_str(), &new_salt, &new_params)?;
    drop(current_password);
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

    if let Some(pool) = { state.db.lock().await.take() } {
        close_pool_with_timeout(pool, Duration::from_secs(15)).await?;
    }

    let hex_old_key = hex::encode(current_key_z.as_slice());
    let hex_new_key = hex::encode(new_key_z.as_slice());

    let temp_db_path = db_path.with_extension("tmp_argon_psec");
    if temp_db_path.exists() {
        let _ = fs::remove_file(&temp_db_path).await;
    }

    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(30))
        .pragma("key", format!("\"x'{}'\"", hex_old_key));
    
    let mut last_err: Option<Error> = None;
    for _ in 0..10 {
        match connect_with_timeout(&connect_options, Duration::from_secs(15)).await {
            Ok(mut conn) => {
                attach_encrypted_db(&mut conn, &temp_db_path, &hex_new_key).await?;
                sqlx::query("SELECT sqlcipher_export('encrypted')").execute(&mut conn).await?;
                sqlx::query("DETACH DATABASE encrypted").execute(&mut conn).await?;

                conn.close().await?;

                write_password_metadata_to_db(&temp_db_path, new_key_z.as_slice(), &metadata).await?;
                
                tokio::time::sleep(Duration::from_millis(1000)).await;
                replace_db_with_backup(&db_path, &temp_db_path, "Argon2 parameter update").await?;
                
                write_password_metadata(db_path.as_path(), &metadata, Some(new_key_z.as_slice()))
                    .await?;
                last_err = None;
                break;
            }
            Err(e) => {
                last_err = Some(Error::Database(e));
                tokio::time::sleep(Duration::from_millis(1500)).await;
            }
        }
    }
    
    if let Some(e) = last_err {
        return Err(Error::Internal(format!("Failed to connect for Argon2 parameter update: {}", e)));
    }

    finalize_unlock(&state, new_key_z.clone()).await?;

    Ok(())
}

pub(crate) async fn verify_master_password_impl(
    state: &State<'_, AppState>,
    password: &str,
) -> Result<bool> {
    if password.trim().is_empty() {
        return Err(Error::Validation("Master password is required.".to_string()));
    }

    let db_path = get_db_path(state).await?;
    let metadata = match read_password_metadata(db_path.as_path()).await? {
        Some(meta) => Some(meta),
        None => {
            let pool = get_db_pool(state).await?;
            load_metadata_from_db(&pool).await?
        }
    };

    let meta = metadata.ok_or_else(|| {
        Error::Internal("Vault is not initialised with a master password.".to_string())
    })?;
    let (salt, nonce, ciphertext) = decode_metadata(&meta)?;
    let argon_params = meta.argon2_params();
    validate_argon_params(&argon_params)?;

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

#[tauri::command]
pub async fn verify_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<bool> {
    let password = Zeroizing::new(password);
    verify_master_password_impl(&state, password.as_str()).await
}

#[tauri::command]
pub async fn configure_login_totp(
    state: State<'_, AppState>,
    secret_b32: String,
) -> Result<()> {
    let secret_b32 = Zeroizing::new(secret_b32);
    let key_opt = {
        let guard = state.key.lock().await;
        guard.clone()
    };

    let key_z = key_opt.ok_or(Error::VaultLocked)?;

    Secret::Encoded(secret_b32.to_string())
        .to_bytes()
        .map_err(|e| Error::Validation(format!("Invalid TOTP secret: {}", e)))?;

    let encrypted = encrypt(secret_b32.as_str(), key_z.as_slice())?;
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
        if let Some(mut key) = pending.take() {
            key.key.zeroize();
        }
    }

    if let Err(error) = app.clipboard().clear() {
        eprintln!("Failed to clear clipboard on lock: {}", error);
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
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
) -> Result<()> {
    let password = Zeroizing::new(password);
    ensure_biometric_available(&app)?;
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
    validate_argon_params(&argon_params)?;

    let mut derived_key = derive_key(password.as_str(), &salt, &argon_params)?;
    let key_z = Zeroizing::new(derived_key.to_vec());
    derived_key.zeroize();
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_z));

    let mut decrypted = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| Error::Validation("Invalid password".to_string()))?;

    let is_valid = decrypted.ct_eq(PASSWORD_CHECK_PLAINTEXT).unwrap_u8() == 1;
    decrypted.zeroize();
    if !is_valid {
        return Err(Error::Validation("Invalid password".to_string()));
    }

    let mut bio_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bio_key_bytes);
    let bio_key_b64 = Zeroizing::new(general_purpose::STANDARD.encode(bio_key_bytes));

    let encrypted_password_blob = encrypt(password.as_str(), &bio_key_bytes)?;
    drop(password);
    let db_pool = get_db_pool(&state).await?;
    sqlx::query(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES ('biometric_encrypted_password', ?)",
    )
    .bind(encrypted_password_blob)
    .execute(&db_pool)
    .await?;

    let vault_user = get_vault_id(&db_path);
    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    entry.set_password(bio_key_b64.as_str()).map_err(|e| Error::Internal(e.to_string()))?;
    bio_key_bytes.zeroize();

    Ok(())
}

#[tauri::command]
pub async fn disable_biometrics(state: State<'_, AppState>) -> Result<()> {
    let db_path = get_db_path(&state).await?;
    let vault_user = get_vault_id(&db_path);

    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    let _ = entry.delete_credential(); 

    if let Ok(db_pool) = get_db_pool(&state).await {
        let _ = sqlx::query("DELETE FROM configuration WHERE key = 'biometric_encrypted_password'")
            .execute(&db_pool)
            .await;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn is_biometrics_enabled(app: AppHandle, state: State<'_, AppState>) -> Result<bool> {
    if ensure_biometric_available(&app).is_err() {
        return Ok(false);
    }
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
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<UnlockResponse> {
    authenticate_biometric(&app, "Unlock your Pulsar vault")?;
    let db_path = get_db_path(&state).await?;
    let vault_user = get_vault_id(&db_path);

    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    let bio_key_b64 = entry.get_password().map_err(|e| {
        if matches!(e, keyring::Error::NoEntry) {
            Error::Internal("Biometrics not configured for this vault".to_string())
        } else {
            Error::Internal(e.to_string())
        }
    })?;

    let mut bio_key_vec = general_purpose::STANDARD
        .decode(&bio_key_b64)
        .map_err(|_| Error::Internal("Invalid biometric key format".to_string()))?;
    if bio_key_vec.len() != 32 {
        bio_key_vec.zeroize();
        return Err(Error::Internal("Invalid biometric key length".to_string()));
    }
    let mut bio_key_bytes = [0u8; 32];
    bio_key_bytes.copy_from_slice(&bio_key_vec);
    bio_key_vec.zeroize();

    let db_pool = get_db_pool(&state).await?;
    let row = sqlx::query("SELECT value FROM configuration WHERE key = 'biometric_encrypted_password'")
        .fetch_optional(&db_pool)
        .await?;

    let encrypted_password_blob: String = match row {
        Some(r) => r.get("value"),
        None => return Err(Error::Internal("Biometric configuration corrupted (DB entry missing)".to_string())),
    };

    let master_password = decrypt(&encrypted_password_blob, &bio_key_bytes)
        .map_err(|_| Error::Internal("Biometric decryption failed".to_string()))?;
    bio_key_bytes.zeroize();

    unlock(state, master_password).await
}

#[cfg(test)]
mod tests {
    use super::replace_db_with_backup;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::fs;

    fn unique_temp_dir() -> std::path::PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("pulsar-rekey-test-{}-{}", nanos, count))
    }

    #[tokio::test]
    async fn replace_db_with_backup_swaps_and_cleans() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).await.unwrap();

        let db_path = dir.join("vault.db");
        let temp_path = dir.join("vault.tmp");

        fs::write(&db_path, b"old").await.unwrap();
        fs::write(&temp_path, b"new").await.unwrap();

        replace_db_with_backup(&db_path, &temp_path, "test")
            .await
            .unwrap();

        let final_bytes = fs::read(&db_path).await.unwrap();
        assert_eq!(final_bytes, b"new");
        assert!(!temp_path.exists());

        let backup_path = db_path.with_extension("psec_backup");
        assert!(!backup_path.exists());

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn replace_db_with_backup_rolls_back_on_failure() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).await.unwrap();

        let db_path = dir.join("vault.db");
        let temp_path = dir.join("missing.tmp");

        fs::write(&db_path, b"old").await.unwrap();

        let result = replace_db_with_backup(&db_path, &temp_path, "test").await;
        assert!(result.is_err());

        let final_bytes = fs::read(&db_path).await.unwrap();
        assert_eq!(final_bytes, b"old");

        let backup_path = db_path.with_extension("psec_backup");
        assert!(!backup_path.exists());

        let _ = fs::remove_dir_all(&dir).await;
    }
}

