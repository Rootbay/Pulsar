use tauri::{AppHandle, State};
use crate::error::{Error, Result};
use crate::state::AppState;
use crate::auth::metadata::get_vault_id;
use crate::encryption::{encrypt, decrypt};
use keyring::Entry;
use zeroize::{Zeroize, Zeroizing};
use rand::rngs::OsRng;
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};

const KEYRING_SERVICE: &str = "pulsar-vault";

#[cfg(mobile)]
use tauri_plugin_biometric::{AuthOptions, BiometricExt, BiometryType};

#[cfg(target_os = "windows")]
use windows::{
    core::HSTRING,
    Security::Credentials::UI::{
        UserConsentVerifier, UserConsentVerifierAvailability, UserConsentVerificationResult,
    },
};

#[cfg(target_os = "macos")]
use {
    block::ConcreteBlock,
    dispatch::Semaphore,
    objc::{class, msg_send, sel, sel_impl},
    objc::runtime::Object,
    objc_foundation::NSString,
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub fn ensure_biometric_available(_app: &AppHandle) -> Result<()> {
    #[cfg(mobile)]
    {
        let status = app
            .biometric()
            .status()
            .map_err(|e| Error::Internal(format!("Biometric status check failed: {}", e)))?;

        if !status.is_available || matches!(status.biometry_type, BiometryType::None) {
            let reason = status
                .error
                .unwrap_or_else(|| "Biometric authentication is unavailable.".to_string());
            return Err(Error::Internal(reason));
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let availability = UserConsentVerifier::CheckAvailabilityAsync()
            .map_err(|e| Error::Internal(format!("Biometric availability check failed: {e}")))?
            .get()
            .map_err(|e| Error::Internal(format!("Biometric availability check failed: {e}")))?;

        match availability {
            UserConsentVerifierAvailability::Available => Ok(()),
            UserConsentVerifierAvailability::DeviceNotPresent => Err(Error::Internal(
                "No biometric device is present.".to_string(),
            )),
            UserConsentVerifierAvailability::NotConfiguredForUser => Err(Error::Internal(
                "Biometrics are not configured for this user.".to_string(),
            )),
            UserConsentVerifierAvailability::DisabledByPolicy => Err(Error::Internal(
                "Biometrics are disabled by policy.".to_string(),
            )),
            _ => Err(Error::Internal(
                "Biometric authentication is unavailable.".to_string(),
            )),
        }
    }

    #[cfg(target_os = "macos")]
    {
        unsafe {
            let context: *mut Object = msg_send![class!(LAContext), new];
            let mut error: *mut Object = std::ptr::null_mut();
            let policy: i64 = 2; // LAPolicyDeviceOwnerAuthentication (biometrics or passcode)
            let can: bool = msg_send![context, canEvaluatePolicy:policy error:&mut error];
            if !can {
                return Err(Error::Internal(
                    "Biometric authentication is unavailable.".to_string(),
                ));
            }
            Ok(())
        }
    }

    #[cfg(not(any(mobile, target_os = "windows", target_os = "macos")))]
    {
        Err(Error::Internal(
            "Biometric authentication is not supported on this platform.".to_string(),
        ))
    }
}

pub fn authenticate_biometric(app: &AppHandle, reason: &str) -> Result<()> {
    ensure_biometric_available(app)?;

    #[cfg(mobile)]
    {
        let options = AuthOptions {
            allow_device_credential: true,
            title: Some("Unlock Pulsar".to_string()),
            subtitle: Some("Confirm your identity to access the vault".to_string()),
            confirmation_required: Some(true),
            ..Default::default()
        };

        app.biometric()
            .authenticate(reason.to_string(), options)
            .map_err(|e| Error::Internal(format!("Biometric authentication failed: {}", e)))
    }

    #[cfg(target_os = "windows")]
    {
        let reason = HSTRING::from(reason);
        let result = UserConsentVerifier::RequestVerificationAsync(&reason)
            .map_err(|e| Error::Internal(format!("Biometric authentication failed: {e}")))?
            .get()
            .map_err(|e| Error::Internal(format!("Biometric authentication failed: {e}")))?;

        match result {
            UserConsentVerificationResult::Verified => Ok(()),
            UserConsentVerificationResult::DeviceBusy => Err(Error::Internal(
                "Biometric device is busy.".to_string(),
            )),
            UserConsentVerificationResult::DeviceNotPresent => Err(Error::Internal(
                "No biometric device is present.".to_string(),
            )),
            UserConsentVerificationResult::DisabledByPolicy => Err(Error::Internal(
                "Biometrics are disabled by policy.".to_string(),
            )),
            UserConsentVerificationResult::NotConfiguredForUser => Err(Error::Internal(
                "Biometrics are not configured for this user.".to_string(),
            )),
            UserConsentVerificationResult::Canceled => Err(Error::Internal(
                "Biometric verification was canceled.".to_string(),
            )),
            _ => Err(Error::Internal(
                "Biometric verification failed.".to_string(),
            )),
        }
    }

    #[cfg(target_os = "macos")]
    {
        unsafe {
            let context: *mut Object = msg_send![class!(LAContext), new];
            let policy: i64 = 2; // LAPolicyDeviceOwnerAuthentication (biometrics or passcode)
            let reason_ns = NSString::from_str(reason);
            let semaphore = Arc::new(Semaphore::new(0));
            let success = Arc::new(AtomicBool::new(false));
            let success_clone = success.clone();
            let semaphore_clone = semaphore.clone();

            let handler = ConcreteBlock::new(move |ok: bool, _err: *mut Object| {
                success_clone.store(ok, Ordering::SeqCst);
                semaphore_clone.signal();
            })
            .copy();

            let _: () = msg_send![context, evaluatePolicy:policy localizedReason:reason_ns reply:handler];
            semaphore.wait();
            if success.load(Ordering::SeqCst) {
                Ok(())
            } else {
                Err(Error::Internal("Biometric verification failed.".to_string()))
            }
        }
    }

    #[cfg(not(any(mobile, target_os = "windows", target_os = "macos")))]
    {
        let _ = reason;
        Err(Error::Internal(
            "Biometric authentication is not supported on this platform.".to_string(),
        ))
    }
}

pub async fn is_biometrics_enabled_impl(app: &AppHandle, state: &State<'_, AppState>) -> Result<bool> {
    if ensure_biometric_available(app).is_err() {
        return Ok(false);
    }
    let db_path = crate::auth::get_db_path(state).await?;
    let vault_user = get_vault_id(&db_path);
    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(Error::Internal(e.to_string())),
    }
}

pub async fn get_biometric_master_password(app: &AppHandle, state: &State<'_, AppState>) -> Result<String> {
    authenticate_biometric(app, "Unlock your Pulsar vault")?;
    let db_path = crate::auth::get_db_path(state).await?;
    let vault_user = get_vault_id(&db_path);

    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    let bio_key_b64 = entry.get_password().map_err(|e| {
        if matches!(e, keyring::Error::NoEntry) {
            Error::Internal("Biometrics not configured for this vault".to_string())
        } else {
            Error::Internal(e.to_string())
        }
    })?;

    let mut bio_key_vec = general_purpose::STANDARD
        .decode(&bio_key_b64)
        .map_err(|_| Error::Internal("Invalid biometric key format".to_string()))?;
    if bio_key_vec.len() != 32 {
        bio_key_vec.zeroize();
        return Err(Error::Internal("Invalid biometric key length".to_string()));
    }
    let mut bio_key_bytes = [0u8; 32];
    bio_key_bytes.copy_from_slice(&bio_key_vec);
    bio_key_vec.zeroize();

    let db_pool = state.db.lock().await.clone().ok_or(Error::VaultNotLoaded)?;
    let row: Option<String> = sqlx::query_scalar("SELECT value FROM configuration WHERE key = 'biometric_encrypted_password'")
        .fetch_optional(&db_pool)
        .await?;

    let encrypted_password_blob = row.ok_or_else(|| Error::Internal("Biometric configuration corrupted (DB entry missing)".to_string()))?;

    let master_password = decrypt(&encrypted_password_blob, &bio_key_bytes)
        .map_err(|_| Error::Internal("Biometric decryption failed".to_string()))?;
    bio_key_bytes.zeroize();

    Ok(master_password)
}

pub async fn enable_biometrics_impl(
    app: &AppHandle,
    state: &State<'_, AppState>,
    password: &str,
) -> Result<()> {
    ensure_biometric_available(app)?;
    let db_path = crate::auth::get_db_path(state).await?;
    
    let mut bio_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bio_key_bytes);
    let bio_key_b64 = Zeroizing::new(general_purpose::STANDARD.encode(bio_key_bytes));

    let encrypted_password_blob = encrypt(password, &bio_key_bytes)?;
    let db_pool = state.db.lock().await.clone().ok_or(Error::VaultNotLoaded)?;
    sqlx::query(
        "INSERT OR REPLACE INTO configuration (key, value) VALUES ('biometric_encrypted_password', ?)",
    )
    .bind(encrypted_password_blob)
    .execute(&db_pool)
    .await?;

    let vault_user = get_vault_id(&db_path);
    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    entry.set_password(bio_key_b64.as_str()).map_err(|e| Error::Internal(e.to_string()))?;
    bio_key_bytes.zeroize();

    Ok(())
}

pub async fn disable_biometrics_impl(state: &State<'_, AppState>) -> Result<()> {
    let db_path = crate::auth::get_db_path(state).await?;
    let vault_user = get_vault_id(&db_path);

    let entry = Entry::new(KEYRING_SERVICE, &vault_user).map_err(|e| Error::Internal(e.to_string()))?;
    let _ = entry.delete_credential(); 

    if let Some(db_pool) = state.db.lock().await.as_ref() {
        let _ = sqlx::query("DELETE FROM configuration WHERE key = 'biometric_encrypted_password'")
            .execute(db_pool)
            .await;
    }
    
    Ok(())
}
