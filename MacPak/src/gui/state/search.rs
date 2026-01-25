//! Search tab state

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use floem::prelude::*;
use maclarian::search::{ContentCache, FileType, IndexedFile, SearchIndex};

/// Index status for UI display
#[derive(Clone, Debug, PartialEq)]
pub enum IndexStatus {
    /// Index has not been built yet
    NotBuilt,
    /// Index is currently being built
    Building { progress: String },
    /// Index is ready with file count
    Ready { file_count: usize, pak_count: usize },
    /// Index building failed
    Error(String),
}

impl Default for IndexStatus {
    fn default() -> Self {
        IndexStatus::NotBuilt
    }
}

/// Column to sort search results by
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SearchSortColumn {
    #[default]
    Name,
    Type,
    Pak,
    Path,
}

/// Sort direction
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// Index Search state
#[derive(Clone)]
pub struct SearchState {
    /// Current search query
    pub query: RwSignal<String>,
    /// Search results to display
    pub results: RwSignal<Vec<SearchResult>>,
    /// Whether a search is in progress
    pub is_searching: RwSignal<bool>,
    /// Active file type filter (None = all types)
    pub active_filter: RwSignal<Option<FileType>>,
    /// Index status for display
    pub index_status: RwSignal<IndexStatus>,
    /// Shared search index (thread-safe)
    pub index: Arc<RwLock<SearchIndex>>,
    /// Shared content cache for deep search (thread-safe)
    pub content_cache: Arc<RwLock<ContentCache>>,
    /// Configured PAK paths to search
    pub pak_paths: RwSignal<Vec<std::path::PathBuf>>,
    /// Show progress dialog during long operations
    pub show_progress: RwSignal<bool>,
    /// Progress message for dialog
    pub progress_message: RwSignal<String>,
    /// Progress percentage (0-100)
    pub progress_percent: RwSignal<u32>,
    /// Current progress count
    pub progress_current: RwSignal<usize>,
    /// Total progress count
    pub progress_total: RwSignal<usize>,
    /// Current sort column
    pub sort_column: RwSignal<SearchSortColumn>,
    /// Current sort direction
    pub sort_direction: RwSignal<SortDirection>,
    /// Selected result paths for multi-select operations
    pub selected_results: RwSignal<HashSet<String>>,
    /// Show "All Matches" dialog
    pub show_all_matches: RwSignal<bool>,
    /// Current file for "Show All Matches" dialog
    pub all_matches_file: RwSignal<Option<SearchResult>>,

    // Extraction dialog state
    /// Show extraction options dialog
    pub show_extract_dialog: RwSignal<bool>,
    /// Files pending extraction (set before showing dialog)
    pub pending_extract_files: RwSignal<Vec<(String, std::path::PathBuf)>>, // (internal_path, pak_path)

    // GR2 bundle options for extraction
    pub gr2_extract_gr2: RwSignal<bool>,
    pub gr2_convert_to_glb: RwSignal<bool>,
    pub gr2_convert_to_gltf: RwSignal<bool>,
    pub gr2_extract_textures: RwSignal<bool>,
    pub gr2_convert_to_png: RwSignal<bool>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: RwSignal::new(String::new()),
            results: RwSignal::new(Vec::new()),
            is_searching: RwSignal::new(false),
            active_filter: RwSignal::new(None),
            index_status: RwSignal::new(IndexStatus::NotBuilt),
            index: Arc::new(RwLock::new(SearchIndex::new())),
            content_cache: Arc::new(RwLock::new(ContentCache::new())),
            pak_paths: RwSignal::new(Vec::new()),
            show_progress: RwSignal::new(false),
            progress_message: RwSignal::new(String::new()),
            progress_percent: RwSignal::new(0),
            progress_current: RwSignal::new(0),
            progress_total: RwSignal::new(0),
            sort_column: RwSignal::new(SearchSortColumn::default()),
            sort_direction: RwSignal::new(SortDirection::default()),
            selected_results: RwSignal::new(HashSet::new()),
            show_all_matches: RwSignal::new(false),
            all_matches_file: RwSignal::new(None),

            // Extraction dialog
            show_extract_dialog: RwSignal::new(false),
            pending_extract_files: RwSignal::new(Vec::new()),

            // GR2 options default to off (user opts in)
            gr2_extract_gr2: RwSignal::new(true),
            gr2_convert_to_glb: RwSignal::new(false),
            gr2_convert_to_gltf: RwSignal::new(false),
            gr2_extract_textures: RwSignal::new(false),
            gr2_convert_to_png: RwSignal::new(false),
        }
    }

    /// Apply persisted state (call after new())
    pub fn apply_persisted(&self, persisted: &super::PersistedSearchState) {
        // Restore last query
        if !persisted.last_query.is_empty() {
            self.query.set(persisted.last_query.clone());
        }

        // Restore sort preferences
        let sort_column = match persisted.sort_column.as_str() {
            "Type" => SearchSortColumn::Type,
            "Pak" => SearchSortColumn::Pak,
            "Path" => SearchSortColumn::Path,
            _ => SearchSortColumn::Name,
        };
        self.sort_column.set(sort_column);

        let sort_direction = if persisted.sort_ascending {
            SortDirection::Ascending
        } else {
            SortDirection::Descending
        };
        self.sort_direction.set(sort_direction);
    }

    /// Check if the index has been built
    pub fn is_indexed(&self) -> bool {
        matches!(self.index_status.get(), IndexStatus::Ready { .. })
    }

    /// Get the number of indexed files
    pub fn file_count(&self) -> usize {
        match self.index_status.get() {
            IndexStatus::Ready { file_count, .. } => file_count,
            _ => 0,
        }
    }

    /// Clear search results
    pub fn clear_results(&self) {
        self.results.set(Vec::new());
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result entry for display
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// Filename
    pub name: String,
    /// Full internal path
    pub path: String,
    /// Source PAK file name
    pub pak_file: String,
    /// File type for icon display
    pub file_type: String,
    /// Full PAK path for extraction
    pub pak_path: std::path::PathBuf,
    /// Match context (for content search)
    pub context: Option<String>,
    /// Number of matches in the file (for content search)
    pub match_count: Option<usize>,
}

impl SearchResult {
    /// Create from an IndexedFile
    pub fn from_indexed_file(file: &IndexedFile) -> Self {
        Self {
            name: file.name.clone(),
            path: file.path.clone(),
            pak_file: file.pak_file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            file_type: file.file_type.display_name().to_string(),
            pak_path: file.pak_file.clone(),
            context: None,
            match_count: None,
        }
    }

    /// Create from a content match
    pub fn from_content_match(
        file: &IndexedFile,
        context: String,
        match_count: usize,
    ) -> Self {
        let mut result = Self::from_indexed_file(file);
        result.context = Some(context);
        result.match_count = Some(match_count);
        result
    }
}
