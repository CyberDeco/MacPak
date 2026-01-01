//! Global application state

use floem::prelude::*;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    /// Currently active tab index
    pub active_tab: RwSignal<usize>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_tab: RwSignal::new(0),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
