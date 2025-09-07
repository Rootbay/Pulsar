use crate::file_dialog::pick_save_file;
use crate::types::{PasswordItem, ExportPayload, PubKeyExportPayload};
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use rand::RngCore;
use argon2::{Argon2, Algorithm, Params, Version};
use chacha20poly1305::{aead::{Aead, KeyInit}, XChaCha20Poly1305, Key, XNonce};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519Public, EphemeralSecret as X25519Secret, StaticSecret};
use zeroize::Zeroize;
use tauri::Window;

/// (Argon2id + XChaCha20-Poly1305)
#[tauri::command]
pub async fn export_password_entry(
    window: Window,
    password_item: PasswordItem,
    passphrase: String,
) -> Result<String, String> {
    let path = pick_save_file(window).await?;
    let plaintext = serde_json::to_vec(&password_item).map_err(|e| e.to_string())?;

    // KDF
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| format!("KDF failed: {}", e))?;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let version: u8 = 2;
    let salt_b64 = general_purpose::STANDARD.encode(&salt);
    let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
    let aad = format!("v{}:{}:{}", version, salt_b64, nonce_b64);

    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            chacha20poly1305::aead::Payload {
                msg: plaintext.as_ref(),
                aad: aad.as_bytes(),
            },
        )
        .map_err(|e| format!("encryption failed: {}", e))?;

    key.zeroize();

    let export = ExportPayload {
        version,
        salt_b64,
        nonce_b64,
        ciphertext_b64: general_purpose::STANDARD.encode(&ciphertext),
    };

    let export_bytes = serde_json::to_vec_pretty(&export).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, export_bytes).await.map_err(|e| e.to_string())?;

    Ok(format!("Exported (passphrase) to {}", path))
}

/// Generate an X25519 keypair (return as base64)
#[tauri::command]
pub async fn generate_x25519_keypair() -> Result<(String, String), String> {
    let sk = StaticSecret::random_from_rng(OsRng);
    let pk = X25519Public::from(&sk);
    let sk_b64 = general_purpose::STANDARD.encode(sk.to_bytes());
    let pk_b64 = general_purpose::STANDARD.encode(pk.as_bytes());
    Ok((pk_b64, sk_b64))
}

/// Export a single password to a recipient's public key.
#[tauri::command]
pub async fn export_password_entry_to_public_key(
    window: Window,
    password_item: PasswordItem,
    recipient_pubkey_b64: String,
) -> Result<String, String> {
    let path = pick_save_file(window).await?;

    // Parse recipient pubkey
    let recip_pk_bytes = general_purpose::STANDARD
        .decode(recipient_pubkey_b64)
        .map_err(|_| "invalid recipient public key b64")?;
    if recip_pk_bytes.len() != 32 {
        return Err("recipient public key must be 32 bytes".into());
    }

    let mut recip_pk_array = [0u8; 32];
    recip_pk_array.copy_from_slice(&recip_pk_bytes);
    let recip_pk = X25519Public::from(recip_pk_array);

    // Ephemeral keypair
    let eph_sk = X25519Secret::random_from_rng(OsRng);
    let eph_pk = X25519Public::from(&eph_sk);

    // DH shared secret
    let shared = eph_sk.diffie_hellman(&recip_pk);

    // Derive symmetric key
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    let hk = Hkdf::<Sha256>::new(Some(&salt), shared.as_bytes());
    let mut aead_key = [0u8; 32];
    hk.expand(b"pulsar:password-export:x25519", &mut aead_key)
        .map_err(|_| "HKDF expand failed")?;

    // AEAD encrypt
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&aead_key));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    let plaintext = serde_json::to_vec(&password_item).map_err(|e| e.to_string())?;

    // Metadata integrity
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
        .map_err(|e| format!("encryption failed: {}", e))?;

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

    let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, bytes).await.map_err(|e| e.to_string())?;

    Ok(format!("Exported (recipient pubkey) to {}", path.to_string()))
}

/// Decrypt a public-key export (recipient side) given the recipient's secret key.
#[tauri::command]
pub async fn import_password_entry_with_private_key(
    payload_json: String,
    recipient_secret_b64: String,
) -> Result<PasswordItem, String> {
    let payload: PubKeyExportPayload =
        serde_json::from_str(&payload_json).map_err(|e| format!("invalid payload: {}", e))?;

    if payload.scheme != "x25519-ephemeral-static"
        || payload.kdf != "hkdf-sha256"
        || payload.enc != "xchacha20poly1305"
    {
        return Err("unsupported payload parameters".into());
    }

    let sk_bytes = general_purpose::STANDARD
        .decode(recipient_secret_b64)
        .map_err(|_| "invalid secret key b64")?;
    if sk_bytes.len() != 32 {
        return Err("secret key must be 32 bytes".into());
    }

    let mut sk_array = [0u8; 32];
    sk_array.copy_from_slice(&sk_bytes);
    let sk = StaticSecret::from(sk_array);

    let eph_pk_bytes = general_purpose::STANDARD
        .decode(&payload.eph_pub_b64)
        .map_err(|_| "invalid eph_pub_b64")?;
    if eph_pk_bytes.len() != 32 {
        return Err("eph pubkey must be 32 bytes".into());
    }

    let mut eph_pk_array = [0u8; 32];
    eph_pk_array.copy_from_slice(&eph_pk_bytes);
    let eph_pk = X25519Public::from(eph_pk_array);

    let salt = general_purpose::STANDARD
        .decode(&payload.salt_b64)
        .map_err(|_| "invalid salt b64")?;
    let nonce = general_purpose::STANDARD
        .decode(&payload.nonce_b64)
        .map_err(|_| "invalid nonce b64")?;
    let ciphertext = general_purpose::STANDARD
        .decode(&payload.ciphertext_b64)
        .map_err(|_| "invalid ciphertext b64")?;

    // DH and HKDF
    let shared = sk.diffie_hellman(&eph_pk);
    let hk = Hkdf::<Sha256>::new(Some(&salt), shared.as_bytes());
    let mut aead_key = [0u8; 32];
    hk.expand(b"pulsar:password-export:x25519", &mut aead_key)
        .map_err(|_| "HKDF expand failed")?;

    // Reconstruct AAD from payload to verify integrity
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
        .map_err(|e| format!("decryption failed: {}", e))?;
    aead_key.zeroize();

    let item: PasswordItem =
        serde_json::from_slice(&plaintext).map_err(|e| format!("invalid inner JSON: {}", e))?;
    Ok(item)
}
