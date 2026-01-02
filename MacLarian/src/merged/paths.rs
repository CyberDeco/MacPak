//! Path helpers for BG3 data locations

use std::path::{Path, PathBuf};

/// Default BG3 data path on macOS
pub const BG3_DATA_PATH_MACOS: &str =
    "~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/Baldur's Gate 3.app/Contents/Data";

/// Get the expanded BG3 data path (resolves ~)
pub fn bg3_data_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(BG3_DATA_PATH_MACOS.replace('~', &home)))
}

/// Get the path to VirtualTextures.pak
pub fn virtual_textures_pak_path() -> Option<PathBuf> {
    bg3_data_path().map(|p| p.join("VirtualTextures.pak"))
}

/// Replace home directory with ~ in a path string
pub fn path_with_tilde(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if let Ok(home) = std::env::var("HOME") {
        path_str.replace(&home, "~")
    } else {
        path_str.to_string()
    }
}

/// Expand ~ to home directory in a path string
pub fn expand_tilde(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        path.replace('~', &home)
    } else {
        path.to_string()
    }
}
