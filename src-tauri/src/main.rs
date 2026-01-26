#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;
mod backup_commands;
mod clipboard;
mod crypto;
mod db;
mod db_commands;
mod encryption;
mod error;
mod file_dialog;
mod security;
mod state;
mod totp;
mod types;
mod vault_commands;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

use crate::state::AppState;
use crate::auth::UNLOCK_CONCURRENCY_LIMIT;
use crate::error::{Error, Result};
use tauri::{Manager, RunEvent, State};
use tauri_plugin_store::StoreBuilder;
use zeroize::Zeroize;

#[tauri::command]
async fn is_database_loaded(app_state: State<'_, AppState>) -> Result<bool> {
    Ok(app_state.db.lock().await.is_some())
}

#[tauri::command]
async fn get_active_db_path(app_state: State<'_, AppState>) -> Result<Option<String>> {
    let guard = app_state.db_path.lock().await;
    Ok(guard.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
async fn switch_database(db_path: PathBuf, app_state: State<'_, AppState>) -> Result<()> {
    {
        let path_guard = app_state.db_path.lock().await;
        if let Some(active_path) = path_guard.as_ref() {
            if active_path == &db_path {
                let db_guard = app_state.db.lock().await;
                if db_guard.is_some() {
                    return Ok(());
                }
            }
        }
    }

    let key_opt = {
        let guard = app_state.key.lock().await;
        guard.clone()
    };
    let key_slice_opt: Option<&[u8]> = key_opt.as_ref().map(|z| z.as_slice());
    let _rekey_lock = app_state.rekey.lock().await;

    let mut last_err = None;
    let mut new_pool_opt = None;
    for _ in 0..5 {
        match crate::db::init_db(&db_path, key_slice_opt).await {
            Ok(pool) => {
                new_pool_opt = Some(pool);
                last_err = None;
                break;
            }
            Err(e) => {
                last_err = Some(e);
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }
    }

    let new_pool = match new_pool_opt {
        Some(pool) => pool,
        None => {
            let e = last_err.unwrap_or_else(|| "Unknown error".to_string());
            eprintln!(
                "Failed to initialize database at {}: {}",
                db_path.display(),
                e
            );
            return Err(Error::Internal(e));
        }
    };

    {
        let mut guard = app_state.db.lock().await;
        if let Some(old_pool) = guard.take() {
            old_pool.close().await;
        }
        *guard = Some(new_pool);
    }

    {
        let mut path_guard = app_state.db_path.lock().await;
        *path_guard = Some(db_path.clone());
    }

    {
        let mut kg = app_state.key.lock().await;
        *kg = None;
    }

    {
        let mut pending = app_state.pending_key.lock().await;
        if let Some(mut key) = pending.take() {
            key.key.zeroize();
        }
    }

    Ok(())
}

fn main() {

    let context = tauri::generate_context!();
    let mut builder = tauri::Builder::default()
        .manage(AppState {
            db: Arc::new(Mutex::new(None)),
            key: Arc::new(Mutex::new(None)),
            pending_key: Arc::new(Mutex::new(None)),
            db_path: Arc::new(Mutex::new(None)),
            rekey: Arc::new(Mutex::new(())),
            clipboard_policy: Arc::new(Mutex::new(Default::default())),
            unlock_rate_limit: Arc::new(Mutex::new(Default::default())),
            unlock_guard: Arc::new(Semaphore::new(UNLOCK_CONCURRENCY_LIMIT)),
        })
        .plugin(tauri_plugin_clipboard_manager::init());

    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_biometric::init());
    }

    builder = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            is_database_loaded,
            get_active_db_path,
            switch_database,
            auth::set_master_password,
            auth::unlock,
            auth::verify_login_totp,
            auth::configure_login_totp,
            auth::disable_login_totp,
            auth::is_login_totp_configured,
            auth::get_argon2_params,
            auth::rotate_master_password,
            auth::update_argon2_params,
            auth::verify_master_password,
            auth::lock,
            auth::is_locked,
            auth::is_master_password_configured,
            auth::enable_biometrics,
            auth::disable_biometrics,
            auth::is_biometrics_enabled,
            auth::unlock_with_biometrics,
            db_commands::save_button,
            db_commands::get_buttons,
            db_commands::update_button,
            db_commands::delete_button,
            db_commands::remove_tag_from_password_items,
            db_commands::rename_tag_in_password_items,
            db_commands::save_password_item,
            db_commands::get_password_items,
            db_commands::update_password_item,
            db_commands::update_password_item_tags,
            db_commands::update_password_item_totp_secret,
            db_commands::delete_password_item,
            db_commands::wipe_vault_database,
            db_commands::add_custom_field,
            db_commands::add_attachment,
            db_commands::delete_attachment,
            db_commands::save_attachment_to_disk,
            db_commands::save_recipient_key,
            db_commands::get_recipient_keys,
            db_commands::delete_recipient_key,
            db_commands::save_profile_settings,
            db_commands::get_profile_settings,
            crypto::export_password_entry,
            crypto::generate_x25519_keypair,
            crypto::export_password_entry_to_public_key,
            crypto::import_password_entry_with_private_key,
            totp::generate_totp_secret,
            totp::generate_totp,
            totp::verify_totp_secret,
            totp::verify_totp,
            file_dialog::pick_open_file,
            file_dialog::pick_save_file,
            file_dialog::elevated_copy,
            file_dialog::check_file_exists,
            backup_commands::export_vault,
            backup_commands::export_vault_backend,
            backup_commands::import_vault,
            backup_commands::restore_vault_backend,
            backup_commands::restore_vault_snapshot,
            vault_commands::list_vaults,
            security::list_devices,
            security::remove_device,
            security::revoke_all_devices,
            security::wipe_memory,
            security::get_security_report,
            security::run_integrity_check,
            get_all_settings,
            set_all_settings,
            clipboard::get_clipboard_capabilities,
            clipboard::apply_clipboard_policy,
            clipboard::clear_clipboard,
        ]);

    let app = builder.build(context).expect("error while building tauri application");
    app.run(|app_handle, event| {
        match event {
            RunEvent::ExitRequested { .. } | RunEvent::Exit { .. } => {
                tauri::async_runtime::block_on(async {
                    let state = app_handle.state::<AppState>();
                    let policy = state.clipboard_policy.lock().await;
                    clipboard::restore_clipboard_history(&policy);
                });
            }
            _ => {}
        }
    });
}

