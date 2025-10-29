use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

#[derive(Debug)]
pub struct ClipboardPolicyState {
    pub integration_enabled: bool,
    pub block_history: bool,
    pub only_unlocked: bool,
}

impl Default for ClipboardPolicyState {
    fn default() -> Self {
        Self {
            integration_enabled: true,
            block_history: false,
            only_unlocked: true,
        }
    }
}

pub struct AppState {
    pub db: Arc<Mutex<Option<SqlitePool>>>,
    pub key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,
    pub pending_key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,
    pub db_path: Arc<Mutex<Option<PathBuf>>>,
    pub rekey: Arc<Mutex<()>>,
    pub clipboard_policy: Arc<Mutex<ClipboardPolicyState>>,
}
