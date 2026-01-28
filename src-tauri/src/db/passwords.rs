use crate::db::utils::{get_db_pool, get_key, CryptoHelper};
use crate::error::{Error, Result};
use crate::state::AppState;
use crate::types::{Attachment, CustomField, PasswordItem, PasswordItemOverview};
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use tauri::State;
use validator::Validate;

async fn fetch_attachments_for_item(
    pool: &SqlitePool,
    helper: &CryptoHelper,
    item_id: i64,
) -> Result<Vec<Attachment>> {
    let rows = sqlx::query(
        "SELECT id, file_name, file_size, mime_type, created_at FROM attachments WHERE item_id = ?",
    )
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

async fn decrypt_password_item_row(
    row: &sqlx::sqlite::SqliteRow,
    helper: &CryptoHelper,
    db_pool: &SqlitePool,
) -> Result<PasswordItem> {
    let id: i64 = row.get("id");

    let category_enc: String = row.get("category");
    let category = helper
        .decrypt(&category_enc)
        .unwrap_or_else(|_| "login".to_string());

    let title_enc: String = row.get("title");
    let title = helper.decrypt(&title_enc)?;

    let description = helper.decrypt_opt(row.get("description"))?;
    let img = helper.decrypt_opt(row.get("img"))?;
    let tags = helper.decrypt_opt(row.get("tags"))?;
    let username = helper.decrypt_opt(row.get("username"))?;
    let url = helper.decrypt_opt(row.get("url"))?;
    let notes = helper.decrypt_secret_opt(row.get("notes"))?;

    let password_enc: String = row.get("password");
    let password = helper.decrypt_secret(&password_enc)?;

    let totp_secret_enc: Option<String> = row.get("totp_secret");
    let totp_secret = totp_secret_enc
        .map(|t| helper.decrypt_secret(&t))
        .transpose()?;

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

fn decrypt_password_item_overview_row(
    row: &sqlx::sqlite::SqliteRow,
    helper: &CryptoHelper,
) -> Result<PasswordItemOverview> {
    let id: i64 = row.get("id");

    let category_enc: String = row.get("category");
    let category = helper
        .decrypt(&category_enc)
        .unwrap_or_else(|_| "login".to_string());

    let title_enc: String = row.get("title");
    let title = helper.decrypt(&title_enc)?;

    let description = helper.decrypt_opt(row.get("description"))?;
    let img = helper.decrypt_opt(row.get("img"))?;
    let tags = helper.decrypt_opt(row.get("tags"))?;
    let username = helper.decrypt_opt(row.get("username"))?;
    let url = helper.decrypt_opt(row.get("url"))?;

    Ok(PasswordItemOverview {
        id,
        category,
        title,
        description,
        img,
        tags,
        username,
        url,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        color: row.get("color"),
    })
}

async fn sync_item_tags(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: i64,
    tags: Option<&String>,
    key: &[u8],
) -> Result<()> {
    sqlx::query("DELETE FROM item_tags WHERE item_id = ?")
        .bind(item_id)
        .execute(&mut **tx)
        .await?;

    let Some(tags_str) = tags else {
        return Ok(());
    };

    let tag_names: Vec<String> = tags_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tag_names.is_empty() {
        return Ok(());
    }

    let buttons = crate::db::buttons::get_buttons_impl(tx.as_mut(), key).await?;

    for name in tag_names {
        if let Some(button) = buttons.iter().find(|b| b.text == name) {
            sqlx::query("INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?, ?)")
                .bind(item_id)
                .bind(button.id)
                .execute(tx.as_mut())
                .await?;
        }
    }

    Ok(())
}

async fn sync_search_indices(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: i64,
    helper: &CryptoHelper,
    title: &str,
    username: Option<&String>,
    tags: Option<&String>,
) -> Result<()> {
    sqlx::query("DELETE FROM search_indices WHERE item_id = ?")
        .bind(item_id)
        .execute(tx.as_mut())
        .await?;

    sqlx::query("DELETE FROM search_trigrams WHERE item_id = ?")
        .bind(item_id)
        .execute(tx.as_mut())
        .await?;

    let mut all_searchable_text = title.to_string();
    if let Some(uname) = username {
        all_searchable_text.push(' ');
        all_searchable_text.push_str(uname);
    }
    if let Some(t) = tags {
        all_searchable_text.push(' ');
        all_searchable_text.push_str(t);
    }

    let title_token = helper.generate_search_token(title);
    if !title_token.is_empty() {
        sqlx::query("INSERT INTO search_indices (item_id, field_name, token) VALUES (?, ?, ?)")
            .bind(item_id)
            .bind("title")
            .bind(title_token)
            .execute(tx.as_mut())
            .await?;
    }

    if let Some(uname) = username {
        let uname_token = helper.generate_search_token(uname);
        if !uname_token.is_empty() {
            sqlx::query("INSERT INTO search_indices (item_id, field_name, token) VALUES (?, ?, ?)")
                .bind(item_id)
                .bind("username")
                .bind(uname_token)
                .execute(tx.as_mut())
                .await?;
        }
    }

    let trigrams = helper.generate_trigram_hashes(&all_searchable_text);
    for hash in trigrams {
        sqlx::query("INSERT OR IGNORE INTO search_trigrams (item_id, trigram_hash) VALUES (?, ?)")
            .bind(item_id)
            .bind(hash)
            .execute(tx.as_mut())
            .await?;
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn get_password_overviews_impl(
    db_pool: &SqlitePool,
    key: &[u8],
) -> Result<Vec<PasswordItemOverview>> {
    let rows = sqlx::query("SELECT id, category, title, description, img, tags, username, url, created_at, updated_at, color FROM password_items")
        .fetch_all(db_pool)
        .await?;

    let helper = CryptoHelper::new(key)?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(decrypt_password_item_overview_row(&row, &helper)?);
    }

    Ok(items)
}

#[tauri::command]
pub async fn search_password_items(
    state: State<'_, AppState>,
    query: String,
    tag_id: Option<i64>,
    category: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<PasswordItemOverview>> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;

    let query_trimmed = query.trim();

    let mut sql = "SELECT DISTINCT p.id, p.category, p.title, p.description, p.img, p.tags, p.username, p.url, p.created_at, p.updated_at, p.color 
                   FROM password_items p".to_string();

    if tag_id.is_some() {
        sql.push_str(" JOIN item_tags it ON p.id = it.item_id AND it.tag_id = ?");
    }

    let mut conditions = Vec::new();

    if !query_trimmed.is_empty() {
        let _token = helper.generate_search_token(query_trimmed);
        let trigrams = helper.generate_trigram_hashes(query_trimmed);

        if trigrams.len() >= 2 {
            let threshold = (trigrams.len() as f64 * 0.6).ceil() as usize;
            conditions.push(format!(
                "p.id IN (
                    SELECT item_id FROM search_trigrams 
                    WHERE trigram_hash IN ({}) 
                    GROUP BY item_id 
                    HAVING COUNT(trigram_hash) >= {}
                )",
                trigrams.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                threshold
            ));
        } else {
            conditions
                .push("p.id IN (SELECT item_id FROM search_indices WHERE token = ?)".to_string());
        }
    }

    if let Some(cat) = &category {
        match cat.as_str() {
            "recent" => {
                let pin_tags = ["pinned", "pin"]
                    .iter()
                    .map(|t| helper.encrypt(t).unwrap_or_default())
                    .collect::<Vec<_>>();

                let placeholders = pin_tags.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

                conditions.push(format!(
                    "(p.id IN (SELECT item_id FROM item_tags JOIN buttons ON item_tags.tag_id = buttons.id WHERE buttons.text IN ({})) OR p.updated_at >= datetime('now', '-7 days'))",
                    placeholders
                ));
            }
            "favorites" => {
                let fav_tags = ["favorite", "fav", "star"]
                    .iter()
                    .map(|t| helper.encrypt(t).unwrap_or_default())
                    .collect::<Vec<_>>();

                let placeholders = fav_tags.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

                conditions.push(format!(
                    "p.id IN (SELECT item_id FROM item_tags JOIN buttons ON item_tags.tag_id = buttons.id WHERE buttons.text IN ({}))",
                    placeholders
                ));
            }
            _ => {}
        }
    }

    if !conditions.is_empty() {
        sql.push_str(if tag_id.is_some() { " AND " } else { " WHERE " });
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY p.updated_at DESC");

    if let Some(l) = limit {
        sql.push_str(&format!(" LIMIT {}", l));
        if let Some(o) = offset {
            sql.push_str(&format!(" OFFSET {}", o));
        }
    }

    let mut q = sqlx::query(&sql);
    if let Some(tid) = tag_id {
        q = q.bind(tid);
    }

    if !query_trimmed.is_empty() {
        let trigrams = helper.generate_trigram_hashes(query_trimmed);
        if trigrams.len() >= 2 {
            for hash in trigrams {
                q = q.bind(hash);
            }
        } else {
            let token = helper.generate_search_token(query_trimmed);
            q = q.bind(token);
        }
    }

    if let Some(cat) = category {
        match cat.as_str() {
            "recent" => {
                let pin_tags = ["pinned", "pin"]
                    .iter()
                    .map(|t| helper.encrypt(t).unwrap_or_default())
                    .collect::<Vec<_>>();
                for tag in pin_tags {
                    q = q.bind(tag);
                }
            }
            "favorites" => {
                let fav_tags = ["favorite", "fav", "star"]
                    .iter()
                    .map(|t| helper.encrypt(t).unwrap_or_default())
                    .collect::<Vec<_>>();
                for tag in fav_tags {
                    q = q.bind(tag);
                }
            }
            _ => {}
        }
    }

    let rows = q.fetch_all(&db_pool).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(decrypt_password_item_overview_row(&row, &helper)?);
    }

    Ok(items)
}

#[tauri::command]
pub async fn get_password_overviews(
    state: State<'_, AppState>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<PasswordItemOverview>> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let mut sql = "SELECT id, category, title, description, img, tags, username, url, created_at, updated_at, color FROM password_items ORDER BY updated_at DESC".to_string();

    if let Some(l) = limit {
        sql.push_str(&format!(" LIMIT {}", l));
        if let Some(o) = offset {
            sql.push_str(&format!(" OFFSET {}", o));
        }
    }

    let rows = sqlx::query(&sql).fetch_all(&db_pool).await?;

    let helper = CryptoHelper::new(key.as_slice())?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(decrypt_password_item_overview_row(&row, &helper)?);
    }

    Ok(items)
}

#[tauri::command]
pub async fn get_password_overviews_by_ids(
    state: State<'_, AppState>,
    ids: Vec<i64>,
) -> Result<Vec<PasswordItemOverview>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;

    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT id, category, title, description, img, tags, username, url, created_at, updated_at, color 
         FROM password_items WHERE id IN ({})",
        placeholders
    );

    let mut q = sqlx::query(&sql);
    for id in ids {
        q = q.bind(id);
    }

    let rows = q.fetch_all(&db_pool).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(decrypt_password_item_overview_row(&row, &helper)?);
    }

    Ok(items)
}

