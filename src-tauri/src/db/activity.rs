use crate::state::AppState;
use crate::error::Result;
use crate::db::utils::{get_db_pool, get_key};
use crate::encryption::{encrypt, decrypt};
use tauri::State;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub id: i64,
    pub event_type: String,
    pub item_id: Option<i64>,
    pub item_title: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

pub async fn log_activity_impl<'a, E>(
    executor: E,
    key: &[u8],
    event_type: &str,
    item_id: Option<i64>,
    item_title: Option<&str>,
    details: Option<&str>,
) -> Result<()>
where
    E: sqlx::SqliteExecutor<'a>,
{
    let item_title_enc = item_title.map(|t| encrypt(t, key)).transpose()?;
    let details_enc = details.map(|d| encrypt(d, key)).transpose()?;

    sqlx::query(
        "INSERT INTO activity_log (event_type, item_id, item_title, details) VALUES (?, ?, ?, ?)",
    )
    .bind(event_type)
    .bind(item_id)
    .bind(item_title_enc)
    .bind(details_enc)
    .execute(executor)
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn get_activity_log(state: State<'_, AppState>, limit: i64) -> Result<Vec<ActivityEntry>> {
    let key = get_key(&state).await?;
    let pool = get_db_pool(&state).await?;

    let rows = sqlx::query("SELECT id, event_type, item_id, item_title, details, created_at FROM activity_log ORDER BY created_at DESC LIMIT ?")
        .bind(limit)
        .fetch_all(&pool)
        .await?;

    let mut entries = Vec::new();
    for row in rows {
        let item_title_enc: Option<String> = row.get("item_title");
        let details_enc: Option<String> = row.get("details");

        entries.push(ActivityEntry {
            id: row.get("id"),
            event_type: row.get("event_type"),
            item_id: row.get("item_id"),
            item_title: item_title_enc.map(|t| decrypt(&t, key.as_slice())).transpose()?,
            details: details_enc.map(|d| decrypt(&d, key.as_slice())).transpose()?,
            created_at: row.get("created_at"),
        });
    }

    Ok(entries)
}

#[tauri::command]
pub async fn clear_activity_log(state: State<'_, AppState>) -> Result<()> {
    let pool = get_db_pool(&state).await?;
    sqlx::query("DELETE FROM activity_log").execute(&pool).await?;
    Ok(())
}
