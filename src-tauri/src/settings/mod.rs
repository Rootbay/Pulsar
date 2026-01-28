use crate::encryption::{decrypt, encrypt};
use crate::error::{Error, Result};
use keyring::Entry;
use sha2::Sha256;
use tauri_plugin_store::StoreBuilder;

pub mod system;
pub use system::*;

const SETTINGS_KEYRING_SERVICE: &str = "Pulsar-App-Settings-v4";
const SETTINGS_KEYRING_USER: &str = "StableRoot";
const SETTINGS_SALT: &[u8] = b"pulsar-v4-hardware-bound-salt";

static SETTINGS_KEY_CACHE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
static STORE_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

fn get_hardware_id() -> String {
    #[cfg(target_os = "windows")]
    {
        use windows::core::HSTRING;
        use windows::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
        };

        let sub_key = HSTRING::from("SOFTWARE\\Microsoft\\Cryptography");
        let value_name = HSTRING::from("MachineGuid");
        let mut h_key = HKEY::default();

        unsafe {
            if RegOpenKeyExW(HKEY_LOCAL_MACHINE, &sub_key, 0, KEY_READ, &mut h_key).is_ok() {
                let mut data_type = windows::Win32::System::Registry::REG_VALUE_TYPE::default();
                let mut data_len = 0u32;

                if RegQueryValueExW(
                    h_key,
                    &value_name,
                    None,
                    Some(&mut data_type),
                    None,
                    Some(&mut data_len),
                )
                .is_ok()
                {
                    let mut buffer = vec![0u16; (data_len / 2) as usize];
                    if RegQueryValueExW(
                        h_key,
                        &value_name,
                        None,
                        None,
                        Some(buffer.as_mut_ptr() as *mut u8),
                        Some(&mut data_len),
                    )
                    .is_ok()
                    {
                        let _ = RegCloseKey(h_key);
                        let guid = String::from_utf16_lossy(&buffer);
                        return guid.trim_matches('\0').to_string();
                    }
                }
                let _ = RegCloseKey(h_key);
            }
        }
    }
    "fallback-stable-id-pulsar".to_string()
}

fn get_or_create_settings_key() -> Result<Vec<u8>> {
    if let Some(key) = SETTINGS_KEY_CACHE.get() {
        return Ok(key.clone());
    }

    let entry = Entry::new(SETTINGS_KEYRING_SERVICE, SETTINGS_KEYRING_USER)
        .map_err(|e| Error::Internal(format!("Keyring init error: {e}")))?;

    // Attempt to get the existing secret or use the hardware ID as a stable seed
    let stable_seed = match entry.get_password() {
        Ok(s) => s,
        Err(_) => {
            let hw_id = get_hardware_id();
            // Fallback to hardware-bound ID if keyring is unavailable
            let _ = entry.set_password(&hw_id);
            hw_id
        }
    };

    use hkdf::Hkdf;

    let hk = Hkdf::<Sha256>::new(Some(SETTINGS_SALT), stable_seed.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(b"pulsar-settings-v4-hkdf-expansion", &mut okm)
        .map_err(|_| Error::Internal("Key derivation failed".to_string()))?;

    let key = okm.to_vec();

    let _ = SETTINGS_KEY_CACHE.set(key.clone());

    // Best-effort cleanup of old legacy entries
    let _ =
        Entry::new("pulsar-v3-settings", SETTINGS_KEYRING_USER).and_then(|e| e.delete_credential());
    let _ =
        Entry::new("pulsar-settings-v2", SETTINGS_KEYRING_USER).and_then(|e| e.delete_credential());
    let _ =
        Entry::new("pulsar-v1-settings", SETTINGS_KEYRING_USER).and_then(|e| e.delete_credential());
    let _ =
        Entry::new("pulsar-settings", SETTINGS_KEYRING_USER).and_then(|e| e.delete_credential());

    Ok(key)
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
    let _guard = STORE_MUTEX.lock().await;
    use tauri::Manager;
    let settings_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| Error::Internal(e.to_string()))?
        .join(".settings.dat");

    let store = StoreBuilder::new(app_handle, settings_path.clone())
        .build()
        .map_err(|e| Error::Internal(e.to_string()))?;

    store.reload().ok();

    if let Some(encrypted_val) = store.get("settings_encrypted") {
        let encrypted_str = encrypted_val
            .as_str()
            .ok_or_else(|| Error::Internal("Invalid encrypted settings format".to_string()))?;

        let key = get_or_create_settings_key()?;
        match decrypt(encrypted_str, &key) {
            Ok(decrypted) => {
                return Ok(Some(decrypted));
            }
            Err(_) => {
                store.delete("settings_encrypted");
                let _ = store.save();
                return Ok(None);
            }
        }
    }

    let plaintext = store.get("settings").and_then(|v| {
        v.as_str()
            .map(|s| s.to_string())
            .or_else(|| Some(v.to_string()))
    });

    if let Some(settings_str) = plaintext {
        let key = get_or_create_settings_key()?;
        let encrypted = encrypt(&settings_str, &key)?;
        store.set(
            "settings_encrypted".to_string(),
            serde_json::Value::String(encrypted),
        );
        store.delete("settings");
        if let Err(e) = store.save() {
            eprintln!("[Settings] Failed to migrate plaintext settings: {}", e);
        }
        return Ok(Some(settings_str));
    }

    Ok(None)
}

#[tauri::command]
pub async fn set_all_settings(app_handle: tauri::AppHandle, settings: String) -> Result<()> {
    let _guard = STORE_MUTEX.lock().await;
    use tauri::Manager;
    let settings_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| Error::Internal(e.to_string()))?
        .join(".settings.dat");

    let store = StoreBuilder::new(&app_handle, settings_path)
        .build()
        .map_err(|e| Error::Internal(e.to_string()))?;

    store.reload().ok();

    let key = get_or_create_settings_key()?;
    let encrypted = encrypt(&settings, &key)?;

    (*store).set(
        "settings_encrypted".to_string(),
        serde_json::Value::String(encrypted),
    );
    (*store).delete("settings");

    match (*store).save() {
        Ok(()) => {}
        Err(e) => return Err(Error::Internal(e.to_string())),
    }
    Ok(())
}
