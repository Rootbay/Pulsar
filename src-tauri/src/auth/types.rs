use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use argon2::Params;
use std::time::Duration;

pub const PENDING_TOTP_TTL: Duration = Duration::from_secs(120);
pub const MAX_TOTP_ATTEMPTS: u8 = 5;
pub const UNLOCK_BACKOFF_BASE_MS: u64 = 250;
pub const UNLOCK_BACKOFF_MAX_MS: u64 = 5000;
pub const ARGON2_MIN_MEMORY_KIB: u32 = 8 * 1024;
pub const ARGON2_MAX_MEMORY_KIB: u32 = 1024 * 1024;
pub const ARGON2_MAX_TIME_COST: u32 = 10;
pub const ARGON2_MAX_PARALLELISM: u32 = 16;
pub const UNLOCK_CONCURRENCY_LIMIT: usize = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordMetadata {
    pub version: u8,
    pub salt_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    #[serde(default)]
    pub argon2_memory_kib: Option<u32>,
    #[serde(default)]
    pub argon2_time_cost: Option<u32>,
    #[serde(default)]
    pub argon2_parallelism: Option<u32>,
    #[serde(default)]
    pub mac_version: Option<u8>,
    #[serde(default)]
    pub mac_nonce_b64: Option<String>,
    #[serde(default)]
    pub mac_tag_b64: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Argon2ParamsConfig {
    pub memory_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Argon2ParamsResponse {
    pub memory_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl From<Argon2ParamsConfig> for Argon2ParamsResponse {
    fn from(value: Argon2ParamsConfig) -> Self {
        Self {
            memory_kib: value.memory_kib,
            time_cost: value.time_cost,
            parallelism: value.parallelism,
        }
    }
}

impl Default for Argon2ParamsConfig {
    fn default() -> Self {
        Self {
            memory_kib: 64 * 1024,
            time_cost: 3,
            parallelism: 4,
        }
    }
}

impl Argon2ParamsConfig {
    pub fn to_params(&self) -> Result<Params> {
        Params::new(self.memory_kib, self.time_cost, self.parallelism, None)
            .map_err(|e| Error::Internal(format!("Invalid Argon2 parameters: {}", e)))
    }
}

impl PasswordMetadata {
    pub fn argon2_params(&self) -> Argon2ParamsConfig {
        let defaults = Argon2ParamsConfig::default();
        Argon2ParamsConfig {
            memory_kib: self.argon2_memory_kib.unwrap_or(defaults.memory_kib),
            time_cost: self.argon2_time_cost.unwrap_or(defaults.time_cost),
            parallelism: self.argon2_parallelism.unwrap_or(defaults.parallelism),
        }
    }
}

#[derive(Serialize)]
pub struct UnlockResponse {
    pub totp_required: bool,
}
