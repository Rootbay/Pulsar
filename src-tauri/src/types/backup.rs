use crate::types::vault::{Button, PasswordItem, RecipientKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VaultBackupAttachment {
    pub id: i64,
    pub item_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: String,
    pub data_b64: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VaultBackupSnapshot {
    pub version: u32,
    pub exported_at: String,
    pub password_items: Vec<PasswordItem>,
    pub buttons: Vec<Button>,
    pub recipient_keys: Vec<RecipientKey>,
    #[serde(default)]
    pub attachments: Vec<VaultBackupAttachment>,
}
