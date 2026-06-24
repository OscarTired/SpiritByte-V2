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
    let session = guard.as_ref().ok_or_else(|| "vault is locked".to_string())?;
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
    let session = guard.as_mut().ok_or_else(|| "vault is locked".to_string())?;
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
    let result = with_session(&state, |s| {
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
    })?;
    persist(&state)?;
    Ok(result)
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: String) -> CmdResult<()> {
    with_session(&state, |s| {
        s.data.entries.retain(|e| e.id != id);
        Ok(())
    })?;
    persist(&state)
}

#[tauri::command]
pub fn upsert_folder(state: State<AppState>, folder: Folder) -> CmdResult<Folder> {
    let result = with_session(&state, |s| {
        match s.data.folders.iter_mut().find(|f| f.id == folder.id) {
            Some(existing) => *existing = folder.clone(),
            None => s.data.folders.push(folder.clone()),
        }
        Ok(folder.clone())
    })?;
    persist(&state)?;
    Ok(result)
}

#[tauri::command]
pub fn delete_folder(state: State<AppState>, id: String) -> CmdResult<()> {
    with_session(&state, |s| {
        s.data.folders.retain(|f| f.id != id);
        // Orphaned entries fall back to "no folder".
        for entry in s.data.entries.iter_mut() {
            if entry.folder_id.as_deref() == Some(id.as_str()) {
                entry.folder_id = None;
            }
        }
        Ok(())
    })?;
    persist(&state)
}

fn persist(state: &State<AppState>) -> CmdResult<()> {
    let guard = state.session.lock().unwrap();
    let session = guard.as_ref().ok_or_else(|| "vault is locked".to_string())?;
    vault::write_data(&state.paths, session).map_err(map)
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
