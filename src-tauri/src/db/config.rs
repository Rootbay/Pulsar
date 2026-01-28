use crate::db::utils::{get_db_pool, get_key};
use crate::encryption::{decrypt, encrypt};
use crate::error::Result;
use crate::state::AppState;
use sqlx::Row;
use tauri::State;

#[tauri::command]
pub async fn wipe_vault_database(state: State<'_, AppState>) -> Result<()> {
    get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let mut tx = db_pool.begin().await?;

    sqlx::query("DELETE FROM password_items")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM buttons").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM recipient_keys")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM attachments")
        .execute(&mut *tx)
        .await?;

    if let Err(e) = sqlx::query("DELETE FROM sqlite_sequence WHERE name IN ('password_items', 'buttons', 'recipient_keys', 'attachments')").execute(&mut *tx).await {
         let _ = e;
    }

    tx.commit().await?;
    Ok(())
}

#[tauri::command]
pub async fn save_profile_settings(
    state: State<'_, AppState>,
    settings_json: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let encrypted = encrypt(&settings_json, key.as_slice())?;

    sqlx::query("INSERT OR REPLACE INTO configuration (key, value) VALUES ('profile_settings', ?)")
        .bind(encrypted)
        .execute(&db_pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_profile_settings(state: State<'_, AppState>) -> Result<Option<String>> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let row = sqlx::query("SELECT value FROM configuration WHERE key = 'profile_settings'")
        .fetch_optional(&db_pool)
        .await?;

    if let Some(row) = row {
        let encrypted: String = row.get("value");
        let decrypted = decrypt(&encrypted, key.as_slice())?;
        Ok(Some(decrypted))
    } else {
        Ok(None)
    }
}
