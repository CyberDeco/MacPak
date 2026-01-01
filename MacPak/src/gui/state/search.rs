//! Search tab state

use floem::prelude::*;

/// Index Search state
#[derive(Clone)]
pub struct SearchState {
    pub query: RwSignal<String>,
    pub results: RwSignal<Vec<SearchResult>>,
    pub is_searching: RwSignal<bool>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: RwSignal::new(String::new()),
            results: RwSignal::new(Vec::new()),
            is_searching: RwSignal::new(false),
        }
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result entry
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub pak_file: String,
    pub file_type: String,
}
