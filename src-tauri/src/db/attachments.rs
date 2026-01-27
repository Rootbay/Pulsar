use crate::state::AppState;
use crate::types::Attachment;
use crate::encryption::{encrypt, encrypt_bytes, decrypt_bytes};
use crate::error::{Error, Result};
use crate::db::utils::{get_key, get_db_pool};
use tauri::State;
use sqlx::Row;
use chrono::Utc;
use std::path::Path;

#[tauri::command]
pub async fn add_attachment(
    state: State<'_, AppState>,
    item_id: i64,
    file_path: String,
) -> Result<Attachment> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let path = Path::new(&file_path);
    if !tokio::fs::try_exists(path).await.unwrap_or(false) {
        return Err(Error::Internal("File not found".to_string()));
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Internal("Invalid file name".to_string()))?
        .to_string();
    
    let file_data = tokio::fs::read(path).await?;
    let file_size = file_data.len() as i64;
    
    let mime_type = mime_guess::from_path(path).first_or_octet_stream().to_string();

    let encrypted_data = encrypt_bytes(&file_data, key.as_slice())?;
    
    let name_enc = encrypt(&file_name, key.as_slice())?;
    let mime_enc = encrypt(&mime_type, key.as_slice())?;
    let now = Utc::now().to_rfc3339();

    let id = sqlx::query("INSERT INTO attachments (item_id, file_name, file_size, mime_type, data, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(item_id)
        .bind(name_enc)
        .bind(file_size)
        .bind(mime_enc)
        .bind(encrypted_data)
        .bind(&now)
        .execute(&db_pool)
        .await?
        .last_insert_rowid();

    Ok(Attachment {
        id,
        item_id,
        file_name,
        file_size,
        mime_type,
        created_at: now,
    })
}

#[tauri::command]
pub async fn delete_attachment(state: State<'_, AppState>, id: i64) -> Result<()> {
    get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("DELETE FROM attachments WHERE id = ?")
        .bind(id)
        .execute(&db_pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn save_attachment_to_disk(
    state: State<'_, AppState>,
    attachment_id: i64,
    save_path: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let row = sqlx::query("SELECT data FROM attachments WHERE id = ?")
        .bind(attachment_id)
        .fetch_optional(&db_pool)
        .await?
        .ok_or_else(|| Error::Internal("Attachment not found".to_string()))?;

    let data_blob: Vec<u8> = row.get("data");
    let file_data = decrypt_bytes(&data_blob, key.as_slice())?;

    write_sensitive_bytes(Path::new(&save_path), &file_data).await?;
    Ok(())
}

async fn write_sensitive_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp_path = path.with_extension("tmp");
    if tokio::fs::try_exists(&tmp_path).await.unwrap_or(false) {
        let _ = tokio::fs::remove_file(&tmp_path).await;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    #[cfg(not(unix))]
    {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    tokio::fs::rename(&tmp_path, path).await?;
    Ok(())
}
