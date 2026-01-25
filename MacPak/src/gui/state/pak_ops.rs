//! PAK Operations tab state

use floem::prelude::*;
use im::Vector as ImVector;
use std::collections::HashSet;

/// Which dialog is currently active (only one at a time)
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ActiveDialog {
    #[default]
    None,
    Progress,
    CreateOptions,
    DropAction,
    FileSelect,
    FolderDropAction,
}

/// Compression options for PAK creation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PakCompression {
    Lz4Hc,
    Lz4,
    None,
}

impl PakCompression {
    pub fn as_str(&self) -> &'static str {
        match self {
            PakCompression::Lz4Hc => "lz4hc",
            PakCompression::Lz4 => "lz4",
            PakCompression::None => "none",
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            PakCompression::Lz4Hc => "Best compression (default)",
            PakCompression::Lz4 => "Fast compression",
            PakCompression::None => "No compression",
        }
    }
}

/// PAK Operations state
#[derive(Clone)]
pub struct PakOpsState {
    // Active dialog (only one at a time)
    pub active_dialog: RwSignal<ActiveDialog>,

    // Operation progress (shared for all operations)
    pub progress: RwSignal<f32>,
    pub progress_message: RwSignal<String>,

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
    pub generate_info_json: RwSignal<bool>,

    // Pending create operation (source, dest)
    pub pending_create: RwSignal<Option<(String, String)>>,

    // Working directory
    pub working_dir: RwSignal<Option<String>>,

    // Dropped file (for drag-drop dialog)
    pub dropped_file: RwSignal<Option<String>>,

    // Dropped folder (for folder drag-drop dialog)
    pub dropped_folder: RwSignal<Option<String>>,

    // File selection dialog (for extract individual files)
    pub file_select_pak: RwSignal<Option<String>>,
    pub file_select_list: RwSignal<Vec<String>>,
    pub file_select_selected: RwSignal<HashSet<String>>,
    pub file_select_filter: RwSignal<String>,

    // GR2 extraction options (shown when GR2 files are in selection)
    pub gr2_auto_convert: RwSignal<bool>,
    pub gr2_auto_textures: RwSignal<bool>,
    pub gr2_auto_virtual_textures: RwSignal<bool>,
    pub keep_original_gr2: RwSignal<bool>,
    pub game_data_path: RwSignal<Option<String>>,
    pub virtual_textures_path: RwSignal<Option<String>>,

    // Progress polling signals (persistent to avoid accumulation on tab switch)
    pub polled_pct: RwSignal<u32>,
    pub polled_current: RwSignal<u32>,
    pub polled_total: RwSignal<u32>,
    pub polled_msg: RwSignal<String>,
    pub timer_active: RwSignal<bool>,
    /// Guards against multiple effect registrations
    pub polling_effect_registered: RwSignal<bool>,
}

impl PakOpsState {
    pub fn new() -> Self {
        Self {
            active_dialog: RwSignal::new(ActiveDialog::None),

            progress: RwSignal::new(0.0),
            progress_message: RwSignal::new(String::new()),

            is_extracting: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            is_listing: RwSignal::new(false),
            is_validating: RwSignal::new(false),

            results_log: RwSignal::new(ImVector::new()),
            status_message: RwSignal::new(String::new()),
            file_search: RwSignal::new(String::new()),

            compression: RwSignal::new(PakCompression::Lz4Hc),
            priority: RwSignal::new(0),
            generate_info_json: RwSignal::new(true), // Default to true for BaldursModManager compatibility
            pending_create: RwSignal::new(None),

            working_dir: RwSignal::new(None),

            dropped_file: RwSignal::new(None),
            dropped_folder: RwSignal::new(None),

            file_select_pak: RwSignal::new(None),
            file_select_list: RwSignal::new(Vec::new()),
            file_select_selected: RwSignal::new(HashSet::new()),
            file_select_filter: RwSignal::new(String::new()),

            // GR2 options default to off (user opts in)
            gr2_auto_convert: RwSignal::new(false),
            gr2_auto_textures: RwSignal::new(false),
            gr2_auto_virtual_textures: RwSignal::new(false),
            keep_original_gr2: RwSignal::new(true),
            game_data_path: RwSignal::new(None),
            virtual_textures_path: RwSignal::new(None),

            polled_pct: RwSignal::new(0),
            polled_current: RwSignal::new(0),
            polled_total: RwSignal::new(0),
            polled_msg: RwSignal::new(String::new()),
            timer_active: RwSignal::new(false),
            polling_effect_registered: RwSignal::new(false),
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
