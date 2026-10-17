//! Register embedded fonts for this process only; do not install system fonts.
use std::{ffi::CString, os::unix::ffi::OsStrExt};

#[link(name = "fontconfig")]
extern "C" {
    fn FcConfigGetCurrent() -> *mut std::ffi::c_void;
    fn FcConfigAppFontAddFile(config: *mut std::ffi::c_void, file: *const u8) -> i32;
}

pub fn register() -> Result<tempfile::TempDir, String> {
    let dir = tempfile::Builder::new()
        .prefix("spiritbyte-fonts-")
        .tempdir()
        .map_err(|e| e.to_string())?;
    let files: [(&str, &[u8]); 7] = [
        (
            "VT323.ttf",
            include_bytes!("../assets/vt323-latin-400-normal.ttf"),
        ),
        (
            "PressStart2P.ttf",
            include_bytes!("../assets/press-start-2p-latin-400-normal.ttf"),
        ),
        (
            "GeistPixel-Square.ttf",
            include_bytes!("../assets/GeistPixel-Square.ttf"),
        ),
        (
            "GeistPixel-Grid.ttf",
            include_bytes!("../assets/GeistPixel-Grid.ttf"),
        ),
        (
            "GeistPixel-Circle.ttf",
            include_bytes!("../assets/GeistPixel-Circle.ttf"),
        ),
        (
            "GeistPixel-Triangle.ttf",
            include_bytes!("../assets/GeistPixel-Triangle.ttf"),
        ),
        (
            "GeistPixel-Line.ttf",
            include_bytes!("../assets/GeistPixel-Line.ttf"),
        ),
    ];
    for (name, bytes) in files {
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        let path = CString::new(path.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
        // Fontconfig consumes the path synchronously. The TempDir remains alive
        // for the application's lifetime; no global font configuration changes.
        if unsafe { FcConfigAppFontAddFile(FcConfigGetCurrent(), path.as_ptr().cast()) } == 0 {
            return Err(format!("Cannot load {name}"));
        }
    }
    Ok(dir)
}
