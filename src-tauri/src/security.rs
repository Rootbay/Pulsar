use crate::db::utils::{get_db_pool, get_key};
use crate::error::{Error, Result};
use crate::state::AppState;
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

async fn get_key_local(state: &AppState) -> Result<Zeroizing<Vec<u8>>> {
    let guard = state.key.lock().await;
    let opt = guard.clone();
    drop(guard);
    opt.ok_or(Error::VaultLocked)
}

async fn get_db_pool_local(state: &AppState) -> Result<SqlitePool> {
    let guard = state.db.lock().await;
    guard.clone().ok_or(Error::VaultNotLoaded)
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
    let key = get_key_local(state).await?;
    let pool = get_db_pool_local(state).await?;
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
    let mut devices = load_devices(&pool, key.as_slice()).await?;
    devices.retain(|device| device.is_current);
    save_devices(&pool, key.as_slice(), &devices).await
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
    pub weak_passwords: Vec<i64>,
    pub breached_passwords: Vec<i64>,
    pub unique_passwords_count: usize,
    pub total_passwords_count: usize,
    pub overall_health_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReusedPasswordGroup {
    pub password_hash: String,
    pub item_ids: Vec<i64>,
    pub count: usize,
}

#[tauri::command]
pub async fn get_security_report(state: State<'_, AppState>) -> Result<SecurityReport> {
    let key = get_key(&state).await?;
    let pool = get_db_pool(&state).await?;
    let items = crate::db::get_password_items_impl(&pool, key.as_slice()).await?;

    use std::collections::HashMap;
    use sha2::{Sha256, Digest};

    let mut password_map: HashMap<String, Vec<i64>> = HashMap::new();
    let mut breached_passwords = Vec::new();
    let total_passwords_count = items.len();

    for item in &items {
        if !item.password.as_str().is_empty() && item.password.as_str() != "N/A" {
            let mut hasher = Sha256::new();
            hasher.update(item.password.as_str().as_bytes());
            let hash = hex::encode(hasher.finalize());
            password_map.entry(hash).or_default().push(item.id);
        }
        
        let mut is_breached = false;
        if let Some(tags) = &item.tags {
            let t = tags.to_lowercase();
            if t.contains("breached") || t.contains("compromised") || t.contains("leaked") {
                is_breached = true;
            }
        }
        
        if is_breached {
            breached_passwords.push(item.id);
        }
    }

    let mut reused_passwords = Vec::new();
    let mut weak_passwords = Vec::new();
    let unique_passwords_count = password_map.len();
    let mut total_reused_items = 0;

    for (hash, ids) in &password_map {
        if ids.len() > 1 {
            reused_passwords.push(ReusedPasswordGroup {
                password_hash: hash.clone(),
                item_ids: ids.clone(),
                count: ids.len(),
            });
            total_reused_items += ids.len();
        }
    }

    for item in &items {
        if !item.password.as_str().is_empty() && (item.password.len() < 12 || item.password.as_str() == "password123") {
            weak_passwords.push(item.id);
        }
    }

    let mut score = 100.0;
    if total_passwords_count > 0 {
        let reused_penalty = (total_reused_items as f64 / total_passwords_count as f64) * 40.0;
        let weak_penalty = (weak_passwords.len() as f64 / total_passwords_count as f64) * 30.0;
        let breached_penalty = (breached_passwords.len() as f64 / total_passwords_count as f64) * 50.0;
        
        score = (score - reused_penalty - weak_penalty - breached_penalty).max(0.0);
    }

    Ok(SecurityReport {
        reused_passwords,
        weak_passwords,
        breached_passwords,
        unique_passwords_count,
        total_passwords_count,
        overall_health_score: score,
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