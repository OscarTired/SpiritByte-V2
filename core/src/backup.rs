//! Portable backups: fixed-cost Argon2id + authenticated encryption, no plaintext files.
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use zeroize::Zeroizing;

use crate::{
    crypto,
    vault::{VaultData, VaultError},
};

pub const MAX_BACKUP_BYTES: usize = 32 * 1024 * 1024;
const FORMAT: &str = "spiritbyte-encrypted-vault";

pub fn save_new(path: &std::path::Path, contents: &str) -> Result<(), VaultError> {
    use std::io::Write;
    let parent = path.parent().ok_or_else(invalid)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(contents.as_bytes())?;
    file.as_file().sync_all()?;
    // Never destroy an existing backup (or vault) selected by mistake.
    file.persist_noclobber(path)
        .map_err(|e| VaultError::Io(e.error.to_string()))?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Backup {
    format: String,
    version: u32,
    salt: String,
    ciphertext: String,
}

fn invalid() -> VaultError {
    VaultError::Serde("invalid or unsupported backup".into())
}

/// An explicit allow-list. Folder IDs retain containers only: they must never
/// implicitly add entries that the user unchecked in the selection UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportSelection {
    pub entry_ids: Vec<String>,
    pub folder_ids: Vec<String>,
}

pub fn select(data: &VaultData, selection: Option<&ExportSelection>) -> Result<VaultData, VaultError> {
    validate(data)?;
    let Some(selection) = selection else { return Ok(data.clone()) };
    if selection.entry_ids.is_empty() && selection.folder_ids.is_empty() {
        return Err(VaultError::Serde("select at least one entry or folder".into()));
    }
    let entries: HashSet<_> = selection.entry_ids.iter().map(String::as_str).collect();
    let mut folders: HashSet<_> = selection.folder_ids.iter().map(String::as_str).collect();
    let known_entries: HashSet<_> = data.entries.iter().map(|e| e.id.as_str()).collect();
    let parents: HashMap<_, _> = data.folders.iter().map(|f| (f.id.as_str(), f.parent_id.as_deref())).collect();
    if entries.iter().any(|id| !known_entries.contains(id)) || folders.iter().any(|id| !parents.contains_key(id)) {
        return Err(VaultError::Serde("selection no longer matches the vault; select again".into()));
    }
    let selected_entries: Vec<_> = data.entries.iter().filter(|e| entries.contains(e.id.as_str())).cloned().collect();
    folders.extend(selected_entries.iter().filter_map(|e| e.folder_id.as_deref()));
    let mut pending: Vec<_> = folders.iter().copied().collect();
    while let Some(id) = pending.pop() {
        if let Some(parent) = parents[id] {
            if folders.insert(parent) { pending.push(parent); }
        }
    }
    let selected_folders = data.folders.iter().filter(|f| folders.contains(f.id.as_str())).cloned().collect();
    let result = VaultData { entries: selected_entries, folders: selected_folders };
    validate(&result)?;
    Ok(result)
}

pub fn export(data: &VaultData, password: &str) -> Result<String, VaultError> {
    if password.chars().count() < 12 {
        return Err(VaultError::Serde(
            "backup password must contain at least 12 characters".into(),
        ));
    }
    validate(data)?;
    let salt = crypto::random_bytes(crypto::SALT_LEN);
    let key = crypto::derive_key(password.as_bytes(), &salt, &crypto::ArgonParams::default())?;
    let plain = Zeroizing::new(serde_json::to_vec(data)?);
    let ciphertext = B64.encode(crypto::encrypt(&key, &plain)?);
    let result = serde_json::to_string(&Backup {
        format: FORMAT.into(),
        version: 1,
        salt: B64.encode(salt),
        ciphertext,
    })?;
    if result.len() > MAX_BACKUP_BYTES {
        return Err(invalid());
    }
    Ok(result)
}

pub fn decrypt(contents: &str, password: &str) -> Result<VaultData, VaultError> {
    if contents.len() > MAX_BACKUP_BYTES {
        return Err(invalid());
    }
    let backup: Backup = serde_json::from_str(contents).map_err(|_| invalid())?;
    if backup.format != FORMAT || backup.version != 1 {
        return Err(invalid());
    }
    let salt = B64.decode(&backup.salt).map_err(|_| invalid())?;
    if salt.len() != crypto::SALT_LEN {
        return Err(invalid());
    }
    let ciphertext = B64.decode(&backup.ciphertext).map_err(|_| invalid())?;
    // The file cannot supply attacker-controlled memory/iteration costs.
    let key = crypto::derive_key(password.as_bytes(), &salt, &crypto::ArgonParams::default())?;
    let plain = Zeroizing::new(crypto::decrypt(&key, &ciphertext)?);
    let data: VaultData = serde_json::from_slice(&plain).map_err(|_| invalid())?;
    validate(&data)?;
    Ok(data)
}

