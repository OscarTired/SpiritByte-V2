mod commands;
mod crypto;
mod generator;
mod state;
mod vault;

use state::AppState;
use tauri::Manager;
use vault::VaultPaths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let base = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&base).ok();
            app.manage(AppState::new(VaultPaths::new(base)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::vault_status,
            commands::create_vault,
            commands::unlock,
            commands::unlock_with_recovery,
            commands::lock,
            commands::change_master_password,
            commands::recover_and_reset,
            commands::list_entries,
            commands::list_folders,
            commands::get_vault,
            commands::upsert_entry,
            commands::delete_entry,
            commands::upsert_folder,
            commands::delete_folder,
            commands::generate_password,
            commands::password_strength,
            commands::get_settings,
            commands::save_settings,
            commands::save_wallpaper,
            commands::delete_wallpaper,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
