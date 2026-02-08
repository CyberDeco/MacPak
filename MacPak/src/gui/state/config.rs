//! Configuration state for MacPak

use floem::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Maximum number of recent files to track
const MAX_RECENT_FILES: usize = 10;

use crate::gui::shared::Theme;

// Default value functions for serde
fn default_window_width() -> f64 {
    1200.0
}
fn default_window_height() -> f64 {
    850.0
}
fn default_true() -> bool {
    true
}
fn default_file_list_width() -> f64 {
    600.0
}
fn default_browser_panel_width() -> f64 {
    400.0
}

/// Window geometry and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedWindowState {
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default = "default_window_width")]
    pub width: f64,
    #[serde(default = "default_window_height")]
    pub height: f64,
}

impl Default for PersistedWindowState {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: default_window_width(),
            height: default_window_height(),
        }
    }
}

/// Editor tab session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedEditorState {
    /// File paths of open tabs (will attempt to reopen)
    #[serde(default)]
    pub open_files: Vec<String>,
    /// Index of active tab
    #[serde(default)]
    pub active_tab_index: usize,
    /// Show line numbers preference
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
}

impl Default for PersistedEditorState {
    fn default() -> Self {
        Self {
            open_files: Vec::new(),
            active_tab_index: 0,
            show_line_numbers: true,
        }
    }
}

/// Browser tab session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedBrowserState {
    #[serde(default)]
    pub current_path: Option<String>,
    #[serde(default)]
    pub sort_column: String,
    #[serde(default = "default_true")]
    pub sort_ascending: bool,
    #[serde(default = "default_file_list_width")]
    pub file_list_width: f64,
    #[serde(default)]
    pub type_filter: String,
}

impl Default for PersistedBrowserState {
    fn default() -> Self {
        Self {
            current_path: None,
            sort_column: String::new(),
            sort_ascending: true,
            file_list_width: default_file_list_width(),
            type_filter: String::new(),
        }
    }
}

/// Search tab session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSearchState {
    #[serde(default)]
    pub last_query: String,
    #[serde(default)]
    pub sort_column: String,
    #[serde(default = "default_true")]
    pub sort_ascending: bool,
}

impl Default for PersistedSearchState {
    fn default() -> Self {
        Self {
            last_query: String::new(),
            sort_column: String::new(),
            sort_ascending: true,
        }
    }
}

/// Dialogue tab session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDialogueState {
    #[serde(default)]
    pub language: String,
    #[serde(default = "default_true")]
    pub show_flags: bool,
    #[serde(default = "default_true")]
    pub show_tags: bool,
    #[serde(default)]
    pub show_editor_data: bool,
    #[serde(default = "default_browser_panel_width")]
    pub browser_panel_width: f64,
}

impl Default for PersistedDialogueState {
    fn default() -> Self {
        Self {
            language: String::new(),
            show_flags: true,
            show_tags: true,
            show_editor_data: false,
            browser_panel_width: default_browser_panel_width(),
        }
    }
}

/// Persistable configuration (saved to disk)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedConfig {
    // Existing fields
    pub bg3_data_path: Option<String>,
    pub recent_files: Vec<String>,
    #[serde(default)]
    pub theme: Theme,

    // Window state
    #[serde(default)]
    pub window: PersistedWindowState,

    // Active main tab
    #[serde(default)]
    pub active_tab: usize,

    // Per-tab session state
    #[serde(default)]
    pub editor: PersistedEditorState,
    #[serde(default)]
    pub browser: PersistedBrowserState,
    #[serde(default)]
    pub search: PersistedSearchState,
    #[serde(default)]
    pub dialogue: PersistedDialogueState,
    #[serde(default)]
    pub workbench: super::PersistedWorkbenchState,
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
            Some(
                "BG3 installation not found at default path. Please configure manually."
                    .to_string(),
            )
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

    /// Save current state to disk (preserves session state)
    pub fn save(&self) {
        // Load existing config to preserve session state
        let mut persisted = PersistedConfig::load();

        // Update only the core config fields
        persisted.bg3_data_path = Some(self.bg3_data_path.get());
        persisted.recent_files = self.recent_files.get();
        persisted.theme = self.theme.get();

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
            self.path_warning
                .set(Some("No BG3 path configured.".to_string()));
        } else if !path_exists(&path) {
            self.path_warning
                .set(Some("Path does not exist.".to_string()));
        } else {
            self.path_warning.set(None);
        }
    }

    /// Save complete session state (call on window close)
    pub fn save_session(
        &self,
        app_state: &super::AppState,
        editor_tabs: &super::EditorTabsState,
        browser: &super::BrowserState,
        search: &super::SearchState,
        dialogue: &super::DialogueState,
        workbench: &super::WorkbenchState,
    ) {
        // Collect editor tab file paths (only tabs with saved files)
        let open_files: Vec<String> = editor_tabs
            .tabs
            .get()
            .iter()
            .filter_map(|tab| tab.file_path.get())
            .collect();

        let persisted = PersistedConfig {
            // Existing fields
            bg3_data_path: Some(self.bg3_data_path.get()),
            recent_files: self.recent_files.get(),
            theme: self.theme.get(),

            // Window state (placeholder - actual values would need Floem window API)
            window: PersistedWindowState::default(),

            // Active main tab
            active_tab: app_state.active_tab.get(),

            // Editor state
            editor: PersistedEditorState {
                open_files,
                active_tab_index: editor_tabs.active_tab_index.get(),
                show_line_numbers: editor_tabs.show_line_numbers.get(),
            },

            // Browser state
            browser: PersistedBrowserState {
                current_path: browser.current_path.get(),
                sort_column: format!("{:?}", browser.sort_column.get()),
                sort_ascending: browser.sort_ascending.get(),
                file_list_width: browser.file_list_width.get(),
                type_filter: browser.type_filter.get(),
            },

            // Search state
            search: PersistedSearchState {
                last_query: search.query.get(),
                sort_column: format!("{:?}", search.sort_column.get()),
                sort_ascending: matches!(
                    search.sort_direction.get(),
                    super::SortDirection::Ascending
                ),
            },

            // Dialogue state
            dialogue: PersistedDialogueState {
                language: dialogue.language.get(),
                show_flags: dialogue.show_flags.get(),
                show_tags: dialogue.show_tags.get(),
                show_editor_data: dialogue.show_editor_data.get(),
                browser_panel_width: dialogue.browser_panel_width.get(),
            },

            // Workbench state
            workbench: {
                let ws = workbench.workbench.get();
                super::PersistedWorkbenchState {
                    last_open_project: ws
                        .as_ref()
                        .map(|w| w.project_dir.to_string_lossy().to_string()),
                    recent_projects: Vec::new(),
                }
            },
        };

        persisted.save();
    }
}

impl Default for ConfigState {
    fn default() -> Self {
        Self::new()
    }
}
