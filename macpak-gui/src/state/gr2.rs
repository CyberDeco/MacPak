//! GR2 Conversion tab state

use floem::prelude::*;
use im::Vector as ImVector;

/// Output format for GR2 conversion
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gr2OutputFormat {
    Glb,   // Single binary file (.glb)
    Gltf,  // Separate .gltf + .bin files
}

impl Gr2OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Gr2OutputFormat::Glb => "glb",
            Gr2OutputFormat::Gltf => "gltf",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Gr2OutputFormat::Glb => "GLB (single binary)",
            Gr2OutputFormat::Gltf => "glTF (separate files)",
        }
    }
}

/// Conversion direction for GR2 tab
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gr2ConversionDirection {
    Gr2ToGltf,  // GR2 → glTF/GLB
    GltfToGr2,  // glTF/GLB → GR2
}

/// GR2 Conversion tab state
#[derive(Clone)]
pub struct Gr2State {
    // Conversion direction
    pub direction: RwSignal<Gr2ConversionDirection>,

    // Output format (for GR2 → glTF direction)
    pub output_format: RwSignal<Gr2OutputFormat>,

    // Single file conversion
    pub input_file: RwSignal<Option<String>>,
    pub output_file: RwSignal<Option<String>>,

    // Batch conversion
    pub batch_input_dir: RwSignal<Option<String>>,
    pub batch_output_dir: RwSignal<Option<String>>,
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
            direction: RwSignal::new(Gr2ConversionDirection::Gr2ToGltf),
            output_format: RwSignal::new(Gr2OutputFormat::Glb),
            input_file: RwSignal::new(None),
            output_file: RwSignal::new(None),
            batch_input_dir: RwSignal::new(None),
            batch_output_dir: RwSignal::new(None),
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