pub fn validate(data: &VaultData) -> Result<(), VaultError> {
    let folders: HashMap<_, _> = data
        .folders
        .iter()
        .map(|f| (f.id.as_str(), f.parent_id.as_deref()))
        .collect();
    let entries: HashSet<_> = data.entries.iter().map(|e| e.id.as_str()).collect();
    if folders.len() != data.folders.len()
        || entries.len() != data.entries.len()
        || folders.contains_key("")
        || entries.contains("")
    {
        return Err(invalid());
    }
    // Iterative, linear traversal rejects missing parents and cycles without recursion.
    let mut done = HashSet::new();
    for id in folders.keys() {
        let mut visiting = HashSet::new();
        let mut next = Some(*id);
        while let Some(current) = next {
            if done.contains(current) {
                break;
            }
            if !visiting.insert(current) {
                return Err(invalid());
            }
            next = *folders.get(current).ok_or_else(invalid)?;
        }
        done.extend(visiting);
    }
    if data.entries.iter().any(|e| {
        e.folder_id
            .as_deref()
            .is_some_and(|id| !folders.contains_key(id))
    }) {
        return Err(invalid());
    }
    Ok(())
}

/// Fresh IDs preserve every record and all relationships without overwriting existing data.
pub fn merge(current: &VaultData, mut imported: VaultData) -> VaultData {
    let ids: HashMap<_, _> = imported
        .folders
        .iter()
        .map(|f| (f.id.clone(), uuid::Uuid::new_v4().to_string()))
        .collect();
    for folder in &mut imported.folders {
        folder.id = ids[&folder.id].clone();
        folder.parent_id = folder.parent_id.as_ref().map(|id| ids[id].clone());
    }
    for entry in &mut imported.entries {
        entry.id = uuid::Uuid::new_v4().to_string();
        entry.folder_id = entry.folder_id.as_ref().map(|id| ids[id].clone());
    }
    let mut merged = current.clone();
    merged.entries.extend(imported.entries);
    merged.folders.extend(imported.folders);
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::{self, Entry, Folder, Session, VaultPaths};

    fn fixture() -> VaultData {
        VaultData {
            entries: vec![Entry {
                id: "entry".into(),
                title: "Private title".into(),
                username: "user".into(),
                password: "secret-value".into(),
                url: "https://example.com".into(),
                notes: "línea uno\n二 🔒".into(),
                folder_id: Some("child".into()),
                icon_id: Some("key".into()),
                favorite: true,
                created_at: 123,
                updated_at: 456,
            }],
            folders: vec![
                Folder {
                    id: "parent".into(),
                    name: "Root".into(),
                    icon: Some("folder".into()),
                    color: Some("#ff6b00".into()),
                    parent_id: None,
                },
                Folder {
                    id: "child".into(),
                    name: "Nested".into(),
                    icon: None,
                    color: None,
                    parent_id: Some("parent".into()),
                },
            ],
        }
    }

    #[test]
    fn roundtrip_tampering_and_merge() {
        let original = fixture();
        let password = "a long backup password";
        let encrypted = export(&original, password).unwrap();
        assert!(!encrypted.contains("secret-value"));
        assert!(!encrypted.contains("Private title"));
        let restored = decrypt(&encrypted, password).unwrap();
        assert_eq!(
            serde_json::to_value(&original).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        assert!(decrypt(&encrypted, "wrong password").is_err());
        let mut corrupted: Backup = serde_json::from_str(&encrypted).unwrap();
        let mut bytes = B64.decode(&corrupted.ciphertext).unwrap();
        bytes[30] ^= 1;
        corrupted.ciphertext = B64.encode(bytes);
        assert!(decrypt(&serde_json::to_string(&corrupted).unwrap(), password).is_err());
        let merged = merge(&original, restored);
        assert_eq!(merged.entries.len(), 2);
        assert_eq!(merged.folders.len(), 4);
        assert_ne!(merged.entries[0].id, merged.entries[1].id);
        assert_eq!(
            merged.entries[1].folder_id.as_ref(),
            Some(&merged.folders[3].id)
        );
        assert_eq!(
            merged.folders[3].parent_id.as_ref(),
            Some(&merged.folders[2].id)
        );
        validate(&merged).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("backup.spiritbyte");
        save_new(&backup_path, &encrypted).unwrap();
        assert!(save_new(&backup_path, "replacement").is_err());
        assert_eq!(std::fs::read_to_string(backup_path).unwrap(), encrypted);
        let paths = VaultPaths::new(dir.path().into());
        let session = Session {
            dek: Zeroizing::new([5; crypto::KEY_LEN]),
            data: merged,
        };
        vault::write_data(&paths, &session).unwrap();
        vault::write_data(&paths, &session).unwrap(); // replaces an existing file on Windows
        let plain = crypto::decrypt(&session.dek, &std::fs::read(&paths.data).unwrap()).unwrap();
        let persisted: VaultData = serde_json::from_slice(&plain).unwrap();
        assert_eq!(persisted.entries.len(), 2);
    }

    #[test]
    fn rejects_invalid_structure_and_format() {
        let mut data = fixture();
        data.folders[0].parent_id = Some("child".into());
        assert!(validate(&data).is_err());
        data = fixture();
        data.entries[0].folder_id = Some("missing".into());
        assert!(validate(&data).is_err());
        data = fixture();
        data.folders.push(data.folders[0].clone());
        assert!(validate(&data).is_err());
        assert!(decrypt("{}", "password").is_err());
        assert!(decrypt(&"x".repeat(MAX_BACKUP_BYTES + 1), "password").is_err());
        assert!(export(&fixture(), "short").is_err());
        assert!(decrypt(
            r#"{"format":"spiritbyte-encrypted-vault","version":999,"salt":"","ciphertext":""}"#,
            "password"
        )
        .is_err());
    }
}
