//! Configuration state for MacPak

use floem::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Default BG3 data path on macOS (Steam installation)
#[cfg(target_os = "macos")]
pub const DEFAULT_BG3_PATH: &str = "~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/Baldur's Gate 3.app/Contents/Data";

/// Default BG3 data path on other platforms (empty, user must configure)
#[cfg(not(target_os = "macos"))]
pub const DEFAULT_BG3_PATH: &str = "";

/// Expand ~ to the user's home directory
pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}

/// Check if a path exists and is a directory
pub fn path_exists(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    Path::new(path).is_dir()
}

/// Configuration state
#[derive(Clone)]
pub struct ConfigState {
    /// Path to BG3 Data directory
    pub bg3_data_path: RwSignal<String>,
    /// Whether the config dialog is visible
    pub show_dialog: RwSignal<bool>,
    /// Whether the app is ready (used to trigger menu initialization) - thread-safe
    pub app_ready: Arc<AtomicBool>,
    /// Warning message for invalid path
    pub path_warning: RwSignal<Option<String>>,
}

impl ConfigState {
    pub fn new() -> Self {
        let expanded_path = expand_tilde(DEFAULT_BG3_PATH);
        let path_valid = path_exists(&expanded_path);

        let warning = if !expanded_path.is_empty() && !path_valid {
            Some("BG3 installation not found at default path. Please configure manually.".to_string())
        } else {
            None
        };

        Self {
            bg3_data_path: RwSignal::new(expanded_path),
            show_dialog: RwSignal::new(false),
            app_ready: Arc::new(AtomicBool::new(false)),
            path_warning: RwSignal::new(warning),
        }
    }

    /// Mark the app as ready (thread-safe)
    pub fn set_ready(&self) {
        self.app_ready.store(true, Ordering::SeqCst);
    }

    /// Check if app is ready (thread-safe)
    pub fn is_ready(&self) -> bool {
        self.app_ready.load(Ordering::SeqCst)
    }

    /// Get the expanded BG3 data path
    pub fn get_bg3_path(&self) -> String {
        self.bg3_data_path.get()
    }

    /// Validate the current path and update warning
    pub fn validate_path(&self) {
        let path = self.bg3_data_path.get();
        if path.is_empty() {
            self.path_warning.set(Some("No BG3 path configured.".to_string()));
        } else if !path_exists(&path) {
            self.path_warning.set(Some("Path does not exist.".to_string()));
        } else {
            self.path_warning.set(None);
        }
    }
}

impl Default for ConfigState {
    fn default() -> Self {
        Self::new()
    }
}
