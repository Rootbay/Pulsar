use crate::error::{Error, Result};
use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    Key, XChaCha20Poly1305, XNonce,
};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;

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

pub fn decrypt(encrypted_payload: &str, key: &[u8]) -> Result<String> {
    Ok((*decrypt_zeroized(encrypted_payload, key)?).clone())
}

pub fn decrypt_zeroized(encrypted_payload: &str, key: &[u8]) -> Result<Zeroizing<String>> {
    ensure_key_len(key, Error::Decryption)?;
    let mut parts = encrypted_payload.split(':');
    let nonce_b64 = parts.next().ok_or_else(|| {
        Error::Decryption("Invalid encrypted payload format: missing nonce".to_string())
    })?;
    let ciphertext_b64 = parts.next().ok_or_else(|| {
        Error::Decryption("Invalid encrypted payload format: missing ciphertext".to_string())
    })?;

    if parts.next().is_some() {
        return Err(Error::Decryption(
            "Invalid encrypted payload format: too many parts".to_string(),
        ));
    }

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce_b64.trim())
        .map_err(|e| Error::Decryption(format!("Nonce decode failed: {e}")))?;

    if nonce_bytes.len() != 24 {
        return Err(Error::Decryption("Invalid nonce length".to_string()));
    }

    let ciphertext = general_purpose::STANDARD
        .decode(ciphertext_b64.trim())
        .map_err(|e| Error::Decryption(format!("Ciphertext decode failed: {e}")))?;

    let nonce = XNonce::from_slice(&nonce_bytes);

    let key: &Key = Key::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| Error::Decryption(format!("Decryption failed: {e}")))?;

    let s = String::from_utf8(decrypted_bytes)
        .map_err(|e| Error::Decryption(format!("UTF-8 conversion failed: {e}")))?;

    Ok(Zeroizing::new(s))
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
        return Err(Error::Decryption(
            "Invalid encrypted data: too short to contain nonce".to_string(),
        ));
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

pub struct CipherSession {
    cipher: XChaCha20Poly1305,
    key: Vec<u8>,
}

impl CipherSession {
    pub fn new(key: &[u8]) -> Result<Self> {
        ensure_key_len(key, Error::Encryption)?;
        let key_vec = key.to_vec();
        let key_slice: &Key = Key::from_slice(key);
        Ok(Self {
            cipher: XChaCha20Poly1305::new(key_slice),
            key: key_vec,
        })
    }

    pub fn generate_search_token(&self, text: &str) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let normalized = text.trim().to_lowercase();
        if normalized.is_empty() {
            return Vec::new();
        }

        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&self.key)
            .expect("HMAC can take key of any size");
        mac.update(normalized.as_bytes());
        let result = mac.finalize().into_bytes();
        // Truncate to 8 bytes for better performance in trigram search
        // with minimal collision risk for this specific use case.
        result[..8].to_vec()
    }

    pub fn generate_trigram_hashes(&self, text: &str) -> Vec<Vec<u8>> {
        let normalized = text.trim().to_lowercase();
        if normalized.len() < 3 {
            return if normalized.is_empty() {
                Vec::new()
            } else {
                vec![self.generate_search_token(&normalized)]
            };
        }

        use rayon::prelude::*;
        let chars: Vec<char> = normalized.chars().collect();
        (0..=chars.len() - 3)
            .into_par_iter()
            .map(|i| {
                let trigram: String = chars[i..i + 3].iter().collect();
                self.generate_search_token(&trigram)
            })
            .collect()
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| Error::Encryption(format!("Encryption failed: {e}")))?;

        let nonce_b64 = general_purpose::STANDARD.encode(nonce_bytes);
        let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);

        Ok(format!("{nonce_b64}:{ciphertext_b64}"))
    }

    pub fn decrypt(&self, encrypted_payload: &str) -> Result<String> {
        Ok((*self.decrypt_zeroized(encrypted_payload)?).clone())
    }

    pub fn decrypt_zeroized(&self, encrypted_payload: &str) -> Result<Zeroizing<String>> {
        let mut parts = encrypted_payload.split(':');
        let nonce_b64 = parts.next().ok_or_else(|| {
            Error::Decryption("Invalid encrypted payload format: missing nonce".to_string())
        })?;
        let ciphertext_b64 = parts.next().ok_or_else(|| {
            Error::Decryption("Invalid encrypted payload format: missing ciphertext".to_string())
        })?;

        if parts.next().is_some() {
            return Err(Error::Decryption(
                "Invalid encrypted payload format: too many parts".to_string(),
            ));
        }

        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce_b64.trim())
            .map_err(|e| Error::Decryption(format!("Nonce decode failed: {e}")))?;

        if nonce_bytes.len() != 24 {
            return Err(Error::Decryption("Invalid nonce length".to_string()));
        }

        let ciphertext = general_purpose::STANDARD
            .decode(ciphertext_b64.trim())
            .map_err(|e| Error::Decryption(format!("Ciphertext decode failed: {e}")))?;

        let nonce = XNonce::from_slice(&nonce_bytes);

        let decrypted_bytes = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| Error::Decryption(format!("Decryption failed: {e}")))?;

        let s = String::from_utf8(decrypted_bytes)
            .map_err(|e| Error::Decryption(format!("UTF-8 conversion failed: {e}")))?;

        Ok(Zeroizing::new(s))
    }
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
