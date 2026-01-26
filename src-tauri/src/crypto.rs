use crate::file_dialog::pick_save_file;
use crate::types::{PasswordItem, ExportPayload, PubKeyExportPayload};
use crate::error::{Error, Result};
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use rand::RngCore;
use argon2::{Argon2, Algorithm, Params, Version};
use chacha20poly1305::{aead::{Aead, KeyInit}, XChaCha20Poly1305, Key, XNonce};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519Public, EphemeralSecret as X25519Secret, StaticSecret};
use zeroize::{Zeroize, Zeroizing};
use tauri::Window;
use std::fs;
use std::io::Write;
use std::path::Path;

#[tauri::command]
pub async fn export_password_entry(
    window: Window,
    password_item: PasswordItem,
    passphrase: String,
) -> Result<String> {
    let path_str = pick_save_file(window).await?;
    let path = std::path::PathBuf::from(&path_str);
    let plaintext = serde_json::to_vec(&password_item)?;

    let passphrase = Zeroizing::new(passphrase);

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| Error::Internal(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| Error::Internal(format!("KDF failed: {}", e)))?;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let version: u8 = 2;
    let salt_b64 = general_purpose::STANDARD.encode(salt);
    let nonce_b64 = general_purpose::STANDARD.encode(nonce);
    let aad = format!("v{version}:{salt_b64}:{nonce_b64}");

    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            chacha20poly1305::aead::Payload {
                msg: plaintext.as_ref(),
                aad: aad.as_bytes(),
            },
        )
        .map_err(|e| Error::Encryption(format!("encryption failed: {e}")))?;

    key.zeroize();

    let export = ExportPayload {
        version,
        salt_b64,
        nonce_b64,
        ciphertext_b64: general_purpose::STANDARD.encode(&ciphertext),
    };

    let export_bytes = serde_json::to_vec_pretty(&export)?;
    write_sensitive_bytes(&path, &export_bytes).await?;

    Ok(format!("Exported (passphrase) to {}", path.display()))
}

#[tauri::command]
pub async fn generate_x25519_keypair() -> Result<(String, String)> {
    let sk = StaticSecret::random_from_rng(OsRng);
    let pk = X25519Public::from(&sk);
    let sk_b64 = general_purpose::STANDARD.encode(sk.to_bytes());
    let pk_b64 = general_purpose::STANDARD.encode(pk.as_bytes());
    Ok((pk_b64, sk_b64))
}

#[tauri::command]
pub async fn export_password_entry_to_public_key(
    window: Window,
    password_item: PasswordItem,
    recipient_pubkey_b64: String,
) -> Result<String> {
    let path_str = pick_save_file(window).await?;
    let path = std::path::PathBuf::from(&path_str);


    let recip_pk_bytes = general_purpose::STANDARD
        .decode(recipient_pubkey_b64)
        .map_err(|_| Error::Validation("invalid recipient public key b64".to_string()))?;
    if recip_pk_bytes.len() != 32 {
        return Err(Error::Validation("recipient public key must be 32 bytes".into()));
    }

    let mut recip_pk_array = [0u8; 32];
    recip_pk_array.copy_from_slice(&recip_pk_bytes);
    let recip_pk = X25519Public::from(recip_pk_array);

    let eph_sk = X25519Secret::random_from_rng(OsRng);
    let eph_pk = X25519Public::from(&eph_sk);
    let shared = eph_sk.diffie_hellman(&recip_pk);

    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    let hk = Hkdf::<Sha256>::new(Some(&salt), shared.as_bytes());
    let mut aead_key = [0u8; 32];
    hk.expand(b"pulsar:password-export:x25519", &mut aead_key)
        .map_err(|_| Error::Internal("HKDF expand failed".to_string()))?;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&aead_key));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let plaintext = serde_json::to_vec(&password_item)?;

    let recipient_pub_b64 = general_purpose::STANDARD.encode(recip_pk.as_bytes());
    let eph_pub_b64 = general_purpose::STANDARD.encode(eph_pk.as_bytes());
    let salt_b64 = general_purpose::STANDARD.encode(&salt);
    let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
    let aad = format!(
        "v1:x25519-ephemeral-static:hkdf-sha256:xchacha20poly1305:{}:{}:{}:{}",
        recipient_pub_b64, eph_pub_b64, salt_b64, nonce_b64
    );

    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            chacha20poly1305::aead::Payload {
                msg: plaintext.as_ref(),
                aad: aad.as_bytes(),
            },
        )
        .map_err(|e| Error::Encryption(format!("encryption failed: {}", e)))?;

    aead_key.zeroize();

    let payload = PubKeyExportPayload {
        version: 1,
        scheme: "x25519-ephemeral-static".into(),
        kdf: "hkdf-sha256".into(),
        enc: "xchacha20poly1305".into(),
        recipient_pub_b64,
        eph_pub_b64,
        salt_b64,
        nonce_b64,
        ciphertext_b64: general_purpose::STANDARD.encode(&ciphertext),
    };

    let bytes = serde_json::to_vec_pretty(&payload)?;
    write_sensitive_bytes(&path, &bytes).await?;

    Ok(format!("Exported (recipient pubkey) to {}", path.display()))
}

