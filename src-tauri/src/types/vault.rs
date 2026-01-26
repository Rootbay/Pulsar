use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::types::secret::SecretString;

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
pub struct Attachment {
    pub id: i64,
    pub item_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[validate(schema(function = "crate::db::validation::validate_password_item_fields"))]
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
