use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

pub struct AppState {
    pub db: Arc<Mutex<Option<SqlitePool>>>,
    pub key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,
    pub rekey: Arc<Mutex<()>>,
}
