use crate::encryption::{encrypt, encrypt_bytes, decrypt_bytes};
use crate::state::AppState;
use crate::types::Attachment;
use crate::error::{Error, Result};
use crate::db::utils::{get_key, get_db_pool};
use tauri::State;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;

async fn get_attachments_dir(state: &AppState) -> Result<PathBuf> {
    let db_path = state.db_path.lock().await.clone()
        .ok_or_else(|| Error::Internal("Database path not set".to_string()))?;
    
    let mut dir = db_path.clone();
    let file_name = dir.file_name()
        .ok_or_else(|| Error::Internal("Invalid DB path".to_string()))?
        .to_string_lossy();
    
    dir.set_file_name(format!("{}.attachments", file_name));
    
    if !fs::try_exists(&dir).await.unwrap_or(false) {
        fs::create_dir_all(&dir).await?;
    }
    
    Ok(dir)
}

#[tauri::command]
pub async fn add_attachment(
    state: State<'_, AppState>,
    item_id: i64,
    file_path: String,
) -> Result<Attachment> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let attachments_dir = get_attachments_dir(&state).await?;

    let path = Path::new(&file_path);
    if !fs::try_exists(path).await.unwrap_or(false) {
        return Err(Error::Internal("File not found".to_string()));
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Internal("Invalid file name".to_string()))?
        .to_string();
    
    let file_data = fs::read(path).await?;
    let file_size = file_data.len() as i64;
    
    let mime_type = mime_guess::from_path(path).first_or_octet_stream().to_string();

    let encrypted_data = encrypt_bytes(&file_data, key.as_slice())?;
    
    let name_enc = encrypt(&file_name, key.as_slice())?;
    let mime_enc = encrypt(&mime_type, key.as_slice())?;
    let now = Utc::now().to_rfc3339();

    let id = sqlx::query("INSERT INTO attachments (item_id, file_name, file_size, mime_type, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(item_id)
        .bind(name_enc)
        .bind(file_size)
        .bind(mime_enc)
        .bind(&now)
        .execute(&db_pool)
        .await?
        .last_insert_rowid();

    let storage_path = attachments_dir.join(id.to_string());
    fs::write(storage_path, encrypted_data).await?;

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
    let attachments_dir = get_attachments_dir(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    
    sqlx::query("DELETE FROM attachments WHERE id = ?")
        .bind(id)
        .execute(&db_pool)
        .await?;

    let storage_path = attachments_dir.join(id.to_string());
    if fs::try_exists(&storage_path).await.unwrap_or(false) {
        let _ = fs::remove_file(storage_path).await;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn save_attachment_to_disk(
    state: State<'_, AppState>,
    attachment_id: i64,
    save_path: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let attachments_dir = get_attachments_dir(&state).await?;

    let storage_path = attachments_dir.join(attachment_id.to_string());
    if !fs::try_exists(&storage_path).await.unwrap_or(false) {
        return Err(Error::Internal("Attachment file not found on disk".to_string()));
    }

    let data_blob = fs::read(storage_path).await?;
    let file_data = decrypt_bytes(&data_blob, key.as_slice())?;

    write_sensitive_bytes(Path::new(&save_path), &file_data).await?;
    Ok(())
}

#[tauri::command]
pub async fn import_file_as_attachment(
    state: State<'_, AppState>,
    item_id: i64,
    file_path: PathBuf,
) -> Result<Attachment> {
    add_attachment(state, item_id, file_path.to_string_lossy().to_string()).await
}

#[tauri::command]
pub async fn export_attachment_to_file(
    state: State<'_, AppState>,
    attachment_id: i64,
    save_path: PathBuf,
) -> Result<()> {
    save_attachment_to_disk(state, attachment_id, save_path.to_string_lossy().to_string()).await
}

async fn write_sensitive_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp_path = path.with_extension("tmp");
    if fs::try_exists(&tmp_path).await.unwrap_or(false) {
        let _ = fs::remove_file(&tmp_path).await;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
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
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

        if let Err(err) = fs::rename(&tmp_path, path).await {

            if err.kind() == std::io::ErrorKind::Other || err.raw_os_error() == Some(17) || err.raw_os_error() == Some(18) {

                fs::copy(&tmp_path, path).await?;

                let _ = fs::remove_file(&tmp_path).await;

            } else {

                return Err(Error::Io(err));

            }

        }

        Ok(())

    }

    