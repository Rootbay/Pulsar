use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Vault is locked")]
    VaultLocked,

    #[error("Vault not loaded")]
    VaultNotLoaded,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("TOTP error: {0}")]
    Totp(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::Database(_) => "Database",
            Error::Io(_) => "Io",
            Error::Encryption(_) => "Encryption",
            Error::Decryption(_) => "Decryption",
            Error::Validation(_) => "Validation",
            Error::Internal(_) => "Internal",
            Error::VaultLocked => "VaultLocked",
            Error::VaultNotLoaded => "VaultNotLoaded",
            Error::InvalidPassword => "InvalidPassword",
            Error::Totp(_) => "Totp",
            Error::Serialization(_) => "Serialization",
            Error::Tauri(_) => "Tauri",
        }
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Error", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
