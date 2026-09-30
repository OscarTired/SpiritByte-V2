//! Cryptographic primitives for SpiritByte.
//!
//! Security model (envelope encryption):
//! - A random 256-bit Data Encryption Key (DEK) encrypts the vault contents.
//! - The DEK is wrapped (encrypted) twice, independently:
//!     * with a KEK derived from the master password (Argon2id)
//!     * with a KEK derived from a 12-word BIP39 recovery phrase (Argon2id)
//! - Either credential can unwrap the DEK; the recovery phrase can also reset
//!   the master password by re-wrapping the DEK.
//!
//! The DEK and master password never leave the Rust process and are zeroized
//! from memory when dropped.

use argon2::{Algorithm, Argon2, Params, Version};
use bip39::Mnemonic;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("key derivation failed: {0}")]
    Argon(String),
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed (wrong credentials or corrupted data)")]
    Decrypt,
    #[error("invalid data length")]
    Length,
    #[error("invalid recovery phrase: {0}")]
    Mnemonic(String),
}

/// Argon2id cost parameters. Stored alongside the vault so future tuning stays
/// backwards compatible.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ArgonParams {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl Default for ArgonParams {
    fn default() -> Self {
        // ~64 MiB, 3 iterations, single lane: solid for interactive desktop unlock.
        Self {
            m_cost: 65536,
            t_cost: 3,
            p_cost: 1,
        }
    }
}

/// Returns `n` cryptographically secure random bytes.
pub fn random_bytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    OsRng.fill_bytes(&mut buf);
    buf
}

/// Derives a 256-bit key from a secret + salt using Argon2id.
pub fn derive_key(
    secret: &[u8],
    salt: &[u8],
    params: &ArgonParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    let p = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_LEN))
        .map_err(|e| CryptoError::Argon(e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, p);
    let mut out = Zeroizing::new([0u8; KEY_LEN]);
    argon
        .hash_password_into(secret, salt, out.as_mut())
        .map_err(|e| CryptoError::Argon(e.to_string()))?;
    Ok(out)
}

/// Encrypts `plaintext` with XChaCha20-Poly1305. Output = nonce(24) || ciphertext.
pub fn encrypt(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(key).map_err(|_| CryptoError::Encrypt)?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::Encrypt)?;
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Decrypts data produced by [`encrypt`].
pub fn decrypt(key: &[u8; KEY_LEN], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < NONCE_LEN {
        return Err(CryptoError::Length);
    }
    let (nonce_bytes, ct) = data.split_at(NONCE_LEN);
    let cipher =
        XChaCha20Poly1305::new_from_slice(key).map_err(|_| CryptoError::Decrypt)?;
    let nonce = XNonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ct).map_err(|_| CryptoError::Decrypt)
}

/// Generates a fresh 12-word (128-bit) BIP39 mnemonic recovery phrase.
pub fn generate_mnemonic() -> Result<String, CryptoError> {
    let mut entropy = [0u8; 16];
    OsRng.fill_bytes(&mut entropy);
    let mnemonic =
        Mnemonic::from_entropy(&entropy).map_err(|e| CryptoError::Mnemonic(e.to_string()))?;
    Ok(mnemonic.to_string())
}

/// Validates and normalizes a recovery phrase (trims, lowercases, single spaces).
pub fn normalize_mnemonic(phrase: &str) -> Result<String, CryptoError> {
    let cleaned = phrase
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let mnemonic = Mnemonic::parse_normalized(&cleaned)
        .map_err(|e| CryptoError::Mnemonic(e.to_string()))?;
    Ok(mnemonic.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [7u8; KEY_LEN];
        let msg = b"super secret vault payload";
        let ct = encrypt(&key, msg).unwrap();
        assert_ne!(&ct[NONCE_LEN..], &msg[..]);
        let pt = decrypt(&key, &ct).unwrap();
        assert_eq!(pt, msg);
    }

    #[test]
    fn decrypt_wrong_key_fails() {
        let key = [1u8; KEY_LEN];
        let bad = [2u8; KEY_LEN];
        let ct = encrypt(&key, b"hello").unwrap();
        assert!(decrypt(&bad, &ct).is_err());
    }

    #[test]
    fn derive_key_is_deterministic() {
        let p = ArgonParams::default();
        let salt = [9u8; SALT_LEN];
        let a = derive_key(b"password123", &salt, &p).unwrap();
        let b = derive_key(b"password123", &salt, &p).unwrap();
        assert_eq!(a.as_ref(), b.as_ref());
        let c = derive_key(b"different", &salt, &p).unwrap();
        assert_ne!(a.as_ref(), c.as_ref());
    }

    #[test]
    fn mnemonic_generation_and_normalization() {
        let phrase = generate_mnemonic().unwrap();
        assert_eq!(phrase.split_whitespace().count(), 12);
        let normalized = normalize_mnemonic(&format!("  {}  ", phrase.to_uppercase())).unwrap();
        assert_eq!(normalized, phrase);
        assert!(normalize_mnemonic("not a valid phrase at all nope nope").is_err());
    }
}
