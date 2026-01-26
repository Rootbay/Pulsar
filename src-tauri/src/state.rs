use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore};
use zeroize::Zeroizing;

#[derive(Debug)]
pub struct ClipboardPolicyState {
    pub integration_enabled: bool,
    pub block_history: bool,
    pub only_unlocked: bool,
    pub prior_history_setting: Option<u32>,
}

impl Default for ClipboardPolicyState {
    fn default() -> Self {
        Self {
            integration_enabled: true,
            block_history: false,
            only_unlocked: true,
            prior_history_setting: None,
        }
    }
}

pub struct AppState {
    pub db: Arc<Mutex<Option<SqlitePool>>>,
    pub key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,
    pub pending_key: Arc<Mutex<Option<PendingUnlock>>>,
    pub db_path: Arc<Mutex<Option<PathBuf>>>,
    pub rekey: Arc<Mutex<()>>,
    pub clipboard_policy: Arc<Mutex<ClipboardPolicyState>>,
    pub unlock_rate_limit: Arc<Mutex<UnlockRateLimit>>,
    pub unlock_guard: Arc<Semaphore>,
}

#[derive(Debug, Clone)]
pub struct PendingUnlock {
    pub key: Zeroizing<Vec<u8>>,
    pub created_at: Instant,
    pub attempts: u8,
}

#[derive(Debug, Clone, Default)]
pub struct UnlockRateLimit {
    pub failures: u32,
    pub last_failure: Option<Instant>,
}
