//! Rendering settings for a single-window desktop utility. Keep CSS effects,
//! animated images and canvas working without forcing permanent compositing.
use webkit2gtk::{SettingsExt, WebContextExt, WebView, WebViewExt};

pub fn prepare_environment() {
    // This small UI does not need a browser's optimizing JavaScript compiler.
    // Avoid its executable pages/compiler caches; crypto stays native Rust.
    // Respect an explicit override, e.g. JSC_useJIT=true for large vaults.
    #[cfg(not(debug_assertions))]
    if std::env::var_os("JSC_useJIT").is_none() {
        std::env::set_var("JSC_useJIT", "false");
    }
}

pub fn configure(view: &WebView) {
    if let Some(context) = view.context() {
        context.set_cache_model(webkit2gtk::CacheModel::DocumentViewer);
    }
    if let Some(settings) = WebViewExt::settings(view) {
        settings.set_enable_page_cache(false);
        settings.set_hardware_acceleration_policy(webkit2gtk::HardwareAccelerationPolicy::OnDemand);
        // SpiritByte uses CSS, Canvas2D and image decoders, never these APIs.
        settings.set_enable_webgl(false);
        settings.set_enable_webaudio(false);
        settings.set_enable_media(false);
        settings.set_enable_media_stream(false);
        settings.set_enable_mediasource(false);
    }
}
