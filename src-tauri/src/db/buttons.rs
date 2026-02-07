use crate::db::utils::{get_db_pool, get_key, CryptoHelper};
use crate::error::Result;
use crate::state::AppState;
use crate::types::Button;
use chrono::Utc;
use sqlx::Row;
use tauri::State;

#[tauri::command]
pub async fn save_button(
    state: State<'_, AppState>,
    text: String,
    icon: String,
    color: String,
) -> Result<()> {
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;

    let text_enc = helper.encrypt(&text)?;
    let icon_enc = helper.encrypt(&icon)?;
    let color_enc = helper.encrypt(&color)?;

    sqlx::query("INSERT INTO buttons (text, icon, color) VALUES (?, ?, ?)")
        .bind(text_enc)
        .bind(icon_enc)
        .bind(color_enc)
        .execute(&db_pool)
        .await?;
    Ok(())
}

pub async fn get_buttons_impl<'a, E>(executor: E, key: &[u8]) -> Result<Vec<Button>>
where
    E: sqlx::SqliteExecutor<'a>,
{
    let rows = sqlx::query("SELECT id, text, icon, color FROM buttons")
        .fetch_all(executor)
        .await?;

    let helper = CryptoHelper::new(key)?;
    let mut buttons = Vec::new();
    for row in rows {
        let text_enc: String = row.get("text");
        let icon_enc: String = row.get("icon");
        let color_enc: String = row.get("color");

        buttons.push(Button {
            id: row.get("id"),
            text: helper.decrypt(&text_enc)?,
            icon: helper.decrypt(&icon_enc)?,
            color: helper.decrypt(&color_enc)?,
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
    let helper = CryptoHelper::new(key.as_slice())?;

    let text_enc = helper.encrypt(&text)?;
    let icon_enc = helper.encrypt(&icon)?;
    let color_enc = helper.encrypt(&color)?;

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
    let key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    let helper = CryptoHelper::new(key.as_slice())?;
    
    let mut tx = db_pool.begin().await?;

    // 1. Get the tag text
    let row: Option<(String,)> = sqlx::query_as("SELECT text FROM buttons WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        
    let tag_text = if let Some((text_enc,)) = row {
        Some(helper.decrypt(&text_enc)?)
    } else {
        None
    };

    // 2. Get affected item IDs
    let item_ids: Vec<i64> = sqlx::query_scalar("SELECT item_id FROM item_tags WHERE tag_id = ?")
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

    // 3. Remove associations from item_tags
    sqlx::query("DELETE FROM item_tags WHERE tag_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // 4. Delete the button itself
    sqlx::query("DELETE FROM buttons WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // 5. Update password_items tags text
    if let Some(tag_trimmed) = tag_text {
        let tag_trimmed = tag_trimmed.trim();
        let now = Utc::now().to_rfc3339();
        
        for item_id in item_ids {
            let tags_enc: Option<String> =
                sqlx::query_scalar("SELECT tags FROM password_items WHERE id = ?")
                    .bind(item_id)
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
                
                // Remove the specific tag
                parts.retain(|t| t != tag_trimmed);

                let new_tags = parts.join(", ");
                let tags_enc_opt = if new_tags.trim().is_empty() {
                    None
                } else {
                    Some(helper.encrypt(&new_tags)?)
                };

                sqlx::query("UPDATE password_items SET tags = ?, updated_at = ? WHERE id = ?")
                    .bind(tags_enc_opt)
                    .bind(&now)
                    .bind(item_id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(())
}

#[tauri::command]
pub async fn get_tag_counts(state: State<'_, AppState>) -> Result<std::collections::HashMap<i64, i64>> {
    let _key = get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;

    let rows = sqlx::query("SELECT tag_id, COUNT(*) as count FROM item_tags GROUP BY tag_id")
        .fetch_all(&db_pool)
        .await?;

    let mut counts = std::collections::HashMap::new();
    for row in rows {
        let tag_id: i64 = row.get("tag_id");
        let count: i64 = row.get("count");
        counts.insert(tag_id, count);
    }

    Ok(counts)
}
