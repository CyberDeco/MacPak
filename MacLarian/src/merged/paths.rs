//! Path helpers for BG3 data locations

use std::path::{Path, PathBuf};

/// Default BG3 data path on macOS
pub const BG3_DATA_PATH_MACOS: &str = "~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/Baldur's Gate 3.app/Contents/Data";

/// Default BG3 data path on Linux
pub const BG3_DATA_PATH_LINUX: &str =
    "~/.steam/steam/steamapps/common/Baldurs Gate 3/Data";

/// Get the expanded BG3 data path for the current platform (resolves ~)
#[must_use]
pub fn bg3_data_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from(BG3_DATA_PATH_MACOS.replace('~', &home)))
    }

    #[cfg(target_os = "linux")]
    {
        Some(PathBuf::from(BG3_DATA_PATH_LINUX.replace('~', &home)))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Get the path to VirtualTextures.pak
#[must_use]
pub fn virtual_textures_pak_path() -> Option<PathBuf> {
    bg3_data_path().map(|p| p.join("VirtualTextures.pak"))
}

/// Replace home directory with ~ in a path string
#[must_use]
pub fn path_with_tilde(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if let Ok(home) = std::env::var("HOME") {
        path_str.replace(&home, "~")
    } else {
        path_str.to_string()
    }
}

/// Expand ~ to home directory in a path string
#[must_use]
pub fn expand_tilde(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        path.replace('~', &home)
    } else {
        path.to_string()
    }
}
