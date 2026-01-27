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

use hmac::{Hmac, Mac};
use sha2::Sha256;

pub struct CryptoHelper {
    session: CipherSession,
    master_key: Vec<u8>,
}

impl CryptoHelper {
    pub fn new(key: &[u8]) -> Result<Self> {
        Ok(Self { 
            session: CipherSession::new(key)?,
            master_key: key.to_vec(),
        })
    }

    pub fn generate_search_token(&self, text: &str) -> Vec<u8> {
        let normalized = text.trim().to_lowercase();
        if normalized.is_empty() {
            return Vec::new();
        }

        let mut mac = Hmac::<Sha256>::new_from_slice(&self.master_key)
            .expect("HMAC can take key of any size");
        mac.update(normalized.as_bytes());
        mac.finalize().into_bytes().to_vec()
    }

    pub fn generate_trigram_hashes(&self, text: &str) -> Vec<Vec<u8>> {
        let normalized = text.trim().to_lowercase();
        if normalized.len() < 3 {
            return if normalized.is_empty() { Vec::new() } else { vec![self.generate_search_token(&normalized)] };
        }

        let chars: Vec<char> = normalized.chars().collect();
        let mut hashes = Vec::new();
        for i in 0..=chars.len() - 3 {
            let trigram: String = chars[i..i+3].iter().collect();
            hashes.push(self.generate_search_token(&trigram));
        }
        hashes
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
