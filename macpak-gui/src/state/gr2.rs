//! GR2 Conversion tab state

use floem::prelude::*;
use im::Vector as ImVector;

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
