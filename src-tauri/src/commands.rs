//! Tauri command surface bridging the React frontend and the Rust core.
//!
//! Secrets policy: the DEK and master password never cross this boundary.
//! The frontend only ever receives decrypted *entries* on demand.

use serde::Serialize;
use tauri::State;

use crate::generator::{self, GenOptions, Strength};
use crate::state::AppState;
use crate::vault::{self, Entry, Folder, Session, VaultData};

type CmdResult<T> = Result<T, String>;

fn map<E: ToString>(e: E) -> String {
    e.to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub exists: bool,
    pub unlocked: bool,
}

#[tauri::command]
pub fn vault_status(state: State<AppState>) -> VaultStatus {
    let unlocked = state.session.lock().unwrap().is_some();
    VaultStatus {
        exists: vault::vault_exists(&state.paths),
        unlocked,
    }
}

/// Creates a new vault. Returns the one-time 12-word recovery phrase.
#[tauri::command]
pub fn create_vault(state: State<AppState>, password: String) -> CmdResult<String> {
    let (session, phrase) = vault::create_vault(&state.paths, &password).map_err(map)?;
    *state.session.lock().unwrap() = Some(session);
    Ok(phrase)
}

#[tauri::command]
pub fn unlock(state: State<AppState>, password: String) -> CmdResult<()> {
    let session = vault::unlock(&state.paths, &password).map_err(map)?;
    *state.session.lock().unwrap() = Some(session);
    Ok(())
}

#[tauri::command]
pub fn unlock_with_recovery(state: State<AppState>, phrase: String) -> CmdResult<()> {
    let session = vault::unlock_with_recovery(&state.paths, &phrase).map_err(map)?;
    *state.session.lock().unwrap() = Some(session);
    Ok(())
}

#[tauri::command]
pub fn lock(state: State<AppState>) {
    *state.session.lock().unwrap() = None;
}

#[tauri::command]
pub fn change_master_password(state: State<AppState>, new_password: String) -> CmdResult<()> {
    let guard = state.session.lock().unwrap();
    let session = guard
        .as_ref()
        .ok_or_else(|| "vault is locked".to_string())?;
    vault::change_password(&state.paths, &session.dek, &new_password).map_err(map)
}

/// Recovers using the phrase and immediately sets a new master password.
#[tauri::command]
pub fn recover_and_reset(
    state: State<AppState>,
    phrase: String,
    new_password: String,
) -> CmdResult<()> {
    let session = vault::recover_and_reset(&state.paths, &phrase, &new_password).map_err(map)?;
    *state.session.lock().unwrap() = Some(session);
    Ok(())
}

fn with_session<T, F>(state: &State<AppState>, f: F) -> CmdResult<T>
where
    F: FnOnce(&mut Session) -> Result<T, String>,
{
    let mut guard = state.session.lock().unwrap();
    let session = guard
        .as_mut()
        .ok_or_else(|| "vault is locked".to_string())?;
    f(session)
}

#[tauri::command]
pub fn list_entries(state: State<AppState>) -> CmdResult<Vec<Entry>> {
    with_session(&state, |s| Ok(s.data.entries.clone()))
}

#[tauri::command]
pub fn list_folders(state: State<AppState>) -> CmdResult<Vec<Folder>> {
    with_session(&state, |s| Ok(s.data.folders.clone()))
}

#[tauri::command]
pub fn get_vault(state: State<AppState>) -> CmdResult<VaultData> {
    with_session(&state, |s| Ok(s.data.clone()))
}

/// Inserts or updates an entry, then persists the encrypted vault.
#[tauri::command]
pub fn upsert_entry(state: State<AppState>, entry: Entry) -> CmdResult<Entry> {
    with_persisted_session(&state, |s| {
        let stored = match s.data.entries.iter_mut().find(|e| e.id == entry.id) {
            Some(existing) => {
                *existing = entry.clone();
                existing.clone()
            }
            None => {
                s.data.entries.push(entry.clone());
                entry.clone()
            }
        };
        Ok(stored)
    })
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: String) -> CmdResult<()> {
    with_persisted_session(&state, |s| {
        s.data.entries.retain(|e| e.id != id);
        Ok(())
    })
}

#[tauri::command]
pub fn upsert_folder(state: State<AppState>, folder: Folder) -> CmdResult<Folder> {
    with_persisted_session(&state, |s| {
        match s.data.folders.iter_mut().find(|f| f.id == folder.id) {
            Some(existing) => *existing = folder.clone(),
            None => s.data.folders.push(folder.clone()),
        }
        Ok(folder.clone())
    })
}

