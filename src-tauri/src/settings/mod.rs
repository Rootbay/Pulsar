use tauri_plugin_store::StoreBuilder;
use keyring::Entry;
use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};
use crate::error::{Error, Result};
use crate::encryption::{encrypt, decrypt};

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
pub async fn get_all_settings(app_handle: tauri::AppHandle) -> Result<Option<String>> {
    use tauri::Manager;
    let settings_path = app_handle.path().app_data_dir().map_err(|e| Error::Internal(e.to_string()))?.join(".settings.dat");
    let store = StoreBuilder::new(&app_handle, settings_path)
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