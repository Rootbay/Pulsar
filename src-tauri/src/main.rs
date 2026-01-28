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
mod tray;
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
    let _rekey_lock = app_state.rekey.lock().await;

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

    {
        let mut guard = app_state.db.lock().await;
        if let Some(old_pool) = guard.take() {
            old_pool.close().await;
        }
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

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    match crate::db::init_db_lazy(&db_path, None, true).await {
        Ok(new_pool) => {
            let mut guard = app_state.db.lock().await;
            *guard = Some(new_pool);
            
            let mut path_guard = app_state.db_path.lock().await;
            *path_guard = Some(db_path.clone());
        }
        Err(e) => {
            eprintln!(
                "Failed to initialize database pool at {}: {}",
                db_path.display(),
                e
            );
            return Err(Error::Internal(e));
        }
    };

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
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            auth::get_login_totp_secret,
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
            db::search_password_items,
            db::get_password_overviews,
            db::get_password_overviews_by_ids,
            db::get_password_item_by_id,
            db::update_password_item,
            db::update_password_item_tags,
            db::update_password_item_totp_secret,
            db::delete_password_item,
            db::wipe_vault_database,
            db::add_custom_field,
            db::add_attachment,
            db::import_file_as_attachment,
            db::export_attachment_to_file,
            db::delete_attachment,
            db::save_attachment_to_disk,
            db::save_recipient_key,
            db::get_recipient_keys,
            db::delete_recipient_key,
            db::get_activity_log,
            db::clear_activity_log,
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
            file_dialog::open_app_data_folder,
            file_dialog::clear_app_logs,
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
            settings::apply_system_settings,
            settings::simulate_autotype,
            clipboard::get_clipboard_capabilities,
            clipboard::apply_clipboard_policy,
            clipboard::copy_to_clipboard,
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