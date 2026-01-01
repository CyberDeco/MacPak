//! Editor tab state

use floem::prelude::*;

/// State for a single editor tab
#[derive(Clone)]
pub struct EditorTab {
    /// Unique ID for this tab
    pub id: u64,
    pub file_path: RwSignal<Option<String>>,
    pub file_format: RwSignal<String>,
    pub content: RwSignal<String>,
    pub modified: RwSignal<bool>,
    pub converted_from_lsf: RwSignal<bool>,

    // Search state (per-tab)
    pub search_visible: RwSignal<bool>,
    pub search_text: RwSignal<String>,
    pub replace_text: RwSignal<String>,
    pub case_sensitive: RwSignal<bool>,
    pub whole_words: RwSignal<bool>,
    pub use_regex: RwSignal<bool>,
    pub match_count: RwSignal<usize>,
    pub current_match: RwSignal<usize>,
    pub search_status: RwSignal<String>,
}

impl EditorTab {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            file_path: RwSignal::new(None),
            file_format: RwSignal::new(String::new()),
            content: RwSignal::new(String::new()),
            modified: RwSignal::new(false),
            converted_from_lsf: RwSignal::new(false),

            search_visible: RwSignal::new(false),
            search_text: RwSignal::new(String::new()),
            replace_text: RwSignal::new(String::new()),
            case_sensitive: RwSignal::new(false),
            whole_words: RwSignal::new(false),
            use_regex: RwSignal::new(false),
            match_count: RwSignal::new(0),
            current_match: RwSignal::new(0),
            search_status: RwSignal::new(String::new()),
        }
    }

    /// Get display name for tab (filename or "Untitled")
    pub fn display_name(&self) -> String {
        self.file_path
            .get()
            .and_then(|p| {
                std::path::Path::new(&p)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "Untitled".to_string())
    }
}

/// Multi-tab editor state
#[derive(Clone)]
pub struct EditorTabsState {
    /// All open tabs
    pub tabs: RwSignal<Vec<EditorTab>>,
    /// Index of currently active tab
    pub active_tab_index: RwSignal<usize>,
    /// Counter for generating unique tab IDs
    pub next_tab_id: RwSignal<u64>,
    /// Global status message
    pub status_message: RwSignal<String>,
    /// Show line numbers (global setting)
    pub show_line_numbers: RwSignal<bool>,

    // Meta.lsx Generator Dialog visibility
    pub show_meta_dialog: RwSignal<bool>,
}

impl EditorTabsState {
    pub fn new() -> Self {
        // Start with one empty tab
        let initial_tab = EditorTab::new(0);
        Self {
            tabs: RwSignal::new(vec![initial_tab]),
            active_tab_index: RwSignal::new(0),
            next_tab_id: RwSignal::new(1),
            status_message: RwSignal::new(String::new()),
            show_line_numbers: RwSignal::new(true),

            show_meta_dialog: RwSignal::new(false),
        }
    }

    /// Get the currently active tab
    pub fn active_tab(&self) -> Option<EditorTab> {
        let tabs = self.tabs.get();
        let index = self.active_tab_index.get();
        tabs.get(index).cloned()
    }

    /// Create a new empty tab and make it active
    pub fn new_tab(&self) -> EditorTab {
        let id = self.next_tab_id.get();
        self.next_tab_id.set(id + 1);

        let tab = EditorTab::new(id);
        let tab_clone = tab.clone();

        self.tabs.update(|tabs| {
            tabs.push(tab);
        });

        let new_index = self.tabs.get().len() - 1;
        self.active_tab_index.set(new_index);

        tab_clone
    }

    /// Close tab at index
    pub fn close_tab(&self, index: usize) {
        let tabs = self.tabs.get();
        if tabs.len() <= 1 {
            // Don't close the last tab, just clear it
            if let Some(tab) = tabs.first() {
                tab.file_path.set(None);
                tab.file_format.set(String::new());
                tab.content.set(String::new());
                tab.modified.set(false);
                tab.converted_from_lsf.set(false);
            }
            return;
        }

        self.tabs.update(|tabs| {
            if index < tabs.len() {
                tabs.remove(index);
            }
        });

        // Adjust active tab index
        let current = self.active_tab_index.get();
        if current >= index && current > 0 {
            self.active_tab_index.set(current - 1);
        }
    }

    /// Close all tabs except the one at index
    pub fn close_others(&self, keep_index: usize) {
        let tabs = self.tabs.get();
        if let Some(tab_to_keep) = tabs.get(keep_index).cloned() {
            self.tabs.set(vec![tab_to_keep]);
            self.active_tab_index.set(0);
        }
    }

    /// Close all tabs (creates a fresh empty tab)
    pub fn close_all(&self) {
        let id = self.next_tab_id.get();
        self.next_tab_id.set(id + 1);
        self.tabs.set(vec![EditorTab::new(id)]);
        self.active_tab_index.set(0);
    }

    /// Find tab by file path and switch to it, returns true if found
    pub fn switch_to_file(&self, path: &str) -> bool {
        let tabs = self.tabs.get();
        for (index, tab) in tabs.iter().enumerate() {
            if let Some(tab_path) = tab.file_path.get() {
                if tab_path == path {
                    self.active_tab_index.set(index);
                    return true;
                }
            }
        }
        false
    }

    /// Check if any tab has unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.tabs.get().iter().any(|tab| tab.modified.get())
    }
}

impl Default for EditorTabsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy EditorState alias for backward compatibility during transition
pub type EditorState = EditorTab;
