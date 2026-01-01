//! PAK Operations tab state

use floem::prelude::*;
use im::Vector as ImVector;
use std::collections::HashSet;

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

    #[allow(dead_code)]
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

    // Results log - uses im::Vector for virtual_list performance
    pub results_log: RwSignal<ImVector<String>>,

    // Status message (shown in header)
    pub status_message: RwSignal<String>,

    // Search filter for results
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

    // File selection dialog (for extract individual files)
    pub show_file_select: RwSignal<bool>,
    pub file_select_pak: RwSignal<Option<String>>,
    pub file_select_list: RwSignal<Vec<String>>,
    pub file_select_selected: RwSignal<HashSet<String>>,
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

            results_log: RwSignal::new(ImVector::new()),
            status_message: RwSignal::new(String::new()),
            file_search: RwSignal::new(String::new()),

            compression: RwSignal::new(PakCompression::Lz4Hc),
            priority: RwSignal::new(0),
            show_create_options: RwSignal::new(false),
            pending_create: RwSignal::new(None),

            working_dir: RwSignal::new(None),

            dropped_file: RwSignal::new(None),
            show_drop_dialog: RwSignal::new(false),

            show_file_select: RwSignal::new(false),
            file_select_pak: RwSignal::new(None),
            file_select_list: RwSignal::new(Vec::new()),
            file_select_selected: RwSignal::new(HashSet::new()),
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
            log.push_back(message.to_string());
        });
    }

    pub fn add_results_batch(&self, messages: Vec<String>) {
        self.results_log.update(|log| {
            for msg in messages {
                log.push_back(msg);
            }
        });
    }

    pub fn clear_results(&self) {
        self.results_log.set(ImVector::new());
    }
}

impl Default for PakOpsState {
    fn default() -> Self {
        Self::new()
    }
}
