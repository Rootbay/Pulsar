use crate::state::AppState;
use crate::types::{Button, PasswordItem, RecipientKey, CustomField, Attachment};
use crate::encryption::{encrypt, decrypt, encrypt_bytes, decrypt_bytes};
use crate::error::{Error, Result};
use tauri::State;
use sqlx::Row;
use sqlx::SqlitePool;
use chrono::Utc;
use zeroize::Zeroizing;
use validator::Validate;
use serde_json;
use std::fs;
use std::path::Path;

pub fn validate_password_item_fields(item: &PasswordItem) -> std::result::Result<(), validator::ValidationError> {
    if item.title.is_empty() {
        return Err(validator::ValidationError::new("title_empty"));
    }
    if item.title.len() > 255 {
        return Err(validator::ValidationError::new("title_too_long"));
    }

    if let Some(username) = &item.username {
        if username.is_empty() {
            return Err(validator::ValidationError::new("username_empty"));
        }
    }

    let is_placeholder_password = item.password.trim().is_empty() || item.password == "N/A";
    if !is_placeholder_password {
        if item.password.len() < 8 {
            return Err(validator::ValidationError::new("password_too_short"));
        }
    }

    if let Some(url) = &item.url {
        if !url.is_empty() && !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(validator::ValidationError::new("invalid_url_format"));
        }
    }

    if let Some(totp_secret) = &item.totp_secret {
        if !totp_secret.is_empty() && totp_secret.len() < 16 {
            return Err(validator::ValidationError::new("totp_secret_too_short"));
        }
    }

    Ok(())
}


async fn get_key(state: &State<'_, AppState>) -> Result<Zeroizing<Vec<u8>>> {
    let guard = state.key.lock().await;
    let opt = guard.clone();
    drop(guard);
    opt.ok_or(Error::VaultLocked)
}

async fn get_db_pool(state: &State<'_, AppState>) -> Result<SqlitePool> {
    let guard = state.db.lock().await;
    guard.clone().ok_or(Error::VaultNotLoaded)
}

#[tauri::command]
pub async fn save_button(
    state: State<'_, AppState>,
    text: String,
    icon: String,
    color: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let text_enc = encrypt(&text, key.as_slice())?;
    let icon_enc = encrypt(&icon, key.as_slice())?;
    let color_enc = encrypt(&color, key.as_slice())?;

    sqlx::query("INSERT INTO buttons (text, icon, color) VALUES (?, ?, ?)")
        .bind(text_enc)
        .bind(icon_enc)
        .bind(color_enc)
        .execute(&db_pool)
        .await?;
    Ok(())
}

pub async fn get_buttons_impl(db_pool: &SqlitePool, key: &[u8]) -> Result<Vec<Button>> {
    let rows = sqlx::query("SELECT id, text, icon, color FROM buttons")
        .fetch_all(db_pool)
        .await?;

    let mut buttons = Vec::new();
    for row in rows {
        let text_enc: String = row.get("text");
        let icon_enc: String = row.get("icon");
        let color_enc: String = row.get("color");

        buttons.push(Button {
            id: row.get("id"),
            text: decrypt(&text_enc, key)?,
            icon: decrypt(&icon_enc, key)?,
            color: decrypt(&color_enc, key)?,
        });
    }

    Ok(buttons)
}

#[tauri::command]
pub async fn get_buttons(state: State<'_, AppState>) -> Result<Vec<Button>> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    get_buttons_impl(&db_pool, key.as_slice()).await
}

