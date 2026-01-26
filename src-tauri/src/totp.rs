use totp_rs::{Algorithm, Secret, TOTP};
use rand::Rng;
use base32::{encode, Alphabet};
use crate::error::{Error, Result};
use tauri::State;
use crate::state::AppState;
use crate::db_commands;

fn build_totp(secret_b32: &str) -> Result<TOTP> {
    let secret = Secret::Encoded(secret_b32.to_string());
    let secret_bytes = secret.to_bytes().map_err(|e| Error::Totp(e.to_string()))?;

    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Pulsar".to_string()),
        "user".to_string(),
    )
    .map_err(|e| Error::Totp(e.to_string()))
}

#[tauri::command]
pub fn generate_totp_secret() -> Result<String> {
    let mut secret_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut secret_bytes);

    let secret_b32 = encode(Alphabet::Rfc4648 { padding: false }, &secret_bytes);

    Ok(secret_b32)
}

#[tauri::command]
pub fn generate_totp(secret_b32: String) -> Result<String> {
    let totp = build_totp(&secret_b32)?;
    totp.generate_current().map_err(|e| Error::Totp(e.to_string()))
}

#[tauri::command]
pub async fn verify_totp(
    state: State<'_, AppState>,
    id: i64,
    token: String,
) -> Result<bool> {
    let password_item_option = db_commands::get_password_item_by_id(state, id).await?;

    if let Some(password_item) = password_item_option {
        if let Some(secret_b32) = password_item.totp_secret {
            let totp = build_totp(&secret_b32)?;
            Ok(totp.check_current(&token).map_err(|e| Error::Totp(e.to_string()))?)
        } else {
            Err(Error::Internal("TOTP secret not found for this item.".to_string()))
        }
    } else {
        Err(Error::Internal("Password item not found.".to_string()))
    }
}

#[tauri::command]
pub fn verify_totp_secret(secret_b32: String, token: String) -> Result<bool> {
    let totp = build_totp(&secret_b32)?;
    Ok(totp.check_current(&token).map_err(|e| Error::Totp(e.to_string()))?)
}
