use tauri_plugin_store::StoreBuilder;
use keyring::Entry;
use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};
use crate::error::{Error, Result};
use crate::encryption::{encrypt, decrypt};

pub mod system;
pub use system::*;

const SETTINGS_KEYRING_SERVICE: &str = "pulsar-settings";
const SETTINGS_KEYRING_USER: &str = "encryption-key";

#[derive(serde::Deserialize)]
struct AllSettings {
    general: GeneralSettings,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeneralSettings {
    start_on_system_boot: bool,
    show_in_system_tray: bool,
}

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
pub async fn apply_system_settings(app_handle: tauri::AppHandle) -> Result<()> {
    if let Some(settings_json) = get_all_settings(app_handle.clone()).await? {
        if let Ok(settings) = serde_json::from_str::<AllSettings>(&settings_json) {
            system::set_autostart(settings.general.start_on_system_boot)?;
            
            if settings.general.show_in_system_tray {
                if app_handle.tray_by_id("main").is_none() {
                    let _ = crate::tray::setup_tray(&app_handle);
                } else if let Some(tray) = app_handle.tray_by_id("main") {
                    let _ = tray.set_visible(true);
                }
            } else {
                if let Some(tray) = app_handle.tray_by_id("main") {
                    let _ = tray.set_visible(false);
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_all_settings(app_handle: tauri::AppHandle) -> Result<Option<String>> {
    get_all_settings_internal(&app_handle).await
}

pub async fn get_all_settings_internal(app_handle: &tauri::AppHandle) -> Result<Option<String>> {
    use tauri::Manager;
    let settings_path = app_handle.path().app_data_dir().map_err(|e| Error::Internal(e.to_string()))?.join(".settings.dat");
    let store = StoreBuilder::new(app_handle, settings_path)
        .build()
        .map_err(|e| Error::Internal(e.to_string()))?;
    store.reload().map_err(|e| Error::Internal(e.to_string()))?;

    if let Some(encrypted_val) = store.get("settings_encrypted") {
        let encrypted_str = encrypted_val.as_str().ok_or_else(|| Error::Internal("Invalid encrypted settings format".to_string()))?;
        let key = get_or_create_settings_key()?;
        match decrypt(encrypted_str, &key) {
            Ok(decrypted) => return Ok(Some(decrypted)),
            Err(e) => {
                eprintln!("Failed to decrypt settings: {}. Settings might be corrupted or encryption key changed. Falling back to defaults.", e);
                return Ok(None);
            }
        }
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
pub async fn set_all_settings(app_handle: tauri::AppHandle, settings: String) -> Result<()> {
    use tauri::Manager;
    let settings_path = app_handle.path().app_data_dir().map_err(|e| Error::Internal(e.to_string()))?.join(".settings.dat");
    let store = StoreBuilder::new(&app_handle, settings_path)
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