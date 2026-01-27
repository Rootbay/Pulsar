use crate::state::AppState;
use crate::types::{RecipientKey, SecretString};
use crate::encryption::{encrypt, decrypt};
use crate::error::Result;
use crate::db::utils::{get_key, get_db_pool};
use tauri::State;
use sqlx::{Row, SqlitePool};

#[tauri::command]
pub async fn save_recipient_key(
    state: State<'_, AppState>,
    name: String,
    public_key: String,
    private_key: SecretString,
) -> Result<()> {
    let key = get_key(&state).await?;
    let name_enc = encrypt(&name, key.as_slice())?;
    let public_key_enc = encrypt(&public_key, key.as_slice())?;
    let private_key_enc = encrypt(private_key.as_str(), key.as_slice())?;

    let db_pool = get_db_pool(&state).await?;
    sqlx::query("INSERT INTO recipient_keys (name, public_key, private_key) VALUES (?, ?, ?)")
        .bind(name_enc)
        .bind(public_key_enc)
        .bind(private_key_enc)
        .execute(&db_pool)
        .await?;
    Ok(())
}

pub async fn get_recipient_keys_impl(db_pool: &SqlitePool, key: &[u8]) -> Result<Vec<RecipientKey>> {
    let rows = sqlx::query("SELECT id, name, public_key, private_key FROM recipient_keys")
        .fetch_all(db_pool)
        .await?;

    let mut keys = Vec::new();
    for row in rows {
        let name_enc: String = row.get("name");
        let public_key_enc: String = row.get("public_key");
        let private_key_enc: String = row.get("private_key");

        keys.push(RecipientKey {
            id: row.get("id"),
            name: decrypt(name_enc.as_str(), key)?,
            public_key: decrypt(public_key_enc.as_str(), key)?,
            private_key: SecretString::new(decrypt(private_key_enc.as_str(), key)?),
        });
    }
    Ok(keys)
}


#[tauri::command]
pub async fn get_recipient_keys(state: State<'_, AppState>) -> Result<Vec<RecipientKey>> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    get_recipient_keys_impl(&db_pool, key.as_slice()).await
}

#[tauri::command]
pub async fn delete_recipient_key(state: State<'_, AppState>, id: i64) -> Result<()> {
    get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("DELETE FROM recipient_keys WHERE id = ?")
        .bind(id)
        .execute(&db_pool)
        .await?;
    Ok(())
}
