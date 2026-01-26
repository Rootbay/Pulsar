use crate::state::AppState;
use crate::types::{PasswordItem, SecretString, CustomField, Attachment};
use crate::encryption::decrypt;
use crate::error::{Error, Result};
use crate::db::utils::{get_key, get_db_pool, CryptoHelper};
use tauri::State;
use sqlx::{Row, SqlitePool};
use chrono::Utc;
use validator::Validate;

async fn fetch_attachments_for_item(pool: &SqlitePool, helper: &CryptoHelper, item_id: i64) -> Result<Vec<Attachment>> {
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
            file_name: helper.decrypt(&name_enc)?,
            file_size: row.get("file_size"),
            mime_type: helper.decrypt(&mime_enc)?,
            created_at: row.get("created_at"),
        });
    }
    Ok(attachments)
}

async fn decrypt_password_item_row(row: &sqlx::sqlite::SqliteRow, helper: &CryptoHelper, db_pool: &SqlitePool) -> Result<PasswordItem> {
    let id: i64 = row.get("id");
    
    let category_enc: String = row.get("category");
    let category = helper.decrypt(&category_enc).unwrap_or_else(|_| "login".to_string());

    let title_enc: String = row.get("title");
    let title = helper.decrypt(&title_enc)?;

    let description = helper.decrypt_opt(row.get("description"))?;
    let img = helper.decrypt_opt(row.get("img"))?;
    let tags = helper.decrypt_opt(row.get("tags"))?;
    let username = helper.decrypt_opt(row.get("username"))?;
    let url = helper.decrypt_opt(row.get("url"))?;
    let notes = helper.decrypt_opt(row.get("notes"))?;

    let password_enc: String = row.get("password");
    let password = SecretString::new(helper.decrypt(&password_enc)?);

    let totp_secret_enc: Option<String> = row.get("totp_secret");
    let totp_secret = totp_secret_enc.map(|t| helper.decrypt(&t)).transpose()?.map(SecretString::new);

    let custom_fields_enc: Option<String> = row.get("custom_fields");
    let custom_fields = custom_fields_enc
        .map(|cf| helper.decrypt(&cf))
        .transpose()?
        .map(|cf| serde_json::from_str(&cf).unwrap_or_default())
        .unwrap_or_default();

    let field_order_enc: Option<String> = row.get("field_order");
    let field_order = field_order_enc
        .and_then(|fo_enc| helper.decrypt(&fo_enc).ok())
        .and_then(|fo_json| serde_json::from_str(&fo_json).ok());

    let attachments = fetch_attachments_for_item(db_pool, helper, id).await.ok();

    Ok(PasswordItem {
        id,
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
    })
}

