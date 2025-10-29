use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::SqlitePool;
use tauri::State;
use zeroize::Zeroize;

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

async fn get_db_pool(state: &State<'_, AppState>) -> Result<SqlitePool, String> {
    let guard = state.db.lock().await;
    guard
        .clone()
        .ok_or_else(|| "Vault database is not loaded.".to_string())
}

async fn load_devices(pool: &SqlitePool) -> Result<Vec<DeviceRecord>, String> {
    let stored: Option<String> =
        sqlx::query_scalar("SELECT value FROM configuration WHERE key = 'device_registry'")
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    if let Some(json) = stored {
        if json.trim().is_empty() {
            Ok(Vec::new())
        } else {
            serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse device registry: {}", e))
        }
    } else {
        Ok(Vec::new())
    }
}

async fn save_devices(pool: &SqlitePool, devices: &[DeviceRecord]) -> Result<(), String> {
    let payload = serde_json::to_string(devices).map_err(|e| e.to_string())?;
    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES ('device_registry', ?)")
        .bind(payload)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<Vec<DeviceRecord>, String> {
    let pool = get_db_pool(&state).await?;
    let mut devices = load_devices(&pool).await?;
    devices
        .iter_mut()
        .filter(|d| d.kind.trim().is_empty())
        .for_each(|d| d.kind = "unknown".to_string());

    Ok(devices)
}

#[tauri::command]
pub async fn remove_device(state: State<'_, AppState>, device_id: String) -> Result<(), String> {
    let pool = get_db_pool(&state).await?;
    let mut devices = load_devices(&pool).await?;
    let original_len = devices.len();
    devices.retain(|device| device.id != device_id);

    if devices.len() == original_len {
        return Err("Device not found".to_string());
    }

    save_devices(&pool, &devices).await
}

#[tauri::command]
pub async fn revoke_all_devices(state: State<'_, AppState>) -> Result<(), String> {
    let pool = get_db_pool(&state).await?;
    save_devices(&pool, &[]).await
}

#[tauri::command]
pub async fn wipe_memory(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut key_guard = state.key.lock().await;
        if let Some(mut key) = key_guard.take() {
            key.zeroize();
        }
    }

    {
        let mut pending_guard = state.pending_key.lock().await;
        if let Some(mut key) = pending_guard.take() {
            key.zeroize();
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

#[tauri::command]
pub async fn run_integrity_check(state: State<'_, AppState>) -> Result<String, String> {
    let pool = get_db_pool(&state).await?;
    let result: (String,) = sqlx::query_as("PRAGMA integrity_check;")
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.0)
}
