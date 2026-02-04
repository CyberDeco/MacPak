//! Workspace tab state

use floem::prelude::*;

use crate::workspace::Workspace;
use crate::workspace::recipe::Recipe;

/// Reactive state for the Workspace tab
#[derive(Clone)]
pub struct WorkspaceState {
    /// Currently open workspace (None = no project open)
    pub workspace: RwSignal<Option<Workspace>>,
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
}

impl WorkspaceState {
    pub fn new() -> Self {
        let recipes = crate::workspace::recipe::load_bundled_recipes();
        Self {
            workspace: RwSignal::new(None),
            recipes: RwSignal::new(recipes),
            show_new_dialog: RwSignal::new(false),
            build_progress: RwSignal::new(None),
            result_message: RwSignal::new(None),
            error_message: RwSignal::new(None),
        }
    }

    /// Apply persisted state (restore last open project)
    pub fn apply_persisted(&self, persisted: &PersistedWorkspaceState) {
        if let Some(ref path) = persisted.last_open_project {
            match Workspace::open(path) {
                Ok(ws) => self.workspace.set(Some(ws)),
                Err(e) => {
                    tracing::warn!("Failed to restore workspace {}: {}", path, e);
                }
            }
        }
    }
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self::new()
    }
}

/// Persisted workspace state (saved to config.json)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct PersistedWorkspaceState {
    #[serde(default)]
    pub recent_projects: Vec<String>,
    #[serde(default)]
    pub last_open_project: Option<String>,
}
