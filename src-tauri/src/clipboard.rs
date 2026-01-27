use serde::{Deserialize, Serialize};

use crate::state::{AppState, ClipboardPolicyState};
use crate::error::{Error, Result};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::Foundation::ERROR_SUCCESS,
    Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_READ, KEY_SET_VALUE, REG_CREATE_KEY_DISPOSITION, REG_DWORD, REG_OPTION_NON_VOLATILE,
        REG_SAM_FLAGS, REG_VALUE_TYPE, RRF_RT_REG_DWORD,
    },
};

#[cfg(target_os = "windows")]
const CLIPBOARD_POLICY_KEY: &str = "Software\\Microsoft\\Clipboard";
#[cfg(target_os = "windows")]
const CLIPBOARD_HISTORY_VALUE: &str = "EnableClipboardHistory";

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
    pub clear_after_duration: u64,
}

#[tauri::command]
pub async fn copy_to_clipboard(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<()> {
    let mut policy = state.clipboard_policy.lock().await;
    
    if !policy.integration_enabled {
        return Err(Error::Internal("Clipboard integration is disabled in settings.".to_string()));
    }

    if policy.only_unlocked {
        let db_guard = state.db.lock().await;
        if db_guard.is_none() {
            return Err(Error::VaultLocked);
        }
    }

    // Write to clipboard
    app.clipboard().write_text(text).map_err(|e| Error::Internal(e.to_string()))?;

    // Cancel existing clear task
    if let Some(handle) = policy.clear_task_handle.take() {
        handle.abort();
    }

    let duration = policy.clear_after_duration;
    if duration > 0 {
        let app_clone = app.clone();
        policy.clear_task_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(duration)).await;
            let _ = app_clone.clipboard().clear();
        }));
    }

    Ok(())
}

fn history_blocking_supported() -> bool {
    cfg!(target_os = "windows")
}

#[cfg(target_os = "windows")]
fn wide_null(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::prelude::OsStrExt;
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
fn read_history_enabled() -> Result<Option<u32>> {
    unsafe {
        let key_name = wide_null(CLIPBOARD_POLICY_KEY);
        let mut key: HKEY = HKEY::default();
        let mut disposition = REG_CREATE_KEY_DISPOSITION(0);
        let status = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            REG_SAM_FLAGS(KEY_READ.0 | KEY_SET_VALUE.0),
            None,
            &mut key,
            Some(&mut disposition as *mut _),
        );
        if status != ERROR_SUCCESS {
            return Err(Error::Internal(
                "Failed to open clipboard policy registry key.".to_string(),
            ));
        }

        let value_name = wide_null(CLIPBOARD_HISTORY_VALUE);
        let mut data: u32 = 0;
        let mut data_len = std::mem::size_of::<u32>() as u32;
        let mut reg_type = REG_VALUE_TYPE(0);
        let status = RegGetValueW(
            key,
            PCWSTR(std::ptr::null()),
            PCWSTR(value_name.as_ptr()),
            RRF_RT_REG_DWORD,
            Some(&mut reg_type),
            Some(&mut data as *mut _ as *mut _),
            Some(&mut data_len),
        );

        if status != ERROR_SUCCESS {
            let _ = RegCloseKey(key);
            return Ok(None);
        }

        let _ = RegCloseKey(key);
        Ok(Some(data))
    }
}

#[cfg(target_os = "windows")]
fn set_history_enabled(enabled: bool) -> Result<()> {
    unsafe {
        let key_name = wide_null(CLIPBOARD_POLICY_KEY);
        let mut key: HKEY = HKEY::default();
        let mut disposition = REG_CREATE_KEY_DISPOSITION(0);
        let status = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            REG_SAM_FLAGS(KEY_READ.0 | KEY_SET_VALUE.0),
            None,
            &mut key,
            Some(&mut disposition as *mut _),
        );
        if status != ERROR_SUCCESS {
            return Err(Error::Internal(
                "Failed to open clipboard policy registry key.".to_string(),
            ));
        }

        let value_name = wide_null(CLIPBOARD_HISTORY_VALUE);
        let value: u32 = if enabled { 1 } else { 0 };
        let value_bytes = value.to_le_bytes();
        let status = RegSetValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            0,
            REG_DWORD,
            Some(&value_bytes),
        );

        let _ = RegCloseKey(key);
        if status != ERROR_SUCCESS {
            return Err(Error::Internal(
                "Failed to update clipboard history policy.".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn read_history_enabled() -> Result<Option<u32>> {
    Ok(None)
}

#[cfg(not(target_os = "windows"))]
fn set_history_enabled(_enabled: bool) -> Result<()> {
    Ok(())
}

fn build_status(policy: &ClipboardPolicyState) -> ClipboardPolicyStatus {
    let supported = history_blocking_supported();
    let history_enabled = read_history_enabled().ok().flatten();
    let history_blocked = history_enabled.map(|value| value == 0).unwrap_or(false);
    ClipboardPolicyStatus {
        integration_available: policy.integration_enabled,
        history_blocking_supported: supported,
        history_blocking_active: policy.block_history
            && policy.integration_enabled
            && supported
            && history_blocked,
    }
}

pub fn restore_clipboard_history(policy: &ClipboardPolicyState) {
    if policy.block_history {
        if let Some(previous) = policy.prior_history_setting {
            let _ = set_history_enabled(previous != 0);
        }
    }
}

#[tauri::command]
pub async fn get_clipboard_capabilities(
    _app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ClipboardPolicyStatus> {
    let policy = state.clipboard_policy.lock().await;
    Ok(build_status(&policy))
}

#[tauri::command]
pub async fn apply_clipboard_policy(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    payload: ClipboardPolicyPayload,
) -> Result<ClipboardPolicyStatus> {
    if payload.block_history && !history_blocking_supported() {
        return Err(Error::Internal("Clipboard history blocking is not supported on this platform.".to_string()));
    }

    let mut policy = state.clipboard_policy.lock().await;
    let was_blocking = policy.block_history;
    policy.integration_enabled = payload.clipboard_integration;
    policy.only_unlocked = payload.only_unlocked;
    policy.clear_after_duration = payload.clear_after_duration;
    policy.block_history = payload.block_history && history_blocking_supported();

    if let Some(handle) = policy.clear_task_handle.take() {
        handle.abort();
    }

    if policy.block_history {
        if !was_blocking {
            policy.prior_history_setting = read_history_enabled().ok().flatten();
        }
        set_history_enabled(false)?;
        if let Err(error) = app.clipboard().clear() {
            eprintln!("Failed to clear clipboard after policy update: {}", error);
        }
    } else if was_blocking {
        if let Some(previous) = policy.prior_history_setting.take() {
            set_history_enabled(previous != 0)?;
        }
    }

    Ok(build_status(&policy))
}

#[tauri::command]
pub async fn clear_clipboard(app: tauri::AppHandle) -> Result<()> {
    app.clipboard().clear().map_err(|error| Error::Internal(error.to_string()))
}

