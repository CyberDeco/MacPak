//! Virtual Textures tab state

use floem::prelude::*;
use im::Vector as ImVector;

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