#[tauri::command]
pub async fn update_button(
    state: State<'_, AppState>,
    id: i64,
    text: String,
    icon: String,
    color: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let text_enc = encrypt(&text, key.as_slice())?;
    let icon_enc = encrypt(&icon, key.as_slice())?;
    let color_enc = encrypt(&color, key.as_slice())?;

    sqlx::query("UPDATE buttons SET text = ?, icon = ?, color = ? WHERE id = ?")
        .bind(text_enc)
        .bind(icon_enc)
        .bind(color_enc)
        .bind(id)
        .execute(&db_pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_button(state: State<'_, AppState>, id: i64) -> Result<()> {
    get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("DELETE FROM buttons WHERE id = ?")
        .bind(id)
        .execute(&db_pool)
        .await?;
    Ok(())
}


#[tauri::command]
pub async fn save_password_item(
    state: State<'_, AppState>,
    item: PasswordItem,
) -> Result<()> {
    item.validate().map_err(|e| Error::Validation(e.to_string()))?;

    let key = get_key(&state).await?;
    let now = Utc::now().to_rfc3339();

    let category_enc = encrypt(&item.category, key.as_slice())?;
    let title_enc = encrypt(&item.title, key.as_slice())?;
    let description_enc = item.description.map(|d| encrypt(&d, key.as_slice())).transpose()?;
    let img_enc = item.img.map(|i| encrypt(&i, key.as_slice())).transpose()?;
    let tags_enc = item.tags.map(|t| encrypt(&t, key.as_slice())).transpose()?;
    let username_enc = item.username.map(|u| encrypt(&u, key.as_slice())).transpose()?;
    let url_enc = item.url.map(|u| encrypt(&u, key.as_slice())).transpose()?;
    let notes_enc = item.notes.map(|n| encrypt(&n, key.as_slice())).transpose()?;
    let password_enc = encrypt(&item.password, key.as_slice())?;
    let totp_secret_enc = item.totp_secret.map(|t| encrypt(&t, key.as_slice())).transpose()?;
    let custom_fields_json = serde_json::to_string(&item.custom_fields)?;
    let custom_fields_enc = encrypt(&custom_fields_json, key.as_slice())?;
    let field_order_json = item.field_order.map(|fo| serde_json::to_string(&fo)).transpose()?;
    let field_order_enc = field_order_json.map(|fo_json| encrypt(&fo_json, key.as_slice())).transpose()?;

    let db_pool = get_db_pool(&state).await?;
    sqlx::query("INSERT INTO password_items (category, title, description, img, tags, username, url, notes, password, created_at, updated_at, color, totp_secret, custom_fields, field_order) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(category_enc)
        .bind(title_enc)
        .bind(description_enc)
        .bind(img_enc)
        .bind(tags_enc)
        .bind(username_enc)
        .bind(url_enc)
        .bind(notes_enc)
        .bind(password_enc)
        .bind(now.clone())
        .bind(now)
        .bind(item.color)
        .bind(totp_secret_enc)
        .bind(custom_fields_enc)
        .bind(field_order_enc)
        .execute(&db_pool)
        .await?;
    Ok(())
}

async fn fetch_attachments_for_item(pool: &SqlitePool, key: &[u8], item_id: i64) -> Result<Vec<Attachment>> {
    let rows = sqlx::query("SELECT id, file_name, file_size, mime_type, created_at FROM attachments WHERE item_id = ?")
        .bind(item_id)
        .fetch_all(pool)
        .await?;

    let mut attachments = Vec::new();
    for row in rows {
        let name_enc: String = row.get("file_name");
        let mime_enc: String = row.get("mime_type");
        
        attachments.push(Attachment {
            id: row.get("id"),
            item_id,
            file_name: decrypt(&name_enc, key)?,
            file_size: row.get("file_size"),
            mime_type: decrypt(&mime_enc, key)?,
            created_at: row.get("created_at"),
        });
    }
    Ok(attachments)
}

pub async fn get_password_items_impl(db_pool: &SqlitePool, key: &[u8]) -> Result<Vec<PasswordItem>> {
    let rows = sqlx::query("SELECT id, category, title, description, img, tags, username, url, notes, password, created_at, updated_at, color, totp_secret, custom_fields, field_order FROM password_items")
        .fetch_all(db_pool)
        .await?;

    let mut items = Vec::new();
    for row in rows {
        let category_enc: String = row.get("category");
        let category = decrypt(&category_enc, key).unwrap_or_else(|_| "login".to_string());

        let title_enc: String = row.get("title");
        let title = decrypt(&title_enc, key)?;

        let description_enc: Option<String> = row.get("description");
        let description = description_enc.map(|d| decrypt(d.as_str(), key)).transpose()?;

        let img_enc: Option<String> = row.get("img");
        let img = img_enc.map(|i| decrypt(i.as_str(), key)).transpose()?;

        let tags_enc: Option<String> = row.get("tags");
        let tags = tags_enc.map(|t| decrypt(t.as_str(), key)).transpose()?;

        let username_enc: Option<String> = row.get("username");
        let username = username_enc.map(|u| decrypt(u.as_str(), key)).transpose()?;

        let url_enc: Option<String> = row.get("url");
        let url = url_enc.map(|u| decrypt(u.as_str(), key)).transpose()?;

        let notes_enc: Option<String> = row.get("notes");
        let notes = notes_enc.map(|n| decrypt(n.as_str(), key)).transpose()?;

        let password_enc: String = row.get("password");
        let password = decrypt(&password_enc, key)?;

        let totp_secret_enc: Option<String> = row.get("totp_secret");
        let totp_secret = totp_secret_enc.map(|t| decrypt(t.as_str(), key)).transpose()?;

        let custom_fields_enc: Option<String> = row.get("custom_fields");
        let custom_fields = custom_fields_enc.map(|cf| decrypt(cf.as_str(), key)).transpose()?.map(|cf| serde_json::from_str(&cf).unwrap_or_default()).unwrap_or_default();

        let field_order_enc: Option<String> = row.get("field_order");
        let field_order = field_order_enc.and_then(|fo_enc| decrypt(fo_enc.as_str(), key).ok()).and_then(|fo_json| serde_json::from_str(&fo_json).ok());

        let attachments = fetch_attachments_for_item(db_pool, key, row.get("id")).await.ok();

        items.push(PasswordItem {
            id: row.get("id"),
            category,
            title,
            description,
            img,
            tags,
            username,
            url,
            notes,
            password,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            color: row.get("color"),
            totp_secret,
            custom_fields,
            field_order,
            attachments,
        });
    }

    Ok(items)
}

#[tauri::command]
pub async fn get_password_items(state: State<'_, AppState>) -> Result<Vec<PasswordItem>> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    get_password_items_impl(&db_pool, key.as_slice()).await
}

#[tauri::command]
pub async fn update_password_item_tags(state: State<'_, AppState>, id: i64, tags: String) -> Result<()> {
    let key = get_key(&state).await?;
    let now = Utc::now().to_rfc3339();
    let tags_enc_opt: Option<String> = if tags.trim().is_empty() {
        None
    } else {
        Some(encrypt(&tags, key.as_slice())?)
    };
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("UPDATE password_items SET tags = ?, updated_at = ? WHERE id = ?")
        .bind(tags_enc_opt)
        .bind(now)
        .bind(id)
        .execute(&db_pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn update_password_item_totp_secret(
    state: State<'_, AppState>,
    id: i64,
    totp_secret: Option<String>,
) -> Result<()> {
    let key = get_key(&state).await?;
    let now = Utc::now().to_rfc3339();
    let totp_secret_clean = totp_secret.and_then(|secret| {
        let trimmed = secret.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    let totp_secret_enc = match totp_secret_clean {
        Some(secret) => Some(encrypt(&secret, key.as_slice())?),
        None => None,
    };
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("UPDATE password_items SET totp_secret = ?, updated_at = ? WHERE id = ?")
        .bind(totp_secret_enc)
        .bind(now)
        .bind(id)
        .execute(&db_pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn update_password_item(
    state: State<'_, AppState>,
    item: PasswordItem,
) -> Result<()> {
    item.validate().map_err(|e| Error::Validation(e.to_string()))?;

    let key = get_key(&state).await?;
    let now = Utc::now().to_rfc3339();

    let category_enc = encrypt(&item.category, key.as_slice())?;
    let title_enc = encrypt(&item.title, key.as_slice())?;
    let description_enc = item.description.map(|d| encrypt(&d, &key)).transpose()?;
    let img_enc = item.img.map(|i| encrypt(&i, &key)).transpose()?;
    let tags_enc = item.tags.map(|t| encrypt(&t, &key)).transpose()?;
    let username_enc = item.username.map(|u| encrypt(&u, &key)).transpose()?;
    let url_enc = item.url.map(|u| encrypt(&u, &key)).transpose()?;
    let notes_enc = item.notes.map(|n| encrypt(&n, &key)).transpose()?;
    let password_enc = encrypt(&item.password, key.as_slice())?;
    let totp_secret_enc = item.totp_secret.map(|t| encrypt(&t, key.as_slice())).transpose()?;
    let custom_fields_json = serde_json::to_string(&item.custom_fields)?;
    let custom_fields_enc = encrypt(&custom_fields_json, key.as_slice())?;

    let field_order_json = item.field_order.map(|fo| serde_json::to_string(&fo)).transpose()?;
    let field_order_enc = field_order_json.map(|fo_json| encrypt(&fo_json, key.as_slice())).transpose()?;


    let db_pool = get_db_pool(&state).await?;
    sqlx::query("UPDATE password_items SET category = ?, title = ?, description = ?, img = ?, tags = ?, username = ?, url = ?, notes = ?, password = ?, updated_at = ?, color = ?, totp_secret = ?, custom_fields = ?, field_order = ? WHERE id = ?")
        .bind(category_enc)
        .bind(title_enc)
        .bind(description_enc)
        .bind(img_enc)
        .bind(tags_enc)
        .bind(username_enc)
        .bind(url_enc)
        .bind(notes_enc)
        .bind(password_enc)
        .bind(now)
        .bind(item.color)
        .bind(totp_secret_enc)
        .bind(custom_fields_enc)
        .bind(field_order_enc)
        .bind(item.id)
        .execute(&db_pool)
        .await?;
    Ok(())
}


#[tauri::command]
pub async fn add_custom_field(
    state: State<'_, AppState>,
    item_id: i64,
    field_name: String,
    field_type: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let row = sqlx::query("SELECT custom_fields FROM password_items WHERE id = ?")
        .bind(item_id)
        .fetch_one(&db_pool)
        .await?;

    let custom_fields_enc: Option<String> = row.get("custom_fields");
    let custom_fields_json = custom_fields_enc
        .map(|cf| decrypt(cf.as_str(), key.as_slice()))
        .transpose()?
        .unwrap_or_else(|| "[]".to_string());

    let mut custom_fields: Vec<CustomField> = serde_json::from_str(&custom_fields_json)?;

    custom_fields.push(CustomField {
        name: field_name,
        value: "".to_string(),
        field_type,
    });

    let updated_custom_fields_json = serde_json::to_string(&custom_fields)?;
    let updated_custom_fields_enc = encrypt(&updated_custom_fields_json, key.as_slice())?;

    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE password_items SET custom_fields = ?, updated_at = ? WHERE id = ?")
        .bind(updated_custom_fields_enc)
        .bind(now)
        .bind(item_id)
        .execute(&db_pool)
        .await?;

    Ok(())
}


#[tauri::command]
pub async fn wipe_vault_database(state: State<'_, AppState>) -> Result<()> {
    get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let mut tx = db_pool.begin().await?;

    sqlx::query("DELETE FROM password_items").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM buttons").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM recipient_keys").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM attachments").execute(&mut *tx).await?;
    
    // We don't wipe 'configuration' fully because we might want to keep some basic setup,
    // but the destructive UI usually means "everything". 
    // However, if we wipe the master password salt, the user can't log in again.
    // Let's just wipe data tables for now as a "Clear All Data" action.
    
    if let Err(e) = sqlx::query("DELETE FROM sqlite_sequence WHERE name IN ('password_items', 'buttons', 'recipient_keys', 'attachments')").execute(&mut *tx).await {
         // Ignore error if table doesn't exist
         let _ = e;
    }

    tx.commit().await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_password_item(state: State<'_, AppState>, id: i64) -> Result<()> {
    get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("DELETE FROM password_items WHERE id = ?")
        .bind(id)
        .execute(&db_pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_password_item_by_id(state: State<'_, AppState>, id: i64) -> Result<Option<PasswordItem>> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let row = sqlx::query("SELECT id, category, title, description, img, tags, username, url, notes, password, created_at, updated_at, color, totp_secret, custom_fields, field_order FROM password_items WHERE id = ?")
        .bind(id)
        .fetch_optional(&db_pool)
        .await?;

    if let Some(row) = row {
        let category_enc: String = row.get("category");
        let category = decrypt(&category_enc, key.as_slice()).unwrap_or_else(|_| "login".to_string());

        let title_enc: String = row.get("title");
        let title = decrypt(&title_enc, key.as_slice())?;

        let description_enc: Option<String> = row.get("description");
        let description = description_enc.map(|d| decrypt(d.as_str(), key.as_slice())).transpose()?;

        let img_enc: Option<String> = row.get("img");
        let img = img_enc.map(|i| decrypt(i.as_str(), key.as_slice())).transpose()?;

        let tags_enc: Option<String> = row.get("tags");
        let tags = tags_enc.map(|t| decrypt(t.as_str(), key.as_slice())).transpose()?;

        let username_enc: Option<String> = row.get("username");
        let username = username_enc.map(|u| decrypt(u.as_str(), key.as_slice())).transpose()?;

        let url_enc: Option<String> = row.get("url");
        let url = url_enc.map(|u| decrypt(u.as_str(), key.as_slice())).transpose()?;

        let notes_enc: Option<String> = row.get("notes");
        let notes = notes_enc.map(|n| decrypt(n.as_str(), key.as_slice())).transpose()?;

        let password_enc: String = row.get("password");
        let password = decrypt(&password_enc, key.as_slice())?;

        let totp_secret_enc: Option<String> = row.get("totp_secret");
        let totp_secret = totp_secret_enc.map(|t| decrypt(t.as_str(), key.as_slice())).transpose()?;

        let custom_fields_enc: Option<String> = row.get("custom_fields");
        let custom_fields = custom_fields_enc.map(|cf| decrypt(cf.as_str(), key.as_slice())).transpose()?.map(|cf| serde_json::from_str(&cf).unwrap_or_default()).unwrap_or_default();

        let field_order_enc: Option<String> = row.get("field_order");
        let field_order = field_order_enc.and_then(|fo_enc| decrypt(fo_enc.as_str(), key.as_slice()).ok()).and_then(|fo_json| serde_json::from_str(&fo_json).ok());

        let attachments = fetch_attachments_for_item(&db_pool, key.as_slice(), row.get("id")).await.ok();

        Ok(Some(PasswordItem {
            id: row.get("id"),
            category,
            title,
            description,
            img,
            tags,
            username,
            url,
            notes,
            password,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            color: row.get("color"),
            totp_secret,
            custom_fields,
            field_order,
            attachments,
        }))
    } else {
        Ok(None)
    }
}


#[tauri::command]
pub async fn save_recipient_key(
    state: State<'_, AppState>,
    name: String,
    public_key: String,
    private_key: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let name_enc = encrypt(&name, key.as_slice())?;
    let public_key_enc = encrypt(&public_key, key.as_slice())?;
    let private_key_enc = encrypt(&private_key, key.as_slice())?;

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
        private_key: decrypt(private_key_enc.as_str(), key)?,
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

#[tauri::command]
pub async fn add_attachment(
    state: State<'_, AppState>,
    item_id: i64,
    file_path: String,
) -> Result<Attachment> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(Error::Internal("File not found".to_string()));
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Internal("Invalid file name".to_string()))?
        .to_string();
    
    let file_data = fs::read(path)?;
    let file_size = file_data.len() as i64;
    
    // Simple mime inference (optional, can be improved or passed from frontend)
    let mime_type = mime_guess::from_path(path).first_or_octet_stream().to_string();

    let encrypted_data = encrypt_bytes(&file_data, key.as_slice())?;
    
    // Encrypt metadata
    let name_enc = encrypt(&file_name, key.as_slice())?;
    let mime_enc = encrypt(&mime_type, key.as_slice())?;
    let now = Utc::now().to_rfc3339();

    let id = sqlx::query("INSERT INTO attachments (item_id, file_name, file_size, mime_type, data, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(item_id)
        .bind(name_enc)
        .bind(file_size)
        .bind(mime_enc)
        .bind(encrypted_data) // Store raw encrypted bytes as BLOB
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

    fs::write(&save_path, file_data)?;
    Ok(())
}

#[tauri::command]
pub async fn save_profile_settings(state: State<'_, AppState>, settings_json: String) -> Result<()> {
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

