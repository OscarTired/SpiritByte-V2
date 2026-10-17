use serde_json::{json, Value};
use spiritbyte_core::vault::{self, Session, VaultData, VaultPaths};
use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
        })
        .join("com.spiritbyte.app")
}

pub fn presets() -> Value {
    serde_json::from_str(include_str!("../assets/palettes.json")).unwrap()
}

pub fn settings(paths: &VaultPaths) -> Value {
    let mut defaults = json!({"paletteId":"amber-crt", "palette":presets()[0]["palette"],
        "font":"mixed", "fontSize":18, "language":"es", "scanlines":true, "glow":true,
        "flicker":false, "lowPowerMode":true, "ditherIntensity":2,
        "background":{"type":"solid", "value":""}, "panelOpacity":1.0,
        "autoLockMinutes":5, "clipboardClearSeconds":20, "showSplash":true, "showUnfiled":true});
    if let Ok(Value::Object(values)) = std::fs::read(&paths.settings)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .ok_or(())
    {
        for (key, value) in values {
            if key == "palette" || key == "background" {
                if let (Some(target), Some(source)) =
                    (defaults[&key].as_object_mut(), value.as_object())
                {
                    target.extend(source.clone());
                }
            } else {
                defaults[&key] = value;
            }
        }
    }
    defaults
}

pub fn save_settings(paths: &VaultPaths, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    vault::atomic_write(&paths.settings, &bytes).map_err(|e| e.to_string())
}

/// Persist the candidate before publishing it to the UI. A failed write leaves
/// the in-memory session and the previous encrypted file intact.
pub fn commit(paths: &VaultPaths, session: &mut Session, data: VaultData) -> Result<(), String> {
    let candidate = Session {
        dek: session.dek.clone(),
        data,
    };
    vault::write_data(paths, &candidate).map_err(|e| e.to_string())?;
    session.data = candidate.data;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spiritbyte_core::{
        backup,
        vault::{Entry, Folder},
    };

    #[test]
    fn compatible_vault_backup_recovery_and_failed_save() {
        let dir = tempfile::tempdir().unwrap();
        let paths = VaultPaths::new(dir.path().to_path_buf());
        let (mut session, phrase) = vault::create_vault(&paths, "native test password").unwrap();
        let mut data = session.data.clone();
        data.folders.push(Folder {
            id: "folder".into(),
            name: "Work".into(),
            icon: None,
            color: None,
            parent_id: None,
        });
        data.entries.push(Entry {
            id: "entry".into(),
            title: "Example".into(),
            username: "user".into(),
            password: "secret".into(),
            url: "https://example.org".into(),
            notes: "note".into(),
            folder_id: Some("folder".into()),
            icon_id: None,
            favorite: true,
            created_at: 1,
            updated_at: 2,
        });
        commit(&paths, &mut session, data).unwrap();
        drop(session);
        let mut reopened = vault::unlock(&paths, "native test password").unwrap();
        assert_eq!(reopened.data.entries[0].password, "secret");
        assert!(vault::unlock(&paths, "wrong").is_err());
        let encrypted = backup::export(&reopened.data, "backup test password").unwrap();
        let restored = backup::decrypt(&encrypted, "backup test password").unwrap();
        let merged = backup::merge(&reopened.data, restored);
        assert_eq!(merged.entries.len(), 2);
        backup::validate(&merged).unwrap();
        let broken = VaultPaths::new(dir.path().join("not-directory"));
        std::fs::write(dir.path().join("not-directory"), "blocked").unwrap();
        assert!(commit(&broken, &mut reopened, merged).is_err());
        assert_eq!(reopened.data.entries.len(), 1);
        vault::recover_and_reset(&paths, &phrase, "replacement password").unwrap();
        assert!(vault::unlock(&paths, "native test password").is_err());
        assert_eq!(
            vault::unlock(&paths, "replacement password")
                .unwrap()
                .data
                .entries[0]
                .title,
            "Example"
        );
    }

    #[test]
    fn settings_preserve_unknown_fields_and_partial_palette() {
        let dir = tempfile::tempdir().unwrap();
        let paths = VaultPaths::new(dir.path().to_path_buf());
        save_settings(
            &paths,
            &json!({"futureOption":42,"palette":{"primary":"1 2 3"}}),
        )
        .unwrap();
        let value = settings(&paths);
        assert_eq!(value["futureOption"], 42);
        assert_eq!(value["palette"]["primary"], "1 2 3");
        assert_eq!(value["palette"]["bg"], "13 12 10");
    }
}
