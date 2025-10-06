use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

pub struct AppState {
    pub db: Arc<Mutex<Option<SqlitePool>>>,
    pub key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,
    pub pending_key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,
    pub db_path: Arc<Mutex<Option<PathBuf>>>,
    pub rekey: Arc<Mutex<()>>,
}
