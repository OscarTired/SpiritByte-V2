//! Vault data model, on-disk format and high-level vault operations.

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::crypto::{self, ArgonParams, CryptoError, KEY_LEN, SALT_LEN};

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serde(String),
    #[error("a vault already exists")]
    AlreadyExists,
    #[error("no vault found")]
    NotFound,
    #[allow(dead_code)]
    #[error("vault is locked")]
    Locked,
    #[error("base64 decode error")]
    Base64,
}

impl From<std::io::Error> for VaultError {
    fn from(e: std::io::Error) -> Self {
        VaultError::Io(e.to_string())
    }
}
impl From<serde_json::Error> for VaultError {
    fn from(e: serde_json::Error) -> Self {
        VaultError::Serde(e.to_string())
    }
}

pub const VAULT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub icon_id: Option<String>,
    #[serde(default)]
    pub favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct VaultData {
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub folders: Vec<Folder>,
}

/// Cleartext metadata stored next to the encrypted vault. Contains no secrets:
/// only salts, Argon2 params and the DEK wrapped under each credential.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VaultMeta {
    pub version: u32,
    pub argon: ArgonParams,
    pub password_salt: String,
    pub recovery_salt: String,
    pub wrapped_dek_password: String,
    pub wrapped_dek_recovery: String,
}

/// Paths to the three on-disk artifacts.
#[derive(Clone, Debug)]
pub struct VaultPaths {
    pub meta: PathBuf,
    pub data: PathBuf,
    pub settings: PathBuf,
}

impl VaultPaths {
    pub fn new(base: PathBuf) -> Self {
        Self {
            meta: base.join("vault.meta.json"),
            data: base.join("vault.dat"),
            settings: base.join("settings.json"),
        }
    }
}

/// An unlocked, in-memory vault session. Holds the DEK (zeroized on drop) and
/// the decrypted data.
pub struct Session {
    pub dek: Zeroizing<[u8; KEY_LEN]>,
    pub data: VaultData,
}

pub fn vault_exists(paths: &VaultPaths) -> bool {
    paths.meta.exists() && paths.data.exists()
}

