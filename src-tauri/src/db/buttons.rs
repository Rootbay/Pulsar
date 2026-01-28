use crate::state::AppState;
use crate::types::Button;
use crate::error::Result;
use crate::db::utils::{get_key, get_db_pool, CryptoHelper};
use tauri::State;
use sqlx::Row;

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
    get_key(&state).await?;
    let db_pool = get_db_pool(&state).await?;
    sqlx::query("DELETE FROM buttons WHERE id = ?")
        .bind(id)
        .execute(&db_pool)
        .await?;
    Ok(())
}
