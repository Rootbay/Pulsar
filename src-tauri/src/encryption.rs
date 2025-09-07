use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, Key, XNonce,
};
use rand::{rngs::OsRng, RngCore};
use base64::{engine::general_purpose, Engine as _};

/// Format: `nonce_b64:ciphertext_b64`
pub fn encrypt(plaintext: &str, key: &[u8]) -> Result<String, String> {
    let key: &Key = Key::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key);

    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let nonce_b64 = general_purpose::STANDARD.encode(nonce_bytes);
    let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);

    Ok(format!("{}:{}", nonce_b64, ciphertext_b64))
}

/// Format: `nonce_b64:ciphertext_b64`
pub fn decrypt(encrypted_payload: &str, key: &[u8]) -> Result<String, String> {
    let mut parts = encrypted_payload.split(':');
    let nonce_b64 = parts.next().ok_or("Invalid encrypted payload format: missing nonce")?;
    let ciphertext_b64 = parts.next().ok_or("Invalid encrypted payload format: missing ciphertext")?;

    if parts.next().is_some() {
        return Err("Invalid encrypted payload format: too many parts".to_string());
    }

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|e| format!("Nonce decode failed: {}", e))?;
    
    if nonce_bytes.len() != 24 {
        return Err("Invalid nonce length".to_string());
    }

    let ciphertext = general_purpose::STANDARD
        .decode(ciphertext_b64)
        .map_err(|e| format!("Ciphertext decode failed: {}", e))?;

    let key: &Key = Key::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(decrypted_bytes)
        .map_err(|e| format!("UTF-8 conversion failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = b"an-example-key-that-is-32-bytes";
        let plaintext = "this is a secret message";
        let encrypted = encrypt(plaintext, key).unwrap();
        let decrypted = decrypt(&encrypted, key).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_decrypt_invalid_format() {
        let key = b"an-example-key-that-is-32-bytes";
        assert!(decrypt("invalid", key).is_err());
        assert!(decrypt("a:b:c", key).is_err());
    }

    #[test]
    fn test_decrypt_bad_base64() {
        let key = b"an-example-key-that-is-32-bytes";
        assert!(decrypt("@@@:valid_b64", key).is_err());
        assert!(decrypt("dmFsaWRfYjY0:@@@", key).is_err());
    }
}
