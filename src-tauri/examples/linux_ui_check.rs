//! Exercises the production UI in WebKit with 1,000 disposable credentials and
//! a real animated GIF. No AppState, crypto commands or real vault are available.
#[cfg(target_os = "linux")]
#[path = "../src/linux_webview.rs"]
mod linux_webview;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
fn vault_status() -> serde_json::Value {
    serde_json::json!({"exists": true, "unlocked": true})
}

#[tauri::command]
fn get_vault() -> serde_json::Value {
    let entries: Vec<_> = (0..1000)
        .map(|i| {
            serde_json::json!({
                "id": format!("demo-{i}"), "title": format!("Fixture {i:04}"),
                "username": "example", "password": "demo-only", "url": "example.invalid",
                "notes": "Disposable UI test", "folderId": null, "iconId": null,
                "favorite": false, "createdAt": 1, "updatedAt": 1
            })
        })
        .collect();
    serde_json::json!({"entries": entries, "folders": []})
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> serde_json::Value {
    serde_json::json!({"showSplash": false, "lowPowerMode": false,
        "scanlines": true, "glow": true, "flicker": true, "panelOpacity": 0.85,
        "background": {"type": "image", "value": app.path().app_data_dir().unwrap().join("wallpaper.ui.gif")}})
}

#[tauri::command]
fn ui_check_results(results: serde_json::Value) {
    println!("ui-check: {results}");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let passed = ["gifLoaded", "background", "scanlines", "flicker", "search"]
        .iter()
        .all(|key| results[key] == true);
    std::process::exit(if passed { 0 } else { 1 });
}

fn main() {
    let data_home = std::env::var("XDG_DATA_HOME").expect("Use isolated XDG directories");
    assert!(data_home.starts_with("/tmp/spiritbyte-ui-check-"));
    #[cfg(target_os = "linux")]
    linux_webview::prepare_environment();
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![vault_status, get_vault, get_settings, ui_check_results])
        .setup(move |app| {
            let base = app.path().app_data_dir()?;
            assert!(base.starts_with(&data_home));
            std::fs::create_dir_all(&base)?;
            let path = base.join("wallpaper.ui.gif");
            std::fs::write(&path, include_bytes!("fixtures/wallpaper.animated.gif"))?;
            let path_json = serde_json::to_string(&path)?;
            let script = format!(r#"
                addEventListener('DOMContentLoaded', async () => {{
                  try {{
                    const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
                    const image = new Image();
                    image.crossOrigin = 'anonymous';
                    image.style.cssText = 'position:fixed;top:0;left:0;width:16px;height:16px;z-index:10000';
                    image.src = window.__TAURI_INTERNALS__.convertFileSrc({path_json}, 'asset');
                    document.body.append(image);
                    // Canvas drawImage intentionally reads a GIF's first frame.
                    // The launcher checks changing pixels in compositor screenshots.
                    for (let i = 0; i < 50; i++) {{
                        await wait(50);
                    }}
                    const gifLoaded = image.complete && image.naturalWidth > 0;
                    image.remove();
                    for (let i = 0; i < 100 && !document.querySelector('input[placeholder]'); i++) await wait(50);
                    const input = document.querySelector('input[placeholder]');
                    const started = performance.now();
                    if (input) {{
                        Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(input, 'Fixture 0999');
                        input.dispatchEvent(new Event('input',{{bubbles:true}}));
                    }}
                    let search = false;
                    for (let i = 0; i < 100; i++) {{
                        search = Boolean(input) && document.body.innerText.includes('Fixture 0999') && !document.body.innerText.includes('Fixture 0000');
                        if (search) break;
                        await wait(16);
                    }}
                    const results = {{
                        gifLoaded,
                        background: getComputedStyle(document.body).backgroundImage.includes('wallpaper.ui.gif'),
                        scanlines: document.documentElement.classList.contains('fx-scanlines'),
                        flicker: getComputedStyle(document.getElementById('root')).animationName === 'flicker',
                        search,
                        searchElapsedMs: Math.round(performance.now() - started),
                    }};
                    await window.__TAURI_INTERNALS__.invoke('ui_check_results',{{results}});
                  }} catch(error) {{
                    await window.__TAURI_INTERNALS__.invoke('ui_check_results',{{results:{{error:String(error)}}}});
                  }}
                }},{{once:true}});
            "#);
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("SpiritByte — disposable UI check")
                .inner_size(1100.0, 720.0)
                .initialization_script(script)
                .build()?;
            #[cfg(target_os = "linux")]
            window.with_webview(|webview| linux_webview::configure(&webview.inner()))?;
            Ok(())
        })
        .run(context)
        .expect("native UI check failed");
}
