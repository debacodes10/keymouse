#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "macos")]
pub use mac::*;

#[cfg(not(target_os = "macos"))]
compile_error!("Keymouse currently supports only macOS. Add platform implementations in src/platforms/ for your target OS.");