pub async fn get_password_items_impl(db_pool: &SqlitePool, key: &[u8]) -> Result<Vec<PasswordItem>> {
    let rows = sqlx::query("SELECT id, category, title, description, img, tags, username, url, notes, password, created_at, updated_at, color, totp_secret, custom_fields, field_order FROM password_items")
        .fetch_all(db_pool)
        .await?;

    let helper = CryptoHelper::new(key)?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(decrypt_password_item_row(&row, &helper, db_pool).await?);
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
pub async fn save_password_item(
    state: State<'_, AppState>,
    item: PasswordItem,
) -> Result<()> {
    item.validate().map_err(|e| Error::Validation(e.to_string()))?;

    let key = get_key(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;
    let now = Utc::now().to_rfc3339();

    let category_enc = helper.encrypt(&item.category)?;
    let title_enc = helper.encrypt(&item.title)?;
    let description_enc = helper.encrypt_opt(item.description.as_ref())?;
    let img_enc = helper.encrypt_opt(item.img.as_ref())?;
    let tags_enc = helper.encrypt_opt(item.tags.as_ref())?;
    let username_enc = helper.encrypt_opt(item.username.as_ref())?;
    let url_enc = helper.encrypt_opt(item.url.as_ref())?;
    let notes_enc = helper.encrypt_opt(item.notes.as_ref())?;
    let password_enc = helper.encrypt(item.password.as_str())?;
    let totp_secret_enc = item.totp_secret.as_ref()
        .map(|s| helper.encrypt(s.as_str()))
        .transpose()?;
    
    let custom_fields_json = serde_json::to_string(&item.custom_fields)?;
    let custom_fields_enc = helper.encrypt(&custom_fields_json)?;
    
    let field_order_json = item.field_order.map(|fo| serde_json::to_string(&fo)).transpose()?;
    let field_order_enc = helper.encrypt_opt(field_order_json.as_ref())?;

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

#[tauri::command]
pub async fn update_password_item(
    state: State<'_, AppState>,
    item: PasswordItem,
) -> Result<()> {
    item.validate().map_err(|e| Error::Validation(e.to_string()))?;

    let key = get_key(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;
    let now = Utc::now().to_rfc3339();

    let category_enc = helper.encrypt(&item.category)?;
    let title_enc = helper.encrypt(&item.title)?;
    let description_enc = helper.encrypt_opt(item.description.as_ref())?;
    let img_enc = helper.encrypt_opt(item.img.as_ref())?;
    let tags_enc = helper.encrypt_opt(item.tags.as_ref())?;
    let username_enc = helper.encrypt_opt(item.username.as_ref())?;
    let url_enc = helper.encrypt_opt(item.url.as_ref())?;
    let notes_enc = helper.encrypt_opt(item.notes.as_ref())?;
    let password_enc = helper.encrypt(item.password.as_str())?;
    let totp_secret_enc = item.totp_secret.as_ref()
        .map(|s| helper.encrypt(s.as_str()))
        .transpose()?;
    
    let custom_fields_json = serde_json::to_string(&item.custom_fields)?;
    let custom_fields_enc = helper.encrypt(&custom_fields_json)?;

    let field_order_json = item.field_order.map(|fo| serde_json::to_string(&fo)).transpose()?;
    let field_order_enc = helper.encrypt_opt(field_order_json.as_ref())?;

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
        let helper = CryptoHelper::new(key.as_slice())?;
        Ok(Some(decrypt_password_item_row(&row, &helper, &db_pool).await?))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn update_password_item_tags(state: State<'_, AppState>, id: i64, tags: String) -> Result<()> {
    let key = get_key(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;
    let now = Utc::now().to_rfc3339();
    let tags_enc_opt = if tags.trim().is_empty() {
        None
    } else {
        Some(helper.encrypt(&tags)?)
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
    let helper = CryptoHelper::new(key.as_slice())?;
    let now = Utc::now().to_rfc3339();
    let totp_secret_clean = totp_secret.and_then(|secret| {
        let trimmed = secret.trim().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    });
    let totp_secret_enc = match totp_secret_clean {
        Some(secret) => Some(helper.encrypt(&secret)?),
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
pub async fn add_custom_field(
    state: State<'_, AppState>,
    item_id: i64,
    field_name: String,
    field_type: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;
    let db_pool = get_db_pool(&state).await?;

    let row = sqlx::query("SELECT custom_fields FROM password_items WHERE id = ?")
        .bind(item_id)
        .fetch_one(&db_pool)
        .await?;

    let custom_fields_enc: Option<String> = row.get("custom_fields");
    let custom_fields_json = custom_fields_enc
        .map(|cf| helper.decrypt(&cf))
        .transpose()?
        .unwrap_or_else(|| "[]".to_string());

    let mut custom_fields: Vec<CustomField> = serde_json::from_str(&custom_fields_json)?;

    custom_fields.push(CustomField {
        name: field_name,
        value: "".to_string(),
        field_type,
    });

    let updated_custom_fields_json = serde_json::to_string(&custom_fields)?;
    let updated_custom_fields_enc = helper.encrypt(&updated_custom_fields_json)?;

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
pub async fn remove_tag_from_password_items(
    state: State<'_, AppState>,
    tag: String,
) -> Result<i64> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;
    let tag_trimmed = tag.trim().to_string();
    if tag_trimmed.is_empty() { return Ok(0); }

    let mut tx = db_pool.begin().await?;
    let rows = sqlx::query("SELECT id, tags FROM password_items").fetch_all(&mut *tx).await?;
    let mut updated = 0i64;
    let now = Utc::now().to_rfc3339();

    for row in rows {
        let id: i64 = row.get("id");
        let tags_enc: Option<String> = row.get("tags");
        let tags = tags_enc.map(|t| helper.decrypt(&t)).transpose()?;
        let Some(tags_str) = tags else { continue };

        let mut parts: Vec<String> = tags_str.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).map(|t| t.to_string()).collect();
        let original_len = parts.len();
        parts.retain(|t| t != &tag_trimmed);
        if parts.len() == original_len { continue; }

        let new_tags = parts.join(", ");
        let tags_enc_opt = if new_tags.trim().is_empty() { None } else { Some(helper.encrypt(&new_tags)?) };

        sqlx::query("UPDATE password_items SET tags = ?, updated_at = ? WHERE id = ?")
            .bind(tags_enc_opt).bind(&now).bind(id).execute(&mut *tx).await?;
        updated += 1;
    }

    tx.commit().await?;
    Ok(updated)
}

#[tauri::command]
pub async fn rename_tag_in_password_items(
    state: State<'_, AppState>,
    old_tag: String,
    new_tag: String,
) -> Result<i64> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;
    let old_trimmed = old_tag.trim().to_string();
    let new_trimmed = new_tag.trim().to_string();
    if old_trimmed.is_empty() || new_trimmed.is_empty() { return Ok(0); }

    let mut tx = db_pool.begin().await?;
    let rows = sqlx::query("SELECT id, tags FROM password_items").fetch_all(&mut *tx).await?;
    let mut updated = 0i64;
    let now = Utc::now().to_rfc3339();

    for row in rows {
        let id: i64 = row.get("id");
        let tags_enc: Option<String> = row.get("tags");
        let tags = tags_enc.map(|t| helper.decrypt(&t)).transpose()?;
        let Some(tags_str) = tags else { continue };

        let mut parts: Vec<String> = tags_str.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).map(|t| t.to_string()).collect();
        let mut changed = false;
        for tag in parts.iter_mut() {
            if tag == &old_trimmed {
                *tag = new_trimmed.clone();
                changed = true;
            }
        }
        if !changed { continue; }

        let new_tags = parts.join(", ");
        let tags_enc_opt = if new_tags.trim().is_empty() { None } else { Some(helper.encrypt(&new_tags)?) };

        sqlx::query("UPDATE password_items SET tags = ?, updated_at = ? WHERE id = ?")
            .bind(tags_enc_opt).bind(&now).bind(id).execute(&mut *tx).await?;
        updated += 1;
    }

    tx.commit().await?;
    Ok(updated)
}
