use aes_gcm::{
    Aes256Gcm, KeyInit,
    aead::{Aead, AeadCore, OsRng},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use sha2::{Digest, Sha256};

/// Derives a 32-byte AES-256 key from an arbitrary secret string via SHA-256.
#[must_use]
pub fn derive_key(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().into()
}

/// Encrypts `plaintext` with AES-256-GCM using a random 12-byte nonce.
///
/// Returns `(ciphertext_base64, nonce_base64)`.
///
/// # Errors
///
/// Returns an error if AES-GCM encryption fails.
pub fn encrypt(key: &[u8; 32], plaintext: &str) -> anyhow::Result<(String, String)> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| anyhow::anyhow!("AES-GCM encryption failed"))?;
    Ok((B64.encode(ciphertext), B64.encode(nonce)))
}

/// Decrypts `ciphertext_b64` using the given `nonce_b64` and AES-256-GCM key.
///
/// # Errors
///
/// Returns an error if base64 decoding or AES-GCM decryption fails.
pub fn decrypt(key: &[u8; 32], ciphertext_b64: &str, nonce_b64: &str) -> anyhow::Result<String> {
    let ciphertext = B64
        .decode(ciphertext_b64)
        .map_err(|e| anyhow::anyhow!("base64 decode ciphertext: {e}"))?;
    let nonce_bytes = B64
        .decode(nonce_b64)
        .map_err(|e| anyhow::anyhow!("base64 decode nonce: {e}"))?;
    anyhow::ensure!(nonce_bytes.len() == 12, "nonce must be exactly 12 bytes");
    #[allow(deprecated)]
    let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow::anyhow!("AES-GCM decryption failed"))?;
    String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("UTF-8 decode: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        derive_key("test-secret-key-for-unit-tests")
    }

    #[test]
    fn round_trip() {
        let key = test_key();
        let (ct, nonce) = encrypt(&key, "hello world").unwrap();
        let plain = decrypt(&key, &ct, &nonce).unwrap();
        assert_eq!(plain, "hello world");
    }

    #[test]
    fn empty_string_round_trip() {
        let key = test_key();
        let (ct, nonce) = encrypt(&key, "").unwrap();
        let plain = decrypt(&key, &ct, &nonce).unwrap();
        assert_eq!(plain, "");
    }

    #[test]
    fn different_nonces_for_same_plaintext() {
        let key = test_key();
        let (ct1, nonce1) = encrypt(&key, "same").unwrap();
        let (ct2, nonce2) = encrypt(&key, "same").unwrap();
        // nonces should differ (random), ciphertexts too
        assert_ne!(nonce1, nonce2);
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = test_key();
        let key2 = derive_key("different-key");
        let (ct, nonce) = encrypt(&key1, "secret").unwrap();
        assert!(decrypt(&key2, &ct, &nonce).is_err());
    }
}
