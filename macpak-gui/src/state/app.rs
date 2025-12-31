//! Global application state

use floem::prelude::*;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    /// Currently active tab index
    pub active_tab: RwSignal<usize>,

    /// Status message shown in the bottom bar
    pub status_message: RwSignal<String>,

    /// Whether a background operation is running
    pub is_busy: RwSignal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_tab: RwSignal::new(0),
            status_message: RwSignal::new(String::new()),
            is_busy: RwSignal::new(false),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