#[tauri::command]
pub fn delete_folder(state: State<AppState>, id: String) -> CmdResult<()> {
    with_persisted_session(&state, |s| {
        s.data.folders.retain(|f| f.id != id);
        for folder in &mut s.data.folders {
            if folder.parent_id.as_deref() == Some(id.as_str()) {
                folder.parent_id = None;
            }
        }
        // Orphaned entries fall back to "no folder".
        for entry in s.data.entries.iter_mut() {
            if entry.folder_id.as_deref() == Some(id.as_str()) {
                entry.folder_id = None;
            }
        }
        Ok(())
    })
}

fn with_persisted_session<T>(
    state: &State<AppState>,
    f: impl FnOnce(&mut Session) -> CmdResult<T>,
) -> CmdResult<T> {
    with_session(state, |session| {
        let previous = session.data.clone();
        match f(session).and_then(|result| {
            vault::write_data(&state.paths, session).map_err(map)?;
            Ok(result)
        }) {
            Ok(result) => Ok(result),
            Err(error) => {
                session.data = previous;
                Err(error)
            }
        }
    })
}

// ----- Utilities -----

#[tauri::command]
pub fn generate_password(options: GenOptions) -> String {
    generator::generate(&options)
}

#[tauri::command]
pub fn password_strength(password: String) -> Strength {
    generator::strength(&password)
}

// ----- Wallpaper -----

#[tauri::command]
pub fn save_wallpaper(state: State<AppState>, data: Vec<u8>, ext: String) -> CmdResult<String> {
    let base = state.paths.settings.parent().unwrap();
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("wallpaper.") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    let filename = format!("wallpaper.{}", ext);
    let path = base.join(&filename);
    std::fs::write(&path, &data).map_err(map)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn delete_wallpaper(state: State<AppState>) -> CmdResult<()> {
    let base = state.paths.settings.parent().unwrap();
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("wallpaper.") {
                std::fs::remove_file(entry.path()).map_err(map)?;
            }
        }
    }
    Ok(())
}

// ----- Settings (cleartext UI preferences) -----

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> CmdResult<serde_json::Value> {
    let path = &state.paths.settings;
    if !path.exists() {
        return Ok(serde_json::Value::Null);
    }
    let bytes = std::fs::read(path).map_err(map)?;
    serde_json::from_slice(&bytes).map_err(map)
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: serde_json::Value) -> CmdResult<()> {
    let path = &state.paths.settings;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(map)?;
    }
    let json = serde_json::to_vec_pretty(&settings).map_err(map)?;
    std::fs::write(path, json).map_err(map)
}

// Password derivation and encryption run off the UI thread.
#[tauri::command]
pub async fn export_backup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    password: String,
    selection: Option<crate::backup::ExportSelection>,
) -> CmdResult<bool> {
    use tauri_plugin_dialog::DialogExt;
    let password = zeroize::Zeroizing::new(password);
    let data = with_session(&state, |s| Ok(s.data.clone()))?;
    tauri::async_runtime::spawn_blocking(move || {
        let data = crate::backup::select(&data, selection.as_ref()).map_err(map)?;
        let contents = crate::backup::export(&data, &password).map_err(map)?;
        let selected = app
            .dialog()
            .file()
            .add_filter("SpiritByte", &["spiritbyte"])
            .set_file_name(format!("SpiritByte-{}.spiritbyte", uuid::Uuid::new_v4()))
            .blocking_save_file();
        let Some(selected) = selected else {
            return Ok(false);
        };
        let path = selected.into_path().map_err(map)?;
        if path.extension().and_then(|s| s.to_str()) != Some("spiritbyte") {
            return Err("use the .spiritbyte extension".into());
        }
        crate::backup::save_new(&path, &contents).map_err(map)?;
        Ok(true)
    })
    .await
    .map_err(map)?
}

#[tauri::command]
pub async fn import_backup(
    state: State<'_, AppState>,
    contents: String,
    password: String,
) -> CmdResult<()> {
    with_session(&state, |_| Ok(()))?;
    let password = zeroize::Zeroizing::new(password);
    let imported = tauri::async_runtime::spawn_blocking(move || {
        crate::backup::decrypt(&contents, &password).map_err(map)
    })
    .await
    .map_err(map)??;
    with_session(&state, |s| {
        let candidate = Session {
            dek: s.dek.clone(),
            data: crate::backup::merge(&s.data, imported),
        };
        // Commit the in-memory state only after the encrypted atomic write succeeds.
        vault::write_data(&state.paths, &candidate).map_err(map)?;
        s.data = candidate.data;
        Ok(())
    })
}
