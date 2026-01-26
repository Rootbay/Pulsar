use serde::{Deserialize, Serialize, Serializer, Deserializer};
use validator::Validate;
use zeroize::{Zeroizing};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretString(Zeroizing<String>);

impl std::hash::Hash for SecretString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}

impl SecretString {
    pub fn new(s: String) -> Self {
        Self(Zeroizing::new(s))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SecretString {
    fn default() -> Self {
        Self(Zeroizing::new(String::new()))
    }
}

impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SecretString(Zeroizing::new(s)))
    }
}

impl std::ops::Deref for SecretString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for SecretString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Button {
    pub id: i64,
    pub text: String,
    pub icon: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomField {
    pub name: String,
    pub value: String,
    pub field_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: i64,
    pub item_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: String,
}

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

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[validate(schema(function = "crate::db_commands::validate_password_item_fields"))]
pub struct PasswordItem {
    pub id: i64,
    pub category: String,
    pub title: String,
    pub description: Option<String>,
    pub img: Option<String>,
    pub tags: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub password: SecretString,
    pub created_at: String,
    pub updated_at: String,
    pub color: Option<String>,
    pub totp_secret: Option<SecretString>,
    pub custom_fields: Vec<CustomField>,
    pub field_order: Option<Vec<String>>,
    pub attachments: Option<Vec<Attachment>>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipientKey {
    pub id: i64,
    pub name: String,
    pub public_key: String,
    pub private_key: String,
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

#[derive(Serialize, Deserialize)]
pub struct ExportPayload {
    pub version: u8,
    pub salt_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

#[derive(Serialize, Deserialize)]
pub struct PubKeyExportPayload {
    pub version: u8,
    pub scheme: String,
    pub kdf: String,
    pub enc: String,
    pub recipient_pub_b64: String,
    pub eph_pub_b64: String,
    pub salt_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}
