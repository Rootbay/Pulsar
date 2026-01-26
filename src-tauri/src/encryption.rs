use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, Key, XNonce,
};
use rand::{rngs::OsRng, RngCore};
use base64::{engine::general_purpose, Engine as _};
use crate::error::{Error, Result};

const KEY_LEN_BYTES: usize = 32;

fn ensure_key_len(key: &[u8], err: fn(String) -> Error) -> Result<()> {
    if key.len() != KEY_LEN_BYTES {
        return Err(err("Invalid key length".to_string()));
    }
    Ok(())
}

pub fn encrypt(plaintext: &str, key: &[u8]) -> Result<String> {
    ensure_key_len(key, Error::Encryption)?;
    let key: &Key = Key::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key);

    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| Error::Encryption(format!("Encryption failed: {e}")))?;

    let nonce_b64 = general_purpose::STANDARD.encode(nonce_bytes);
    let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);

    Ok(format!("{nonce_b64}:{ciphertext_b64}"))
}

pub fn encrypt_bytes(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    ensure_key_len(key, Error::Encryption)?;
    let key: &Key = Key::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key);

    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| Error::Encryption(format!("Encryption failed: {e}")))?;

    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_bytes(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    ensure_key_len(key, Error::Decryption)?;
    if encrypted_data.len() < 24 {
        return Err(Error::Decryption("Invalid encrypted data: too short to contain nonce".to_string()));
    }

    let (nonce_bytes, ciphertext) = encrypted_data.split_at(24);
    let nonce = XNonce::from_slice(nonce_bytes);

    let key: &Key = Key::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| Error::Decryption(format!("Decryption failed: {e}")))?;

    Ok(decrypted_bytes)
}

pub fn decrypt(encrypted_payload: &str, key: &[u8]) -> Result<String> {
    ensure_key_len(key, Error::Decryption)?;
    let mut parts = encrypted_payload.split(':');
    let nonce_b64 = parts.next().ok_or_else(|| Error::Decryption("Invalid encrypted payload format: missing nonce".to_string()))?;
    let ciphertext_b64 = parts.next().ok_or_else(|| Error::Decryption("Invalid encrypted payload format: missing ciphertext".to_string()))?;

    if parts.next().is_some() {
        return Err(Error::Decryption("Invalid encrypted payload format: too many parts".to_string()));
    }

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|e| Error::Decryption(format!("Nonce decode failed: {e}")))?;
    
    if nonce_bytes.len() != 24 {
        return Err(Error::Decryption("Invalid nonce length".to_string()));
    }

    let ciphertext = general_purpose::STANDARD
        .decode(ciphertext_b64)
        .map_err(|e| Error::Decryption(format!("Ciphertext decode failed: {e}")))?;

    let key: &Key = Key::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| Error::Decryption(format!("Decryption failed: {e}")))?;

    String::from_utf8(decrypted_bytes)
        .map_err(|e| Error::Decryption(format!("UTF-8 conversion failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = b"an-example-key-that-is-32-bytes!";
        let plaintext = "this is a secret message";
        let encrypted = encrypt(plaintext, key).unwrap();
        let decrypted = decrypt(&encrypted, key).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_decrypt_invalid_format() {
        let key = b"an-example-key-that-is-32-bytes!";
        assert!(decrypt("invalid", key).is_err());
        assert!(decrypt("a:b:c", key).is_err());
    }

    #[test]
    fn test_decrypt_bad_base64() {
        let key = b"an-example-key-that-is-32-bytes!";
        assert!(decrypt("@@@:valid_b64", key).is_err());
        assert!(decrypt("dmFsaWRfYjY0:@@@", key).is_err());
    }

    #[test]
    fn test_invalid_key_length_errors() {
        let short_key = b"short-key";
        assert!(encrypt("hello", short_key).is_err());
        assert!(encrypt_bytes(b"hello", short_key).is_err());
        assert!(decrypt("a:b", short_key).is_err());
        assert!(decrypt_bytes(&[0u8; 24], short_key).is_err());
    }
}