#[tauri::command]
pub async fn import_password_entry_with_private_key(
    payload_json: String,
    recipient_secret_b64: String,
) -> Result<PasswordItem> {
    let payload: PubKeyExportPayload =
        serde_json::from_str(&payload_json)?;

    if payload.scheme != "x25519-ephemeral-static"
        || payload.kdf != "hkdf-sha256"
        || payload.enc != "xchacha20poly1305"
    {
        return Err(Error::Validation("unsupported payload parameters".into()));
    }

    let sk_bytes = general_purpose::STANDARD
        .decode(recipient_secret_b64)
        .map_err(|_| Error::Validation("invalid secret key b64".to_string()))?;
    if sk_bytes.len() != 32 {
        return Err(Error::Validation("secret key must be 32 bytes".into()));
    }

    let mut sk_array = [0u8; 32];
    sk_array.copy_from_slice(&sk_bytes);
    let sk = StaticSecret::from(sk_array);

    let eph_pk_bytes = general_purpose::STANDARD
        .decode(&payload.eph_pub_b64)
        .map_err(|_| Error::Validation("invalid eph_pub_b64".to_string()))?;
    if eph_pk_bytes.len() != 32 {
        return Err(Error::Validation("eph pubkey must be 32 bytes".into()));
    }

    let mut eph_pk_array = [0u8; 32];
    eph_pk_array.copy_from_slice(&eph_pk_bytes);
    let eph_pk = X25519Public::from(eph_pk_array);

    let salt = general_purpose::STANDARD
        .decode(&payload.salt_b64)
        .map_err(|_| Error::Validation("invalid salt b64".to_string()))?;
    let nonce = general_purpose::STANDARD
        .decode(&payload.nonce_b64)
        .map_err(|_| Error::Validation("invalid nonce b64".to_string()))?;
    if nonce.len() != 24 {
        return Err(Error::Validation("nonce must be 24 bytes".into()));
    }
    let ciphertext = general_purpose::STANDARD
        .decode(&payload.ciphertext_b64)
        .map_err(|_| Error::Validation("invalid ciphertext b64".to_string()))?;

    let shared = sk.diffie_hellman(&eph_pk);
    let hk = Hkdf::<Sha256>::new(Some(&salt), shared.as_bytes());
    let mut aead_key = [0u8; 32];
    hk.expand(b"pulsar:password-export:x25519", &mut aead_key)
        .map_err(|_| Error::Internal("HKDF expand failed".to_string()))?;

    let aad = format!(
        "v1:x25519-ephemeral-static:hkdf-sha256:xchacha20poly1305:{}:{}:{}:{}",
        payload.recipient_pub_b64,
        payload.eph_pub_b64,
        payload.salt_b64,
        payload.nonce_b64
    );

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&aead_key));
    let plaintext = cipher
        .decrypt(
            XNonce::from_slice(&nonce),
            chacha20poly1305::aead::Payload {
                msg: &ciphertext,
                aad: aad.as_bytes(),
            },
        )
        .map_err(|e| Error::Decryption(format!("decryption failed: {}", e)))?;
    aead_key.zeroize();

    let item: PasswordItem = serde_json::from_slice(&plaintext)?;
    Ok(item)
}

async fn write_sensitive_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp_path = path.with_extension("tmp");
    if tokio::fs::try_exists(&tmp_path).await.unwrap_or(false) {
        let _ = tokio::fs::remove_file(&tmp_path).await;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    #[cfg(not(unix))]
    {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    tokio::fs::rename(&tmp_path, path).await?;
    Ok(())
}

