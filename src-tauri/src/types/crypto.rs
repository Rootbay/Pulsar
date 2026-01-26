use serde::{Deserialize, Serialize};

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
