#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;
mod backup_commands;
mod clipboard;
mod crypto;
mod db;
mod encryption;
mod error;
mod file_dialog;
mod security;
mod settings;
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
use zeroize::Zeroize;

#[tauri::command]
async fn is_database_loaded(state: State<'_, AppState>) -> Result<bool> {
    let guard = state.db.lock().await;
    Ok(guard.is_some())
}

#[tauri::command]
async fn get_active_db_path(state: State<'_, AppState>) -> Result<Option<String>> {
    let guard = state.db_path.lock().await;
    Ok(guard.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
async fn switch_database(db_path: PathBuf, app_state: State<'_, AppState>) -> Result<()> {
    // Check if the requested database is already active and loaded.
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

    // Capture current key and locking state before we close the old pool.
    let key_opt = {
        let guard = app_state.key.lock().await;
        guard.clone()
    };
    let key_slice_opt: Option<&[u8]> = key_opt.as_ref().map(|z| z.as_slice());
    
    // Use the rekey lock to serialize database switching operations.
    let _rekey_lock = app_state.rekey.lock().await;

    // Close the old database pool first to release file locks.
    {
        let mut guard = app_state.db.lock().await;
        if let Some(old_pool) = guard.take() {
            old_pool.close().await;
        }
    }

    // Small delay to ensure the OS has released any file system handles.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Initialize the new database pool.
    match crate::db::init_db(&db_path, key_slice_opt).await {
        Ok(new_pool) => {
            // Update the app state with the new pool and database path.
            let mut guard = app_state.db.lock().await;
            *guard = Some(new_pool);
            
            let mut path_guard = app_state.db_path.lock().await;
            *path_guard = Some(db_path.clone());
        }
        Err(e) => {
            // If it failed because it's encrypted or not a database yet, 
            // we still want to set the db_path so that it can be unlocked/initialized later.
            let mut path_guard = app_state.db_path.lock().await;
            *path_guard = Some(db_path.clone());

            // If we have no key and it failed, it's likely just encrypted, which is fine at this stage.
            if key_slice_opt.is_none() {
                eprintln!("Database at {} is encrypted or requires initialization.", db_path.display());
            } else {
                eprintln!(
                    "Failed to initialize database at {}: {}",
                    db_path.display(),
                    e
                );
                return Err(Error::Internal(e));
            }
        }
    };

    // Clear session-specific keys to ensure security.
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
            db::save_button,
            db::get_buttons,
            db::update_button,
            db::delete_button,
            db::remove_tag_from_password_items,
            db::rename_tag_in_password_items,
            db::save_password_item,
            db::get_password_items,
            db::update_password_item,
            db::update_password_item_tags,
            db::update_password_item_totp_secret,
            db::delete_password_item,
            db::wipe_vault_database,
            db::add_custom_field,
            db::add_attachment,
            db::delete_attachment,
            db::save_attachment_to_disk,
            db::save_recipient_key,
            db::get_recipient_keys,
            db::delete_recipient_key,
            db::save_profile_settings,
            db::get_profile_settings,
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
            settings::get_all_settings,
            settings::set_all_settings,
            clipboard::get_clipboard_capabilities,
            clipboard::apply_clipboard_policy,
            clipboard::clear_clipboard,
        ]);

    let app = builder.build(context).expect("error while building tauri application");
    app.run(|app_handle, event| {
        match event {
            RunEvent::ExitRequested { .. } | RunEvent::Exit => {
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