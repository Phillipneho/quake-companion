fn main() {
    // Only run tauri-build codegen when the Tauri runtime feature is enabled.
    #[cfg(feature = "tauri-runtime")]
    tauri_build::build()
}