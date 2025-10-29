#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;
mod backup_commands;
mod crypto;
mod db;
mod db_commands;
mod encryption;
mod file_dialog;
mod security;
mod state;
mod totp;
mod types;
mod vault_commands;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use tauri::State;
use tauri_plugin_store::StoreBuilder;

#[tauri::command]
async fn is_database_loaded(app_state: State<'_, AppState>) -> Result<bool, String> {
    Ok(app_state.db.lock().await.is_some())
}

#[tauri::command]
async fn switch_database(db_path: PathBuf, app_state: State<'_, AppState>) -> Result<(), String> {
    let key_opt = {
        let guard = app_state.key.lock().await;
        guard.clone()
    };
    let key_slice_opt: Option<&[u8]> = key_opt.as_ref().map(|z| z.as_slice());
    let _rekey_lock = app_state.rekey.lock().await;

    let new_pool = match crate::db::init_db(&db_path, key_slice_opt).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!(
                "Failed to initialize database at {}: {}",
                db_path.display(),
                e
            );
            return Err(e);
        }
    };

    {
        let mut guard = app_state.db.lock().await;
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
        *pending = None;
    }

    Ok(())
}

fn main() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .manage(AppState {
            db: Arc::new(Mutex::new(None)),
            key: Arc::new(Mutex::new(None)),
            pending_key: Arc::new(Mutex::new(None)),
            db_path: Arc::new(Mutex::new(None)),
            rekey: Arc::new(Mutex::new(())),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            is_database_loaded,
            switch_database,
            // Auth commands
            auth::set_master_password,
            auth::unlock,
            auth::verify_login_totp,
            auth::configure_login_totp,
            auth::disable_login_totp,
            auth::is_login_totp_configured,
            auth::get_argon2_params,
            auth::rotate_master_password,
            auth::update_argon2_params,
            auth::lock,
            auth::is_locked,
            auth::is_master_password_configured,
            // DB commands
            db_commands::save_button,
            db_commands::get_buttons,
            db_commands::update_button,
            db_commands::delete_button,
            db_commands::save_password_item,
            db_commands::get_password_items,
            db_commands::update_password_item,
            db_commands::update_password_item_tags,
            db_commands::update_password_item_totp_secret,
            db_commands::delete_password_item,
            db_commands::add_custom_field,
            db_commands::save_recipient_key,
            db_commands::get_recipient_keys,
            db_commands::delete_recipient_key,
            // Crypto / export commands
            crypto::export_password_entry,
            crypto::generate_x25519_keypair,
            crypto::export_password_entry_to_public_key,
            crypto::import_password_entry_with_private_key,
            // TOTP commands for vault items
            totp::generate_totp_secret,
            totp::generate_totp,
            totp::verify_totp,
            // File Dialog commands
            file_dialog::pick_open_file,
            file_dialog::pick_save_file,
            file_dialog::elevated_copy,
            file_dialog::check_file_exists,
            // Backup commands
            backup_commands::export_vault,
            backup_commands::import_vault,
            backup_commands::restore_vault_snapshot,
            vault_commands::list_vaults,
            // Security commands
            security::list_devices,
            security::remove_device,
            security::revoke_all_devices,
            security::wipe_memory,
            security::run_integrity_check,
            get_all_settings,
            set_all_settings,
        ])
        .run(context)
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_all_settings(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = StoreBuilder::new(&app_handle, ".settings.dat".parse::<PathBuf>().unwrap())
        .build()
        .map_err(|e| e.to_string())?;
    store.reload().map_err(|e| e.to_string())?; // Changed load() to reload()
    Ok(store.get("settings").map(|v| v.to_string()))
}

#[tauri::command]
async fn set_all_settings(app_handle: tauri::AppHandle, settings: String) -> Result<(), String> {
    let store = StoreBuilder::new(&app_handle, ".settings.dat".parse::<PathBuf>().unwrap())
        .build()
        .map_err(|e| e.to_string())?;
    store.reload().map_err(|e| e.to_string())?;

    (*store).set("settings".to_string(), settings);

    match (*store).save() {
        Ok(()) => {}
        Err(e) => return Err(e.to_string()),
    }
    Ok(())
}
