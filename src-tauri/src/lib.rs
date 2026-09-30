mod backup;
mod commands;
mod generator;
mod state;
mod vault;
mod wallpaper;

use state::AppState;
use tauri::Manager;
use vault::VaultPaths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let base = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&base).ok();
            app.manage(AppState::new(VaultPaths::new(base)));
            #[cfg(target_os = "linux")]
            if let Some(window) = app.get_webview_window("main") {
                window.with_webview(|webview| {
                    use webkit2gtk::{SettingsExt, WebContextExt, WebViewExt};
                    let view = webview.inner();
                    // A single-page utility needs no browsing-history page cache.
                    if let Some(context) = view.context() {
                        context.set_cache_model(webkit2gtk::CacheModel::DocumentViewer);
                    }
                    if let Some(settings) = WebViewExt::settings(&view) {
                        settings.set_enable_page_cache(false);
                    }
                })?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::export_backup,
            commands::import_backup,
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
            commands::pick_wallpaper,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
