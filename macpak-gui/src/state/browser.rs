//! Browser tab state

use floem::prelude::*;

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
    // 3D Preview state
    pub preview_3d_path: RwSignal<Option<String>>,  // Path to .glb file for 3D preview
    // Panel layout
    pub file_list_width: RwSignal<f64>,  // Width of file list panel in pixels
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
            preview_3d_path: RwSignal::new(None),
            file_list_width: RwSignal::new(600.0),  // Default width in pixels
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
