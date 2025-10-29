use serde::{Deserialize, Serialize};

use crate::state::{AppState, ClipboardPolicyState};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardPolicyStatus {
    pub integration_available: bool,
    pub history_blocking_supported: bool,
    pub history_blocking_active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardPolicyPayload {
    pub clipboard_integration: bool,
    pub block_history: bool,
    pub only_unlocked: bool,
}

fn history_blocking_supported() -> bool {
    cfg!(target_os = "windows")
}

fn build_status(policy: &ClipboardPolicyState) -> ClipboardPolicyStatus {
    let supported = history_blocking_supported();
    ClipboardPolicyStatus {
        integration_available: policy.integration_enabled,
        history_blocking_supported: supported,
        history_blocking_active: policy.block_history && policy.integration_enabled && supported,
    }
}

#[tauri::command]
pub async fn get_clipboard_capabilities(
    _app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ClipboardPolicyStatus, String> {
    let policy = state.clipboard_policy.lock().await;
    Ok(build_status(&policy))
}

#[tauri::command]
pub async fn apply_clipboard_policy(
    state: tauri::State<'_, AppState>,
    payload: ClipboardPolicyPayload,
) -> Result<ClipboardPolicyStatus, String> {
    if payload.block_history && !history_blocking_supported() {
        return Err("Clipboard history blocking is not supported on this platform.".to_string());
    }

    let mut policy = state.clipboard_policy.lock().await;
    policy.integration_enabled = payload.clipboard_integration;
    policy.only_unlocked = payload.only_unlocked;
    policy.block_history = payload.block_history && history_blocking_supported();

    Ok(build_status(&policy))
}

#[tauri::command]
pub async fn clear_clipboard(app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard().clear().map_err(|error| error.to_string())
}