fn load_meta(paths: &VaultPaths) -> Result<VaultMeta, VaultError> {
    let bytes = std::fs::read(&paths.meta)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn save_meta(paths: &VaultPaths, meta: &VaultMeta) -> Result<(), VaultError> {
    if let Some(parent) = paths.meta.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(meta)?;
    atomic_write(&paths.meta, &json)?;
    Ok(())
}

fn b64_decode(s: &str) -> Result<Vec<u8>, VaultError> {
    B64.decode(s).map_err(|_| VaultError::Base64)
}

/// Derives the password KEK and unwraps the DEK.
fn unwrap_with(
    secret: &[u8],
    salt_b64: &str,
    wrapped_b64: &str,
    params: &ArgonParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>, VaultError> {
    let salt = b64_decode(salt_b64)?;
    let kek = crypto::derive_key(secret, &salt, params)?;
    let wrapped = b64_decode(wrapped_b64)?;
    let dek_bytes = Zeroizing::new(crypto::decrypt(&kek, &wrapped)?);
    if dek_bytes.len() != KEY_LEN {
        return Err(VaultError::Crypto(CryptoError::Length));
    }
    let mut dek = Zeroizing::new([0u8; KEY_LEN]);
    dek.copy_from_slice(&dek_bytes);
    Ok(dek)
}

fn wrap_dek(
    secret: &[u8],
    salt: &[u8],
    dek: &[u8; KEY_LEN],
    params: &ArgonParams,
) -> Result<String, VaultError> {
    let kek = crypto::derive_key(secret, salt, params)?;
    let wrapped = crypto::encrypt(&kek, dek)?;
    Ok(B64.encode(wrapped))
}

fn read_data(paths: &VaultPaths, dek: &[u8; KEY_LEN]) -> Result<VaultData, VaultError> {
    let blob = std::fs::read(&paths.data)?;
    let plain = Zeroizing::new(crypto::decrypt(dek, &blob)?);
    Ok(serde_json::from_slice(&plain)?)
}

/// Encrypts and persists the current vault data.
pub fn write_data(paths: &VaultPaths, session: &Session) -> Result<(), VaultError> {
    if let Some(parent) = paths.data.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let plain = Zeroizing::new(serde_json::to_vec(&session.data)?);
    let blob = crypto::encrypt(&session.dek, &plain)?;
    atomic_write(&paths.data, &blob)?;
    Ok(())
}

/// Creates a brand new vault, returning the live session and the 12-word
/// recovery phrase (shown once to the user).
pub fn create_vault(paths: &VaultPaths, password: &str) -> Result<(Session, String), VaultError> {
    if vault_exists(paths) {
        return Err(VaultError::AlreadyExists);
    }
    let params = ArgonParams::default();
    let password_salt = crypto::random_bytes(SALT_LEN);
    let recovery_salt = crypto::random_bytes(SALT_LEN);

    let dek_vec = crypto::random_bytes(KEY_LEN);
    let mut dek = Zeroizing::new([0u8; KEY_LEN]);
    dek.copy_from_slice(&dek_vec);

    let mnemonic = crypto::generate_mnemonic()?;

    let wrapped_dek_password = wrap_dek(password.as_bytes(), &password_salt, &dek, &params)?;
    let wrapped_dek_recovery = wrap_dek(mnemonic.as_bytes(), &recovery_salt, &dek, &params)?;

    let meta = VaultMeta {
        version: VAULT_VERSION,
        argon: params,
        password_salt: B64.encode(&password_salt),
        recovery_salt: B64.encode(&recovery_salt),
        wrapped_dek_password,
        wrapped_dek_recovery,
    };

    let session = Session {
        dek,
        data: VaultData::default(),
    };
    save_meta(paths, &meta)?;
    write_data(paths, &session)?;

    Ok((session, mnemonic))
}

/// Unlocks an existing vault with the master password.
pub fn unlock(paths: &VaultPaths, password: &str) -> Result<Session, VaultError> {
    if !vault_exists(paths) {
        return Err(VaultError::NotFound);
    }
    let meta = load_meta(paths)?;
    let dek = unwrap_with(
        password.as_bytes(),
        &meta.password_salt,
        &meta.wrapped_dek_password,
        &meta.argon,
    )?;
    let data = read_data(paths, &dek)?;
    Ok(Session { dek, data })
}

/// Unlocks an existing vault with the 12-word recovery phrase.
pub fn unlock_with_recovery(paths: &VaultPaths, phrase: &str) -> Result<Session, VaultError> {
    if !vault_exists(paths) {
        return Err(VaultError::NotFound);
    }
    let normalized = crypto::normalize_mnemonic(phrase)?;
    let meta = load_meta(paths)?;
    let dek = unwrap_with(
        normalized.as_bytes(),
        &meta.recovery_salt,
        &meta.wrapped_dek_recovery,
        &meta.argon,
    )?;
    let data = read_data(paths, &dek)?;
    Ok(Session { dek, data })
}

/// Re-wraps the DEK under a new master password. Requires the current DEK
/// (i.e. an unlocked session).
pub fn change_password(
    paths: &VaultPaths,
    dek: &[u8; KEY_LEN],
    new_password: &str,
) -> Result<(), VaultError> {
    let mut meta = load_meta(paths)?;
    let new_salt = crypto::random_bytes(SALT_LEN);
    meta.wrapped_dek_password = wrap_dek(new_password.as_bytes(), &new_salt, dek, &meta.argon)?;
    meta.password_salt = B64.encode(&new_salt);
    save_meta(paths, &meta)?;
    Ok(())
}

/// Recovers via the phrase and resets the master password in one operation.
pub fn recover_and_reset(
    paths: &VaultPaths,
    phrase: &str,
    new_password: &str,
) -> Result<Session, VaultError> {
    let session = unlock_with_recovery(paths, phrase)?;
    change_password(paths, &session.dek, new_password)?;
    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> (VaultPaths, std::path::PathBuf) {
        let mut dir = std::env::temp_dir();
        dir.push(format!("spiritbyte_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        (VaultPaths::new(dir.clone()), dir)
    }

    #[test]
    fn create_unlock_and_persist() {
        let (paths, dir) = temp_paths();
        let (mut session, phrase) = create_vault(&paths, "master-pw").unwrap();
        assert_eq!(phrase.split_whitespace().count(), 12);

        session.data.entries.push(Entry {
            id: "1".into(),
            title: "GitHub".into(),
            username: "oscar".into(),
            password: "hunter2".into(),
            url: "https://github.com".into(),
            notes: String::new(),
            folder_id: None,
            icon_id: None,
            favorite: false,
            created_at: 0,
            updated_at: 0,
        });
        write_data(&paths, &session).unwrap();

        let reopened = unlock(&paths, "master-pw").unwrap();
        assert_eq!(reopened.data.entries.len(), 1);
        assert_eq!(reopened.data.entries[0].password, "hunter2");

        assert!(unlock(&paths, "wrong-pw").is_err());

        // Recovery path unlocks the same data.
        let via_recovery = unlock_with_recovery(&paths, &phrase).unwrap();
        assert_eq!(via_recovery.data.entries[0].title, "GitHub");

        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn recovery_resets_password() {
        let (paths, dir) = temp_paths();
        let (_session, phrase) = create_vault(&paths, "old-pw").unwrap();

        recover_and_reset(&paths, &phrase, "new-pw").unwrap();
        assert!(unlock(&paths, "old-pw").is_err());
        assert!(unlock(&paths, "new-pw").is_ok());
        // Recovery phrase still works after reset.
        assert!(unlock_with_recovery(&paths, &phrase).is_ok());

        std::fs::remove_dir_all(dir).ok();
    }
}

/// Write and sync a sibling temporary file before atomically replacing the destination.
/// A failed write leaves the previous vault intact, including on Windows.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), VaultError> {
    let parent = path
        .parent()
        .ok_or_else(|| VaultError::Io("missing parent directory".into()))?;
    std::fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path)
        .map_err(|e| VaultError::Io(e.error.to_string()))?;
    Ok(())
}
