//! GR2 Conversion tab state

use floem::prelude::*;
use im::Vector as ImVector;

use crate::gui::shared::{BatchOperationState, SharedProgress};

/// Global shared progress instance for GR2 operations
static GR2_SHARED_PROGRESS: std::sync::OnceLock<SharedProgress> = std::sync::OnceLock::new();

/// Get or create the global shared progress instance for GR2
pub fn get_shared_progress() -> &'static SharedProgress {
    GR2_SHARED_PROGRESS.get_or_init(SharedProgress::new)
}

/// GR2 Conversion tab state
#[derive(Clone)]
pub struct Gr2State {
    // Single file conversion
    pub input_file: RwSignal<Option<String>>,

    // Batch conversion
    pub batch_input_dir: RwSignal<Option<String>>,
    pub batch_files: RwSignal<Vec<String>>,

    // Progress (uses shared atomic state for thread-safe updates)
    pub is_converting: RwSignal<bool>,

    // Results - uses ImVector for efficient batch updates with virtual_list
    pub results_log: RwSignal<ImVector<String>>,
    pub status_message: RwSignal<String>,

    // Working directory for file dialogs
    pub working_dir: RwSignal<Option<String>>,
}

impl Gr2State {
    pub fn new() -> Self {
        Self {
            input_file: RwSignal::new(None),
            batch_input_dir: RwSignal::new(None),
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

impl Default for Gr2State {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchOperationState for Gr2State {
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
        Gr2State::add_result(self, message);
    }

    fn add_results_batch(&self, messages: Vec<String>) {
        Gr2State::add_results_batch(self, messages);
    }

    fn clear_results(&self) {
        Gr2State::clear_results(self);
    }

    fn get_shared_progress(&self) -> &'static SharedProgress {
        get_shared_progress()
    }
}
