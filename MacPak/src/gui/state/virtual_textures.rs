//! Virtual Textures tab state

use floem::prelude::*;
use im::Vector as ImVector;

use crate::gui::shared::{BatchOperationState, SharedProgress};

/// Global shared progress instance for Virtual Textures operations
static VT_SHARED_PROGRESS: std::sync::OnceLock<SharedProgress> = std::sync::OnceLock::new();

/// Get or create the global shared progress instance for Virtual Textures
pub fn get_shared_progress() -> &'static SharedProgress {
    VT_SHARED_PROGRESS.get_or_init(SharedProgress::new)
}

/// Virtual Textures extraction state
#[derive(Clone)]
pub struct VirtualTexturesState {
    // Single file extraction
    pub gts_file: RwSignal<Option<String>>,

    // Batch extraction
    pub batch_input_dir: RwSignal<Option<String>>,
    pub batch_output_dir: RwSignal<Option<String>>,
    pub batch_gts_files: RwSignal<Vec<String>>,

    // Layer selection: None = All layers, Some(n) = specific layer
    pub selected_layer: RwSignal<Option<usize>>,

    // Progress (uses shared atomic state for thread-safe updates)
    pub is_extracting: RwSignal<bool>,

    // Results - uses ImVector for efficient batch updates with virtual_list
    pub results_log: RwSignal<ImVector<String>>,
    pub status_message: RwSignal<String>,

    // Working directory for file dialogs
    pub working_dir: RwSignal<Option<String>>,
}

impl VirtualTexturesState {
    pub fn new() -> Self {
        Self {
            gts_file: RwSignal::new(None),
            batch_input_dir: RwSignal::new(None),
            batch_output_dir: RwSignal::new(None),
            batch_gts_files: RwSignal::new(Vec::new()),
            selected_layer: RwSignal::new(None), // Default to All Layers
            is_extracting: RwSignal::new(false),
            results_log: RwSignal::new(ImVector::new()),
            status_message: RwSignal::new(String::new()),
            working_dir: RwSignal::new(None),
        }
    }

    pub fn add_result(&self, message: &str) {
        self.results_log.update(|log| {
            log.push_back(message.to_string());
        });
    }

    /// Add multiple results in a single batch update (avoids UI freezing)
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

impl Default for VirtualTexturesState {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchOperationState for VirtualTexturesState {
    fn is_processing(&self) -> RwSignal<bool> {
        self.is_extracting
    }

    fn results_log(&self) -> RwSignal<ImVector<String>> {
        self.results_log
    }

    fn status_message(&self) -> RwSignal<String> {
        self.status_message
    }

    fn add_result(&self, message: &str) {
        VirtualTexturesState::add_result(self, message);
    }

    fn add_results_batch(&self, messages: Vec<String>) {
        VirtualTexturesState::add_results_batch(self, messages);
    }

    fn clear_results(&self) {
        VirtualTexturesState::clear_results(self);
    }

    fn get_shared_progress(&self) -> &'static SharedProgress {
        get_shared_progress()
    }
}
