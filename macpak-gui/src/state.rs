//! Shared application state for MacPak

use floem::prelude::*;
use im::Vector as ImVector;

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
/// TODO: Remove once all references are updated
pub type EditorState = EditorTab;

/// Sort column options for the file browser
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortColumn {
    Name,
    Type,
    Size,
    Modified,
}

/// Raw RGBA image data for preview display
#[derive(Clone)]
pub struct RawImageData {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
}

/// Asset Browser state
#[derive(Clone)]
pub struct BrowserState {
    pub current_path: RwSignal<Option<String>>,
    pub browser_path: RwSignal<String>,  // Editable path shown in toolbar
    pub selected_index: RwSignal<Option<usize>>,
    pub files: RwSignal<Vec<FileEntry>>,
    pub all_files: RwSignal<Vec<FileEntry>>,
    pub search_query: RwSignal<String>,
    pub type_filter: RwSignal<String>,
    pub preview_content: RwSignal<String>,
    pub preview_name: RwSignal<String>,
    pub preview_info: RwSignal<String>,
    pub preview_image: RwSignal<(u64, Option<RawImageData>)>,  // (version, data) - version forces rebuilds
    pub file_count: RwSignal<usize>,
    pub folder_count: RwSignal<usize>,
    pub total_size: RwSignal<String>,
    pub status_message: RwSignal<String>,
    pub sort_column: RwSignal<SortColumn>,
    pub sort_ascending: RwSignal<bool>,
    // Inline rename state
    pub renaming_path: RwSignal<Option<String>>,  // Path of file being renamed (None = not renaming)
    pub rename_text: RwSignal<String>,            // Current text in rename input
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            current_path: RwSignal::new(None),
            browser_path: RwSignal::new(String::new()),
            selected_index: RwSignal::new(None),
            files: RwSignal::new(Vec::new()),
            all_files: RwSignal::new(Vec::new()),
            search_query: RwSignal::new(String::new()),
            type_filter: RwSignal::new("All".to_string()),
            preview_content: RwSignal::new(String::new()),
            preview_name: RwSignal::new(String::new()),
            preview_info: RwSignal::new(String::new()),
            preview_image: RwSignal::new((0, None)),
            file_count: RwSignal::new(0),
            folder_count: RwSignal::new(0),
            total_size: RwSignal::new(String::new()),
            status_message: RwSignal::new(String::new()),
            sort_column: RwSignal::new(SortColumn::Name),
            sort_ascending: RwSignal::new(true),
            renaming_path: RwSignal::new(None),
            rename_text: RwSignal::new(String::new()),
        }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self::new()
    }
}

/// File entry for the asset browser
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub size_formatted: String,
    pub extension: String,
    pub file_type: String,
    pub modified: String,
    pub icon: String,
}

/// Compression options for PAK creation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PakCompression {
    Lz4Hc,
    Lz4,
    Zlib,
    ZlibFast,
    None,
}

impl PakCompression {
    pub fn as_str(&self) -> &'static str {
        match self {
            PakCompression::Lz4Hc => "lz4hc",
            PakCompression::Lz4 => "lz4",
            PakCompression::Zlib => "zlib",
            PakCompression::ZlibFast => "zlibfast",
            PakCompression::None => "none",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PakCompression::Lz4Hc => "Best compression (default)",
            PakCompression::Lz4 => "Fast compression",
            PakCompression::Zlib => "Standard compression",
            PakCompression::ZlibFast => "Fast zlib",
            PakCompression::None => "No compression",
        }
    }
}

/// PAK Operations state
#[derive(Clone)]
pub struct PakOpsState {
    // Operation progress (shared for all operations)
    pub progress: RwSignal<f32>,
    pub progress_message: RwSignal<String>,
    pub show_progress: RwSignal<bool>,

    // Operation flags
    pub is_extracting: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub is_listing: RwSignal<bool>,
    pub is_validating: RwSignal<bool>,

    // Results log
    pub results_log: RwSignal<Vec<String>>,

