//! Configuration state for MacPak

use floem::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Maximum number of recent files to track
const MAX_RECENT_FILES: usize = 10;

use crate::gui::shared::Theme;

/// Persistable configuration (saved to disk)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedConfig {
    pub bg3_data_path: Option<String>,
    pub recent_files: Vec<String>,
    #[serde(default)]
    pub theme: Theme,
}

impl PersistedConfig {
    /// Get the config file path
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("MacPak").join("config.json"))
    }

    /// Load config from disk, or return default
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save config to disk
    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, content);
            }
        }
    }
}

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
    /// Recent files list (most recent first)
    pub recent_files: RwSignal<Vec<String>>,
    /// Current theme
    pub theme: RwSignal<Theme>,
}

impl ConfigState {
    pub fn new() -> Self {
        // Load persisted config
        let persisted = PersistedConfig::load();

        // Use persisted BG3 path if available, otherwise use default
        let expanded_path = persisted
            .bg3_data_path
            .map(|p| expand_tilde(&p))
            .unwrap_or_else(|| expand_tilde(DEFAULT_BG3_PATH));
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
            recent_files: RwSignal::new(persisted.recent_files),
            theme: RwSignal::new(persisted.theme),
        }
    }

    /// Add a file to the recent files list
    pub fn add_recent_file(&self, path: &str) {
        let mut files = self.recent_files.get();

        // Remove if already in list (we'll re-add at front)
        files.retain(|p| p != path);

        // Add to front
        files.insert(0, path.to_string());

        // Truncate to max
        files.truncate(MAX_RECENT_FILES);

        self.recent_files.set(files);
        self.save();
    }

    /// Clear all recent files
    pub fn clear_recent_files(&self) {
        self.recent_files.set(Vec::new());
        self.save();
    }

    /// Save current state to disk
    pub fn save(&self) {
        let persisted = PersistedConfig {
            bg3_data_path: Some(self.bg3_data_path.get()),
            recent_files: self.recent_files.get(),
            theme: self.theme.get(),
        };
        persisted.save();
    }

    /// Set the theme and save
    pub fn set_theme(&self, theme: Theme) {
        self.theme.set(theme);
        self.save();
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
