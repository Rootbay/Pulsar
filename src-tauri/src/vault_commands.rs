use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;
use crate::error::{Error, Result};
use crate::state::AppState;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StoredVaultSettings {
    pub name: String,
    pub totp: bool,
    pub backups: bool,
    pub compression: bool,
}

impl Default for StoredVaultSettings {
    fn default() -> Self {
        Self {
            name: String::new(),
            totp: true,
            backups: false,
            compression: false,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct StoredAppSettings {
    #[serde(default)]
    recent_databases: Vec<String>,
    #[serde(default, rename = "vaultSettingsById")]
    vault_settings_by_id: HashMap<String, StoredVaultSettings>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultInfo {
    pub id: String,
    pub path: String,
    pub name: String,
    pub status: String,
    pub encrypted: bool,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<i64>,
    pub item_count: Option<u64>,
    pub settings: StoredVaultSettings,
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

async fn load_stored_settings(app_handle: &tauri::AppHandle) -> Result<StoredAppSettings> {
    use crate::settings::get_all_settings_internal;
    
    let settings_json = get_all_settings_internal(app_handle).await?;

    let Some(raw) = settings_json else {
        return Ok(StoredAppSettings::default());
    };

    serde_json::from_str::<StoredAppSettings>(&raw)
        .map_err(|e| Error::Internal(format!("Failed to parse stored settings: {e}")))
}

fn gather_ordered_paths(
    settings: &StoredAppSettings,
    active_path: &Option<PathBuf>,
) -> Vec<String> {
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();

    if let Some(path) = active_path {
        if let Some(str_path) = path.to_str() {
            if seen.insert(str_path.to_string()) {
                ordered.push(str_path.to_string());
            }
        }
    }

    for path in &settings.recent_databases {
        if seen.insert(path.clone()) {
            ordered.push(path.clone());
        }
    }

    for path in settings.vault_settings_by_id.keys() {
        if seen.insert(path.clone()) {
            ordered.push(path.clone());
        }
    }

    ordered
}

async fn resolve_item_count(pool: Option<SqlitePool>, include: bool) -> Option<u64> {
    if !include {
        return None;
    }

    let db_pool = pool?;

    match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM password_items")
        .fetch_one(&db_pool)
        .await
    {
        Ok(count) => Some(count.max(0) as u64),
        Err(error) => {
            eprintln!("Failed to count password items: {error}");
            None
        }
    }
}

#[tauri::command]
pub async fn list_vaults(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<VaultInfo>> {
    let stored_settings = load_stored_settings(&app_handle).await?;

    let active_path = { state.db_path.lock().await.clone() };
    let active_pool = { state.db.lock().await.clone() };
    let is_unlocked = state.key.lock().await.is_some();

    let ordered_paths = gather_ordered_paths(&stored_settings, &active_path);

    let mut results = Vec::new();

    for path_str in ordered_paths {
        let path = PathBuf::from(&path_str);
        let metadata = match tokio::fs::metadata(&path).await {
            Ok(meta) => meta,
            Err(err) => {
                eprintln!("Failed to stat vault {}: {}", path.display(), err);
                continue;
            }
        };

        let size_bytes = Some(metadata.len());
        let modified_at = metadata.modified().ok().and_then(|mtime| {
            mtime
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|duration| duration.as_millis() as i64)
        });

        let settings = stored_settings
            .vault_settings_by_id
            .get(&path_str)
            .cloned()
            .unwrap_or_default();

        let display_name = if settings.name.trim().is_empty() {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("Vault")
                .to_string()
        } else {
            settings.name.clone()
        };

        let is_active = active_path
            .as_ref()
            .map(|active| active == &path)
            .unwrap_or(false);

        let status = if is_active {
            if is_unlocked {
                "unlocked"
            } else {
                "locked"
            }
        } else {
            "available"
        };

        let encrypted = tokio::fs::try_exists(metadata_path(&path)).await.unwrap_or(false);

        let item_count = resolve_item_count(active_pool.clone(), is_active && is_unlocked).await;

        results.push(VaultInfo {
            id: path_str.clone(),
            path: path_str,
            name: display_name,
            status: status.to_string(),
            encrypted,
            size_bytes,
            modified_at,
            item_count,
            settings,
        });
    }

    Ok(results)
}

