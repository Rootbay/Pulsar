use totp_rs::{Algorithm, Secret, TOTP};
use rand::Rng;
use base32::{encode, Alphabet};

#[tauri::command]
pub fn generate_totp_secret() -> Result<String, String> {
    let mut secret_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut secret_bytes);

    let secret_b32 = encode(Alphabet::Rfc4648 { padding: false }, &secret_bytes);

    Ok(secret_b32)
}

#[tauri::command]
pub fn generate_totp(secret_b32: String) -> Result<String, String> {
    let secret = Secret::Encoded(secret_b32.clone());
    let secret_bytes = secret.to_bytes().map_err(|e| e.to_string())?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Pulsar".to_string()),
        "user".to_string(),
    ).map_err(|e| e.to_string())?;

    Ok(totp.generate_current().map_err(|e| e.to_string())?)
}

use tauri::State;
use crate::state::AppState;
use crate::db_commands;

#[tauri::command]
pub async fn verify_totp(
    state: State<'_, AppState>,
    id: i64,
    token: String,
) -> Result<bool, String> {
    let password_item_option = db_commands::get_password_item_by_id(state, id).await?;

    if let Some(password_item) = password_item_option {
        if let Some(secret_b32) = password_item.totp_secret {
            let secret = Secret::Encoded(secret_b32.clone());
            let secret_bytes = secret.to_bytes().map_err(|e| e.to_string())?;

            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret_bytes,
                Some("Pulsar".to_string()),
                "user".to_string(),
            ).map_err(|e| e.to_string())?;

            Ok(totp.check_current(&token).unwrap_or(false))
        } else {
            Err("TOTP secret not found for this item.".to_string())
        }
    } else {
        Err("Password item not found.".to_string())
    }
}