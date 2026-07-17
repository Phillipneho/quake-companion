// Quake Companion entry point.
//
// With the default `tauri-runtime` feature this boots the Tauri app. Without
// it, the crate compiles its core (hid/device/stats) only — useful for hosts
// that lack the webkit2gtk system libraries and for type-checking in CI.

#![cfg_attr(not(feature = "tauri-runtime"), allow(dead_code))]

#[cfg(feature = "tauri-runtime")]
#[tauri::main]
async fn main() {
    if let Err(e) = quake_companion::app::run() {
        eprintln!("Quake Companion error: {e}");
    }
}

#[cfg(not(feature = "tauri-runtime"))]
fn main() {
    eprintln!("quake-companion: built core-only (tauri-runtime feature disabled).");
    eprintln!("Rebuild with default features to launch the desktop app.");
}