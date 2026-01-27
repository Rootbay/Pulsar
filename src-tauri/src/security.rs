use crate::state::AppState;
use crate::error::{Error, Result};
use crate::types::{SecretString};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;
use zeroize::{Zeroize, Zeroizing};
use crate::encryption::{encrypt, decrypt};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub last_seen: Option<String>,
    #[serde(default)]
    pub is_current: bool,
}

async fn get_key(state: &AppState) -> Result<Zeroizing<Vec<u8>>> {
    let guard = state.key.lock().await;
    let opt = guard.clone();
    drop(guard);
    opt.ok_or(Error::VaultLocked)
}

async fn get_db_pool(state: &AppState) -> Result<SqlitePool> {
    let guard = state.db.lock().await;
    guard
        .clone()
        .ok_or(Error::VaultNotLoaded)
}

async fn load_devices(pool: &SqlitePool, key: &[u8]) -> Result<Vec<DeviceRecord>> {
    let stored: Option<String> =
        sqlx::query_scalar("SELECT value FROM configuration WHERE key = 'device_registry'")
            .fetch_optional(pool)
            .await?;

    if let Some(enc_json) = stored {
        if enc_json.trim().is_empty() {
            Ok(Vec::new())
        } else {
            let json = decrypt(&enc_json, key)?;
            serde_json::from_str(&json)
                .map_err(|e| Error::Internal(format!("Failed to parse device registry: {e}")))
        }
    } else {
        Ok(Vec::new())
    }
}

async fn save_devices(pool: &SqlitePool, key: &[u8], devices: &[DeviceRecord]) -> Result<()> {
    let json = serde_json::to_string(devices)?;
    let payload = encrypt(&json, key)?;
    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES ('device_registry', ?)")
        .bind(payload)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn register_device(state: &AppState) -> Result<()> {
    let key = get_key(state).await?;
    let pool = get_db_pool(state).await?;
    let mut devices = load_devices(&pool, key.as_slice()).await?;

    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "Unknown Device".to_string());
    
    let platform = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
    };

    let device_id = format!("{hostname}-{platform}");
    let now = chrono::Utc::now().to_rfc3339();

    let mut found = false;
    for device in &mut devices {
        if device.id == device_id {
            device.last_seen = Some(now.clone());
            device.is_current = true;
            found = true;
        } else {
            device.is_current = false;
        }
    }

    if !found {
        devices.push(DeviceRecord {
            id: device_id,
            name: hostname,
            kind: platform.to_lowercase(),
            last_seen: Some(now),
            is_current: true,
        });
    }

    save_devices(&pool, key.as_slice(), &devices).await
}

#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<Vec<DeviceRecord>> {
    let key = get_key(&state).await?;
    let pool = get_db_pool(&state).await?;
    let mut devices = load_devices(&pool, key.as_slice()).await?;
    devices
        .iter_mut()
        .filter(|d| d.kind.trim().is_empty())
        .for_each(|d| d.kind = "unknown".to_string());

    Ok(devices)
}

#[tauri::command]
pub async fn remove_device(state: State<'_, AppState>, device_id: String) -> Result<()> {
    let key = get_key(&state).await?;
    let pool = get_db_pool(&state).await?;
    let mut devices = load_devices(&pool, key.as_slice()).await?;
    let original_len = devices.len();
    devices.retain(|device| device.id != device_id);

    if devices.len() == original_len {
        return Err(Error::Internal("Device not found".to_string()));
    }

    save_devices(&pool, key.as_slice(), &devices).await
}

#[tauri::command]
pub async fn revoke_all_devices(state: State<'_, AppState>) -> Result<()> {
    let key = get_key(&state).await?;
    let pool = get_db_pool(&state).await?;
    save_devices(&pool, key.as_slice(), &[]).await
}

#[tauri::command]
pub async fn wipe_memory(state: State<'_, AppState>) -> Result<()> {
    {
        let mut key_guard = state.key.lock().await;
        if let Some(mut key) = key_guard.take() {
            key.zeroize();
        }
    }

    {
        let mut pending_guard = state.pending_key.lock().await;
        if let Some(mut key) = pending_guard.take() {
            key.key.zeroize();
        }
    }

    {
        let mut db_guard = state.db.lock().await;
        if let Some(pool) = db_guard.take() {
            pool.close().await;
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityReport {
    pub reused_passwords: Vec<ReusedPasswordGroup>,
    pub weak_passwords_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReusedPasswordGroup {
    pub item_ids: Vec<i64>,
    pub count: usize,
}

#[tauri::command]
pub async fn get_security_report(state: State<'_, AppState>) -> Result<SecurityReport> {
    let key = get_key(&state).await?;
    let pool = get_db_pool(&state).await?;
    let items = crate::db::get_password_items_impl(&pool, key.as_slice()).await?;

    use std::collections::HashMap;
    let mut password_map: HashMap<SecretString, Vec<i64>> = HashMap::new();

    for item in items {
        if !item.password.as_str().is_empty() && item.password.as_str() != "N/A" {
            password_map.entry(item.password.clone()).or_default().push(item.id);
        }
    }

    let mut reused_passwords = Vec::new();
    let mut weak_passwords_count = 0;

    for (password, ids) in password_map {
        if ids.len() > 1 {
            reused_passwords.push(ReusedPasswordGroup {
                item_ids: ids.clone(),
                count: ids.len(),
            });
        }

        if password.len() < 8 {
            weak_passwords_count += ids.len();
        }
    }

    Ok(SecurityReport {
        reused_passwords,
        weak_passwords_count,
    })
}

#[tauri::command]
pub async fn run_integrity_check(state: State<'_, AppState>) -> Result<String> {
    let pool = get_db_pool(&state).await?;
    let result: (String,) = sqlx::query_as("PRAGMA integrity_check;")
        .fetch_one(&pool)
        .await?;
    Ok(result.0)
}