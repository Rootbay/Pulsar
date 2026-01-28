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
mod utils;
mod vault_commands;

use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

use crate::auth::UNLOCK_CONCURRENCY_LIMIT;
use crate::state::AppState;
use tauri::{Manager, RunEvent};

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
            vault_commands::is_database_loaded,
            vault_commands::get_active_db_path,
            vault_commands::switch_database,
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

    let app = builder
        .build(context)
        .expect("error while building tauri application");
    app.run(|app_handle, event| match event {
        RunEvent::ExitRequested { .. } | RunEvent::Exit => {
            tauri::async_runtime::block_on(async {
                let state = app_handle.state::<AppState>();
                let policy = state.clipboard_policy.lock().await;
                clipboard::restore_clipboard_history(&policy);
            });
        }
        _ => {}
    });
}
