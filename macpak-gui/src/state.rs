//! Shared application state for MacPak

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

/// Editor-specific state
#[derive(Clone)]
pub struct EditorState {
    pub file_path: RwSignal<Option<String>>,
    pub file_format: RwSignal<String>,
    pub content: RwSignal<String>,
    pub modified: RwSignal<bool>,
    pub converted_from_lsf: RwSignal<bool>,
    pub status_message: RwSignal<String>,
    pub show_line_numbers: RwSignal<bool>,

    // Search state
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

impl EditorState {
    pub fn new() -> Self {
        Self {
            file_path: RwSignal::new(None),
            file_format: RwSignal::new(String::new()),
            content: RwSignal::new(String::new()),
            modified: RwSignal::new(false),
            converted_from_lsf: RwSignal::new(false),
            status_message: RwSignal::new(String::new()),
            show_line_numbers: RwSignal::new(true),

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
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

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

    // List contents (for file list view)
    pub list_contents: RwSignal<Vec<String>>,

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
            list_contents: RwSignal::new(Vec::new()),

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

/// UUID Generator state
#[derive(Clone)]
pub struct UuidGenState {
    // UUID
    pub generated_uuid: RwSignal<String>,
    pub uuid_format: RwSignal<UuidFormat>,
    pub uuid_history: RwSignal<Vec<String>>,

    // Handle
    pub generated_handle: RwSignal<String>,
    pub handle_history: RwSignal<Vec<String>>,

    // Status
    pub status_message: RwSignal<String>,
}

impl UuidGenState {
    pub fn new() -> Self {
        Self {
            generated_uuid: RwSignal::new(String::new()),
            uuid_format: RwSignal::new(UuidFormat::Standard),
            uuid_history: RwSignal::new(Vec::new()),

            generated_handle: RwSignal::new(String::new()),
            handle_history: RwSignal::new(Vec::new()),

            status_message: RwSignal::new(String::new()),
        }
    }
}

impl Default for UuidGenState {
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