use keyring::Entry;
use crate::encryption::{encrypt, decrypt};
use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};

const SETTINGS_KEYRING_SERVICE: &str = "pulsar-settings";
const SETTINGS_KEYRING_USER: &str = "encryption-key";

fn get_or_create_settings_key() -> Result<Vec<u8>> {
    let entry = Entry::new(SETTINGS_KEYRING_SERVICE, SETTINGS_KEYRING_USER).map_err(|e| Error::Internal(e.to_string()))?;
    match entry.get_password() {
        Ok(key_b64) => {
            general_purpose::STANDARD.decode(&key_b64).map_err(|e| Error::Internal(e.to_string()))
        }
        Err(keyring::Error::NoEntry) => {
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);
            let key_b64 = general_purpose::STANDARD.encode(key);
            entry.set_password(&key_b64).map_err(|e| Error::Internal(e.to_string()))?;
            Ok(key.to_vec())
        }
        Err(e) => Err(Error::Internal(e.to_string())),
    }
}

#[tauri::command]
async fn get_all_settings(app_handle: tauri::AppHandle) -> Result<Option<String>> {
    let store = StoreBuilder::new(&app_handle, PathBuf::from(".settings.dat"))
        .build()
        .map_err(|e| Error::Internal(e.to_string()))?;
    store.reload().map_err(|e| Error::Internal(e.to_string()))?;

    if let Some(encrypted_val) = store.get("settings_encrypted") {
        let encrypted_str = encrypted_val.as_str().ok_or_else(|| Error::Internal("Invalid encrypted settings format".to_string()))?;
        let key = get_or_create_settings_key()?;
        let decrypted = decrypt(encrypted_str, &key)
            .map_err(|_| Error::Internal("Failed to decrypt settings.".to_string()))?;
        return Ok(Some(decrypted));
    }

    let plaintext = store.get("settings").and_then(|v| {
        v.as_str()
            .map(|s| s.to_string())
            .or_else(|| Some(v.to_string()))
    });

    if let Some(settings) = plaintext {
        let key = get_or_create_settings_key()?;
        let encrypted = encrypt(&settings, &key)?;
        store.set("settings_encrypted".to_string(), serde_json::Value::String(encrypted));
        store.delete("settings");
        store.save().map_err(|e| Error::Internal(e.to_string()))?;
        return Ok(Some(settings));
    }

    Ok(None)
}

#[tauri::command]
async fn set_all_settings(app_handle: tauri::AppHandle, settings: String) -> Result<()> {
    let store = StoreBuilder::new(&app_handle, PathBuf::from(".settings.dat"))
        .build()
        .map_err(|e| Error::Internal(e.to_string()))?;
    store.reload().map_err(|e| Error::Internal(e.to_string()))?;

    let key = get_or_create_settings_key()?;
    let encrypted = encrypt(&settings, &key)?;

    (*store).set("settings_encrypted".to_string(), serde_json::Value::String(encrypted));
    (*store).delete("settings");

    match (*store).save() {
        Ok(()) => {}
        Err(e) => return Err(Error::Internal(e.to_string())),
    }
    Ok(())
}

