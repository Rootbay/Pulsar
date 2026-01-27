use crate::state::AppState;
use crate::encryption::CipherSession;
use crate::error::{Error, Result};
use crate::types::SecretString;
use tauri::State;
use sqlx::SqlitePool;
use zeroize::Zeroizing;

pub async fn get_key(state: &State<'_, AppState>) -> Result<Zeroizing<Vec<u8>>> {
    let guard = state.key.lock().await;
    let opt = guard.clone();
    drop(guard);
    opt.ok_or(Error::VaultLocked)
}

pub async fn get_db_pool(state: &State<'_, AppState>) -> Result<SqlitePool> {
    let guard = state.db.lock().await;
    guard.clone().ok_or(Error::VaultNotLoaded)
}

pub struct CryptoHelper {
    session: CipherSession,
}

impl CryptoHelper {
    pub fn new(key: &[u8]) -> Result<Self> {
        Ok(Self { 
            session: CipherSession::new(key)? 
        })
    }

    pub fn encrypt(&self, text: &str) -> Result<String> {
        self.session.encrypt(text)
    }

    pub fn encrypt_opt(&self, text: Option<&String>) -> Result<Option<String>> {
        text.map(|t| self.encrypt(t)).transpose()
    }

    pub fn decrypt(&self, text: &str) -> Result<String> {
        self.session.decrypt(text)
    }

    pub fn decrypt_opt(&self, text: Option<String>) -> Result<Option<String>> {
        text.map(|t| self.decrypt(&t)).transpose()
    }

    pub fn decrypt_secret(&self, text: &str) -> Result<SecretString> {
        Ok(SecretString::from_zeroized(self.session.decrypt_zeroized(text)?))
    }

    pub fn decrypt_secret_opt(&self, text: Option<String>) -> Result<Option<SecretString>> {
        text.map(|t| self.decrypt_secret(&t)).transpose()
    }
}
