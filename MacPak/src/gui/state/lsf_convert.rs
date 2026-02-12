//! LSF/LSX/LSJ/LOCA Conversion tab state

use floem::prelude::*;
use im::Vector as ImVector;

use crate::gui::shared::{BatchOperationState, SharedProgress};

/// Global shared progress instance for LSF conversion operations
static LSF_SHARED_PROGRESS: std::sync::OnceLock<SharedProgress> = std::sync::OnceLock::new();

/// Get or create the global shared progress instance for LSF conversions
pub fn get_shared_progress() -> &'static SharedProgress {
    LSF_SHARED_PROGRESS.get_or_init(SharedProgress::new)
}

/// LSF/LSX/LSJ/LOCA Conversion tab state
#[derive(Clone)]
pub struct LsfConvertState {
    // Single file conversion
    pub input_file: RwSignal<Option<String>>,
    pub detected_format: RwSignal<String>,
    pub target_format: RwSignal<String>,

    // LOCA/XML card source/target
    pub loca_source_format: RwSignal<String>,
    pub loca_target_format: RwSignal<String>,

    // Batch conversion
    pub batch_input_dir: RwSignal<Option<String>>,
    pub batch_source_format: RwSignal<String>,
    pub batch_target_format: RwSignal<String>,
    pub batch_files: RwSignal<Vec<String>>,

    // Progress
    pub is_converting: RwSignal<bool>,

    // Results - uses ImVector for efficient batch updates with virtual_list
    pub results_log: RwSignal<ImVector<String>>,
    pub status_message: RwSignal<String>,

    // Working directory for file dialogs
    pub working_dir: RwSignal<Option<String>>,
}

impl LsfConvertState {
    pub fn new() -> Self {
        Self {
            input_file: RwSignal::new(None),
            detected_format: RwSignal::new("LSF".to_string()),
            target_format: RwSignal::new("LSX".to_string()),
            loca_source_format: RwSignal::new("LOCA".to_string()),
            loca_target_format: RwSignal::new("XML".to_string()),
            batch_input_dir: RwSignal::new(None),
            batch_source_format: RwSignal::new("LSF".to_string()),
            batch_target_format: RwSignal::new("LSX".to_string()),
            batch_files: RwSignal::new(Vec::new()),
            is_converting: RwSignal::new(false),
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

impl Default for LsfConvertState {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchOperationState for LsfConvertState {
    fn is_processing(&self) -> RwSignal<bool> {
        self.is_converting
    }

    fn results_log(&self) -> RwSignal<ImVector<String>> {
        self.results_log
    }

    fn status_message(&self) -> RwSignal<String> {
        self.status_message
    }

    fn add_result(&self, message: &str) {
        LsfConvertState::add_result(self, message);
    }

    fn add_results_batch(&self, messages: Vec<String>) {
        LsfConvertState::add_results_batch(self, messages);
    }

    fn clear_results(&self) {
        LsfConvertState::clear_results(self);
    }

    fn get_shared_progress(&self) -> &'static SharedProgress {
        get_shared_progress()
    }
}
