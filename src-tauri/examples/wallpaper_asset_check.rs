//! Native regression check. Run with isolated XDG directories, never real app data.
//! `--old-scope` reproduces the former Linux dot-directory failure.
use serde::Deserialize;
use std::io::Write;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "linux")]
#[path = "../src/linux_webview.rs"]
mod linux_webview;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ImageResult {
    path: String,
    loaded: bool,
    width: u32,
    height: u32,
}

// Keep the actual production page idle without opening a real vault.
#[tauri::command]
fn vault_status() -> serde_json::Value {
    serde_json::json!({"exists": false, "unlocked": false})
}

#[tauri::command]
fn get_settings() -> serde_json::Value {
    serde_json::json!({"showSplash": false})
}

#[tauri::command]
fn wallpaper_asset_results(results: Vec<ImageResult>) {
    let passed = results.len() == 5
        && results
            .iter()
            .all(|r| r.loaded && r.width > 0 && r.height > 0);
    for result in &results {
        println!(
            "decoder: {} loaded={} dimensions={}x{}",
            result.path, result.loaded, result.width, result.height
        );
    }
    let _ = std::io::stdout().flush();
    // This executable is a disposable test; the launcher cleans up its process group.
    std::process::exit(if passed { 0 } else { 1 });
}

fn main() {
    #[cfg(target_os = "linux")]
    linux_webview::prepare_environment();
    let data_home = std::env::var("XDG_DATA_HOME").expect("Set an isolated XDG_DATA_HOME");
    assert!(
        data_home.starts_with("/tmp/spiritbyte-asset-check-"),
        "Use a disposable test directory"
    );
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    if std::env::args().any(|arg| arg == "--old-scope") {
        context.config_mut().app.security.asset_protocol.scope =
            tauri::utils::config::FsScope::AllowedPaths(vec!["**".into()]);
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![wallpaper_asset_results, vault_status, get_settings])
        .setup(|app| {
            let base = app.path().app_data_dir()?;
            assert!(
                base.starts_with(std::env::var("XDG_DATA_HOME")?),
                "Runtime ignored the isolated XDG path; refusing to write test fixtures"
            );
            std::fs::create_dir_all(&base)?;
            let fixtures: [(&str, &[u8]); 5] = [
                ("png", include_bytes!("fixtures/wallpaper.png")),
                ("jpg", include_bytes!("fixtures/wallpaper.jpg")),
                ("gif", include_bytes!("fixtures/wallpaper.gif")),
                ("webp", include_bytes!("fixtures/wallpaper.webp")),
                ("bmp", include_bytes!("fixtures/wallpaper.bmp")),
            ];
            let mut paths = Vec::new();
            for (ext, bytes) in fixtures {
                let path = base.join(format!("wallpaper.asset-check.{ext}"));
                std::fs::write(&path, bytes)?;
                println!("scope: {} allowed={}", path.display(), app.asset_protocol_scope().is_allowed(&path));
                paths.push(path);
            }
            for name in ["vault.dat", "settings.json"] {
                let private = base.join(name);
                std::fs::write(&private, b"disposable scope probe")?;
                assert!(!app.asset_protocol_scope().is_allowed(&private), "Private app data must not be exposed");
            }
            let paths_json = serde_json::to_string(&paths)?;
            let script = format!(r#"
                addEventListener('DOMContentLoaded', async () => {{
                    const paths = {paths_json};
                    const results = await Promise.all(paths.map(path => new Promise(resolve => {{
                        const image = new Image();
                        const timer = setTimeout(() => finish(false), 8000);
                        function finish(loaded) {{
                            clearTimeout(timer);
                            image.onload = image.onerror = null;
                            resolve({{path, loaded, width:image.naturalWidth, height:image.naturalHeight}});
                            image.removeAttribute('src');
                        }}
                        image.onload = () => finish(true);
                        image.onerror = () => finish(false);
                        image.src = window.__TAURI_INTERNALS__.convertFileSrc(path, 'asset');
                    }})));
                    await window.__TAURI_INTERNALS__.invoke('wallpaper_asset_results', {{results}});
                }}, {{once:true}});
            "#);
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .visible(false)
                .initialization_script(script)
                .build()?;
            #[cfg(target_os = "linux")]
            window.with_webview(|webview| linux_webview::configure(&webview.inner()))?;
            Ok(())
        })
        .run(context)
        .expect("native asset check failed");
}
