use crate::auth::crypto_utils::derive_metadata_mac_key;
use crate::auth::types::PasswordMetadata;
use crate::error::{Error, Result};
use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    Key, XChaCha20Poly1305, XNonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;
use tokio::fs;

pub fn metadata_path(db_path: &Path) -> PathBuf {
    let file_name = db_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("vault.db");
    let meta_name = format!("{}.meta.json", file_name);
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(meta_name)
}

pub async fn read_password_metadata(db_path: &Path) -> Result<Option<PasswordMetadata>> {
    let path = metadata_path(db_path);
    match fs::read(&path).await {
        Ok(bytes) => {
            let meta: PasswordMetadata = serde_json::from_slice(&bytes)?;
            Ok(Some(meta))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(Error::Io(err)),
    }
}

pub async fn write_password_metadata(
    db_path: &Path,
    meta: &PasswordMetadata,
    mac_key: Option<&[u8]>,
) -> Result<()> {
    let path = metadata_path(db_path);
    let tmp_path = path.with_extension("meta.json.tmp");
    let mut meta = meta.clone();
    if let Some(key) = mac_key {
        let vault_id = get_vault_id(db_path);
        let (nonce_b64, tag_b64) = compute_metadata_mac(&meta, &vault_id, key)?;
        meta.mac_version = Some(1);
        meta.mac_nonce_b64 = Some(nonce_b64);
        meta.mac_tag_b64 = Some(tag_b64);
    }

    let bytes = serde_json::to_vec_pretty(&meta)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path)
            .await?;
        let _ = file
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .await;
        file.write_all(&bytes).await?;
        file.sync_all().await?;
        fs::rename(&tmp_path, &path).await?;
        if let Ok(dir) = fs::File::open(path.parent().unwrap_or_else(|| Path::new("."))).await {
            let _ = dir.sync_all().await;
        }
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        fs::write(&tmp_path, bytes).await?;
        fs::rename(&tmp_path, &path).await?;
        Ok(())
    }
}

pub fn decode_metadata(meta: &PasswordMetadata) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let salt = general_purpose::STANDARD
        .decode(&meta.salt_b64)
        .map_err(|e| Error::Internal(format!("Invalid salt encoding: {}", e)))?;
    let nonce = general_purpose::STANDARD
        .decode(&meta.nonce_b64)
        .map_err(|e| Error::Internal(format!("Invalid nonce encoding: {}", e)))?;
    if nonce.len() != 24 {
        return Err(Error::Validation("Invalid nonce length".to_string()));
    }
    let ciphertext = general_purpose::STANDARD
        .decode(&meta.ciphertext_b64)
        .map_err(|e| Error::Internal(format!("Invalid ciphertext encoding: {}", e)))?;
    Ok((salt, nonce, ciphertext))
}

#[derive(Serialize)]
struct MetadataMacPayload<'a> {
    vault_id: &'a str,
    version: u8,
    salt_b64: &'a str,
    nonce_b64: &'a str,
    ciphertext_b64: &'a str,
    argon2_memory_kib: Option<u32>,
    argon2_time_cost: Option<u32>,
    argon2_parallelism: Option<u32>,
}

fn metadata_mac_payload(meta: &PasswordMetadata, vault_id: &str) -> Result<Vec<u8>> {
    serde_json::to_vec(&MetadataMacPayload {
        vault_id,
        version: meta.version,
        salt_b64: &meta.salt_b64,
        nonce_b64: &meta.nonce_b64,
        ciphertext_b64: &meta.ciphertext_b64,
        argon2_memory_kib: meta.argon2_memory_kib,
        argon2_time_cost: meta.argon2_time_cost,
        argon2_parallelism: meta.argon2_parallelism,
    })
    .map_err(|e| Error::Internal(format!("Failed to serialize metadata MAC payload: {}", e)))
}

pub fn compute_metadata_mac(
    meta: &PasswordMetadata,
    vault_id: &str,
    master_key: &[u8],
) -> Result<(String, String)> {
    let mac_key = derive_metadata_mac_key(master_key)?;
    let payload = metadata_mac_payload(meta, vault_id)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&mac_key));

    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let tag = cipher
        .encrypt(
            nonce,
            Payload {
                msg: b"",
                aad: &payload,
            },
        )
        .map_err(|e| Error::Encryption(format!("Metadata MAC failed: {}", e)))?;

    let nonce_b64 = general_purpose::STANDARD.encode(nonce_bytes);
    let tag_b64 = general_purpose::STANDARD.encode(tag);
    Ok((nonce_b64, tag_b64))
}

pub fn verify_metadata_mac(
    meta: &PasswordMetadata,
    vault_id: &str,
    master_key: &[u8],
) -> Result<()> {
    let mac_key = derive_metadata_mac_key(master_key)?;
    if meta.mac_version.unwrap_or(1) != 1 {
        return Err(Error::Validation(
            "Unsupported metadata MAC version".to_string(),
        ));
    }
    let nonce_b64 = meta
        .mac_nonce_b64
        .as_deref()
        .ok_or_else(|| Error::Validation("Missing metadata MAC nonce".to_string()))?;
    let tag_b64 = meta
        .mac_tag_b64
        .as_deref()
        .ok_or_else(|| Error::Validation("Missing metadata MAC tag".to_string()))?;

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|e| Error::Validation(format!("Invalid metadata MAC nonce: {}", e)))?;
    if nonce_bytes.len() != 24 {
        return Err(Error::Validation(
            "Invalid metadata MAC nonce length".to_string(),
        ));
    }

    let tag_bytes = general_purpose::STANDARD
        .decode(tag_b64)
        .map_err(|e| Error::Validation(format!("Invalid metadata MAC tag: {}", e)))?;

    let payload = metadata_mac_payload(meta, vault_id)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&mac_key));
    let expected_tag = cipher
        .encrypt(
            XNonce::from_slice(&nonce_bytes),
            Payload {
                msg: b"",
                aad: &payload,
            },
        )
        .map_err(|e| Error::Encryption(format!("Metadata MAC failed: {}", e)))?;

    if expected_tag.len() != tag_bytes.len() || expected_tag.ct_eq(&tag_bytes).unwrap_u8() != 1 {
        return Err(Error::Validation(
            "Vault metadata integrity check failed.".to_string(),
        ));
    }

    Ok(())
}

pub fn get_vault_id(db_path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(db_path.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    format!("vault-{}", hex::encode(&result[..8]))
}