pub async fn get_password_items_impl(
    db_pool: &SqlitePool,
    key: &[u8],
) -> Result<Vec<PasswordItem>> {
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
pub async fn save_password_item(state: State<'_, AppState>, item: PasswordItem) -> Result<i64> {
    item.validate()
        .map_err(|e| Error::Validation(e.to_string()))?;

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
    let notes_enc = helper.encrypt_opt(item.notes.as_ref().map(|v| &**v))?;
    let password_enc = helper.encrypt(item.password.as_str())?;
    let totp_secret_enc = item
        .totp_secret
        .as_ref()
        .map(|s| helper.encrypt(s.as_str()))
        .transpose()?;

    let custom_fields_json = serde_json::to_string(&item.custom_fields)?;
    let custom_fields_enc = helper.encrypt(&custom_fields_json)?;

    let field_order_json = item
        .field_order
        .as_ref()
        .map(|fo| serde_json::to_string(&fo))
        .transpose()?;
    let field_order_enc = helper.encrypt_opt(field_order_json.as_ref())?;

    let db_pool = get_db_pool(&state).await?;
    let mut tx = db_pool.begin().await?;

    let item_id = sqlx::query("INSERT INTO password_items (category, title, description, img, tags, username, url, notes, password, created_at, updated_at, color, totp_secret, custom_fields, field_order) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
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
        .execute(tx.as_mut())
        .await?
        .last_insert_rowid();

    sync_item_tags(&mut tx, item_id, item.tags.as_ref(), key.as_slice()).await?;
    sync_search_indices(
        &mut tx,
        item_id,
        &helper,
        &item.title,
        item.username.as_ref(),
        item.tags.as_ref(),
    )
    .await?;

    let _ = crate::db::activity::log_activity_impl(
        tx.as_mut(),
        key.as_slice(),
        "item_created",
        Some(item_id),
        Some(&item.title),
        Some("New password item created"),
    )
    .await;

    tx.commit().await?;
    Ok(item_id)
}

#[tauri::command]
pub async fn update_password_item(state: State<'_, AppState>, item: PasswordItem) -> Result<()> {
    item.validate()
        .map_err(|e| Error::Validation(e.to_string()))?;

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
    let notes_enc = helper.encrypt_opt(item.notes.as_ref().map(|v| &**v))?;
    let password_enc = helper.encrypt(item.password.as_str())?;
    let totp_secret_enc = item
        .totp_secret
        .as_ref()
        .map(|s| helper.encrypt(s.as_str()))
        .transpose()?;

    let custom_fields_json = serde_json::to_string(&item.custom_fields)?;
    let custom_fields_enc = helper.encrypt(&custom_fields_json)?;

    let field_order_json = item
        .field_order
        .as_ref()
        .map(|fo| serde_json::to_string(&fo))
        .transpose()?;
    let field_order_enc = helper.encrypt_opt(field_order_json.as_ref())?;

    let db_pool = get_db_pool(&state).await?;
    let mut tx = db_pool.begin().await?;

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
        .execute(tx.as_mut())
        .await?;

    sync_item_tags(&mut tx, item.id, item.tags.as_ref(), key.as_slice()).await?;
    sync_search_indices(
        &mut tx,
        item.id,
        &helper,
        &item.title,
        item.username.as_ref(),
        item.tags.as_ref(),
    )
    .await?;

    let _ = crate::db::activity::log_activity_impl(
        tx.as_mut(),
        key.as_slice(),
        "item_updated",
        Some(item.id),
        Some(&item.title),
        Some("Password item updated"),
    )
    .await;

    tx.commit().await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_password_item(state: State<'_, AppState>, id: i64) -> Result<()> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let title_enc: Option<String> =
        sqlx::query_scalar("SELECT title FROM password_items WHERE id = ?")
            .bind(id)
            .fetch_optional(&db_pool)
            .await?;

    let helper = CryptoHelper::new(key.as_slice())?;
    let title = title_enc.and_then(|t| helper.decrypt(&t).ok());

    let mut tx = db_pool.begin().await?;

    sqlx::query("DELETE FROM attachments WHERE item_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM password_items WHERE id = ?")
        .bind(id)
        .execute(tx.as_mut())
        .await?;

    let _ = crate::db::activity::log_activity_impl(
        tx.as_mut(),
        key.as_slice(),
        "item_deleted",
        Some(id),
        title.as_deref(),
        Some("Password item deleted"),
    )
    .await;

    tx.commit().await?;
    Ok(())
}

#[tauri::command]
pub async fn get_password_item_by_id(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<PasswordItem>> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let row = sqlx::query("SELECT id, category, title, description, img, tags, username, url, notes, password, created_at, updated_at, color, totp_secret, custom_fields, field_order FROM password_items WHERE id = ?")
        .bind(id)
        .fetch_optional(&db_pool)
        .await?;

    if let Some(row) = row {
        let helper = CryptoHelper::new(key.as_slice())?;
        Ok(Some(
            decrypt_password_item_row(&row, &helper, &db_pool).await?,
        ))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn update_password_item_tags(
    state: State<'_, AppState>,
    id: i64,
    tags: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;
    let now = Utc::now().to_rfc3339();

    let db_pool = get_db_pool(&state).await?;
    let mut tx = db_pool.begin().await?;

    let row = sqlx::query("SELECT title, username FROM password_items WHERE id = ?")
        .bind(id)
        .fetch_one(tx.as_mut())
        .await?;

    let title_enc: String = row.get("title");
    let username_enc: Option<String> = row.get("username");

    let tags_enc_opt = if tags.trim().is_empty() {
        None
    } else {
        Some(helper.encrypt(&tags)?)
    };

    sqlx::query("UPDATE password_items SET tags = ?, updated_at = ? WHERE id = ?")
        .bind(&tags_enc_opt)
        .bind(&now)
        .bind(id)
        .execute(tx.as_mut())
        .await?;

    let title = helper.decrypt(&title_enc)?;
    let username = username_enc.map(|u| helper.decrypt(&u)).transpose()?;

    sync_search_indices(&mut tx, id, &helper, &title, username.as_ref(), Some(&tags)).await?;

    tx.commit().await?;
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
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    let totp_secret_enc = match totp_secret_clean {
        Some(secret) => Some(helper.encrypt(&secret)?),
        None => None,
    };
    let db_pool = get_db_pool(&state).await?;
    let mut tx = db_pool.begin().await?;

    sqlx::query("UPDATE password_items SET totp_secret = ?, updated_at = ? WHERE id = ?")
        .bind(totp_secret_enc)
        .bind(&now)
        .bind(id)
        .execute(tx.as_mut())
        .await?;

    let row = sqlx::query("SELECT title, username, tags FROM password_items WHERE id = ?")
        .bind(id)
        .fetch_one(tx.as_mut())
        .await?;

    let title_enc: String = row.get("title");
    let username_enc: Option<String> = row.get("username");
    let tags_enc: Option<String> = row.get("tags");

    let title = helper.decrypt(&title_enc)?;
    let username = username_enc.map(|u| helper.decrypt(&u)).transpose()?;
    let tags = tags_enc.map(|t| helper.decrypt(&t)).transpose()?;

    sync_search_indices(
        &mut tx,
        id,
        &helper,
        &title,
        username.as_ref(),
        tags.as_ref(),
    )
    .await?;

    tx.commit().await?;
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
    if tag_trimmed.is_empty() {
        return Ok(0);
    }

    let mut tx = db_pool.begin().await?;

    let tag_id: Option<i64> = sqlx::query_scalar("SELECT id FROM buttons WHERE text = ?")
        .bind(helper.encrypt(&tag_trimmed)?)
        .fetch_optional(&mut *tx)
        .await?;

    let Some(tag_id) = tag_id else {
        return Ok(0);
    };

    let item_ids: Vec<i64> = sqlx::query_scalar("SELECT item_id FROM item_tags WHERE tag_id = ?")
        .bind(tag_id)
        .fetch_all(&mut *tx)
        .await?;

    let updated_count = item_ids.len() as i64;

    sqlx::query("DELETE FROM item_tags WHERE tag_id = ?")
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;

    let now = Utc::now().to_rfc3339();
    for id in item_ids {
        let tags_enc: Option<String> =
            sqlx::query_scalar("SELECT tags FROM password_items WHERE id = ?")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

        if let Some(t_enc) = tags_enc {
            let tags_str = helper.decrypt(&t_enc)?;
            let mut parts: Vec<String> = tags_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            parts.retain(|t| t != &tag_trimmed);

            let new_tags = parts.join(", ");
            let tags_enc_opt = if new_tags.trim().is_empty() {
                None
            } else {
                Some(helper.encrypt(&new_tags)?)
            };

            sqlx::query("UPDATE password_items SET tags = ?, updated_at = ? WHERE id = ?")
                .bind(tags_enc_opt)
                .bind(&now)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(updated_count)
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
    if old_trimmed.is_empty() || new_trimmed.is_empty() {
        return Ok(0);
    }

    let mut tx = db_pool.begin().await?;

    let tag_id: Option<i64> = sqlx::query_scalar("SELECT id FROM buttons WHERE text = ?")
        .bind(helper.encrypt(&old_trimmed)?)
        .fetch_optional(&mut *tx)
        .await?;

    let Some(tag_id) = tag_id else {
        return Ok(0);
    };

    let item_ids: Vec<i64> = sqlx::query_scalar("SELECT item_id FROM item_tags WHERE tag_id = ?")
        .bind(tag_id)
        .fetch_all(&mut *tx)
        .await?;

    let updated_count = item_ids.len() as i64;

    let now = Utc::now().to_rfc3339();
    for id in item_ids {
        let tags_enc: Option<String> =
            sqlx::query_scalar("SELECT tags FROM password_items WHERE id = ?")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

        if let Some(t_enc) = tags_enc {
            let tags_str = helper.decrypt(&t_enc)?;
            let mut parts: Vec<String> = tags_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            let mut changed = false;
            for tag in parts.iter_mut() {
                if tag == &old_trimmed {
                    *tag = new_trimmed.clone();
                    changed = true;
                }
            }

            if changed {
                let new_tags = parts.join(", ");
                let tags_enc_opt = Some(helper.encrypt(&new_tags)?);

                sqlx::query("UPDATE password_items SET tags = ?, updated_at = ? WHERE id = ?")
                    .bind(tags_enc_opt)
                    .bind(&now)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(updated_count)
}