    // List contents (for file list view) - uses im::Vector for virtual_list performance
    pub list_contents: RwSignal<ImVector<String>>,
    pub file_search: RwSignal<String>,

    // PAK creation options
    pub compression: RwSignal<PakCompression>,
    pub priority: RwSignal<i32>,
    pub show_create_options: RwSignal<bool>,

    // Pending create operation (source, dest)
    pub pending_create: RwSignal<Option<(String, String)>>,

    // Working directory
    pub working_dir: RwSignal<Option<String>>,

    // Dropped file (for drag-drop dialog)
    pub dropped_file: RwSignal<Option<String>>,
    pub show_drop_dialog: RwSignal<bool>,
}

impl PakOpsState {
    pub fn new() -> Self {
        Self {
            progress: RwSignal::new(0.0),
            progress_message: RwSignal::new(String::new()),
            show_progress: RwSignal::new(false),

            is_extracting: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            is_listing: RwSignal::new(false),
            is_validating: RwSignal::new(false),

            results_log: RwSignal::new(Vec::new()),
            list_contents: RwSignal::new(ImVector::new()),
            file_search: RwSignal::new(String::new()),

            compression: RwSignal::new(PakCompression::Lz4Hc),
            priority: RwSignal::new(0),
            show_create_options: RwSignal::new(false),
            pending_create: RwSignal::new(None),

            working_dir: RwSignal::new(None),

            dropped_file: RwSignal::new(None),
            show_drop_dialog: RwSignal::new(false),
        }
    }

    pub fn is_busy(&self) -> bool {
        self.is_extracting.get()
            || self.is_creating.get()
            || self.is_listing.get()
            || self.is_validating.get()
    }

    pub fn add_result(&self, message: &str) {
        self.results_log.update(|log| {
            log.push(message.to_string());
        });
    }

    pub fn clear_results(&self) {
        self.results_log.set(Vec::new());
    }
}

impl Default for PakOpsState {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Tools tab state (UUID, Handle, Color Picker, Version Calculator)
#[derive(Clone)]
pub struct ToolsState {
    // UUID
    pub generated_uuid: RwSignal<String>,
    pub uuid_format: RwSignal<UuidFormat>,
    pub uuid_history: RwSignal<Vec<String>>,

    // Handle
    pub generated_handle: RwSignal<String>,
    pub handle_history: RwSignal<Vec<String>>,

    // Color Picker
    pub color_hex: RwSignal<String>,
    pub color_r: RwSignal<u8>,
    pub color_g: RwSignal<u8>,
    pub color_b: RwSignal<u8>,
    pub color_a: RwSignal<u8>,
    pub color_history: RwSignal<Vec<String>>,

    // Version Calculator
    pub version_int: RwSignal<String>,
    pub version_major: RwSignal<u32>,
    pub version_minor: RwSignal<u32>,
    pub version_patch: RwSignal<u32>,
    pub version_build: RwSignal<u32>,

    // Status
    pub status_message: RwSignal<String>,
}

impl ToolsState {
    pub fn new() -> Self {
        Self {
            generated_uuid: RwSignal::new(String::new()),
            uuid_format: RwSignal::new(UuidFormat::Standard),
            uuid_history: RwSignal::new(Vec::new()),

            generated_handle: RwSignal::new(String::new()),
            handle_history: RwSignal::new(Vec::new()),

            color_hex: RwSignal::new("FF5500".to_string()),
            color_r: RwSignal::new(255),
            color_g: RwSignal::new(85),
            color_b: RwSignal::new(0),
            color_a: RwSignal::new(255),
            color_history: RwSignal::new(Vec::new()),

            version_int: RwSignal::new(String::new()),
            version_major: RwSignal::new(1),
            version_minor: RwSignal::new(0),
            version_patch: RwSignal::new(0),
            version_build: RwSignal::new(0),

            status_message: RwSignal::new(String::new()),
        }
    }
}

impl Default for ToolsState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UuidFormat {
    Standard,    // 8-4-4-4-12
    Compact,     // No dashes
    Larian,      // Larian's format (h prefix + specific format)
}
