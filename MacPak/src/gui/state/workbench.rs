//! Workbench tab state

use std::collections::HashMap;
use std::path::PathBuf;

use floem::prelude::*;

use crate::workbench::Workbench;
use crate::workbench::recipe::Recipe;

/// Reactive state for the Workbench tab
#[derive(Clone)]
pub struct WorkbenchState {
    /// Currently open workbench (None = no project open)
    pub workbench: RwSignal<Option<Workbench>>,
    /// Available recipes (loaded once at startup)
    pub recipes: RwSignal<Vec<Recipe>>,
    /// Whether the "New Project" dialog is showing
    pub show_new_dialog: RwSignal<bool>,
    /// Build progress message
    pub build_progress: RwSignal<Option<String>>,
    /// Build/validation result message
    pub result_message: RwSignal<Option<String>>,
    /// Error message
    pub error_message: RwSignal<Option<String>>,
    /// Persisted expanded state for the file tree (full path → expanded).
    /// Survives tab switches since WorkbenchState lives in app state.
    pub file_tree_expanded: RwSignal<HashMap<PathBuf, bool>>,
}

impl WorkbenchState {
    pub fn new() -> Self {
        let recipes = crate::workbench::recipe::load_bundled_recipes();
        Self {
            workbench: RwSignal::new(None),
            recipes: RwSignal::new(recipes),
            show_new_dialog: RwSignal::new(false),
            build_progress: RwSignal::new(None),
            result_message: RwSignal::new(None),
            error_message: RwSignal::new(None),
            file_tree_expanded: RwSignal::new(HashMap::new()),
        }
    }

    /// Apply persisted state (restore last open project)
    pub fn apply_persisted(&self, persisted: &PersistedWorkbenchState) {
        if let Some(ref path) = persisted.last_open_project {
            match Workbench::open(path) {
                Ok(ws) => self.workbench.set(Some(ws)),
                Err(e) => {
                    tracing::warn!("Failed to restore workbench {}: {}", path, e);
                }
            }
        }
    }
}

impl Default for WorkbenchState {
    fn default() -> Self {
        Self::new()
    }
}

/// Persisted workbench state (saved to config.json)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct PersistedWorkbenchState {
    #[serde(default)]
    pub recent_projects: Vec<String>,
    #[serde(default)]
    pub last_open_project: Option<String>,
}
