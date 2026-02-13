//! Types and shared state for Virtual Texture operations

use floem::ext_event::create_ext_action;
use floem_reactive::{Scope, SignalUpdate};

use crate::gui::state::VirtualTexturesState;

// Re-export get_shared_progress for use by extraction.rs
pub use crate::gui::state::virtual_textures::get_shared_progress;

/// Result type for background Virtual Texture operations
pub enum VtResult {
    SingleDone {
        success: bool,
        gts_name: String,
        texture_count: usize,
        error: Option<String>,
    },
    BatchDone {
        success_count: usize,
        error_count: usize,
        texture_count: usize,
        results: Vec<String>,
    },
}

/// Create a sender for background operations that updates UI on the main thread
pub fn create_result_sender(state: VirtualTexturesState) -> impl FnOnce(VtResult) {
    create_ext_action(Scope::new(), move |result| {
        handle_vt_result(state, result);
    })
}

/// Handle results from background Virtual Texture operations
pub fn handle_vt_result(state: VirtualTexturesState, result: VtResult) {
    match result {
        VtResult::SingleDone {
            success,
            gts_name,
            texture_count,
            error,
        } => {
            if success {
                state.add_result(&format!(
                    "Extracted {} textures from {}",
                    texture_count, gts_name
                ));
                state.status_message.set("Extraction complete!".to_string());
            } else {
                state.add_result(&format!("Error: {}", error.unwrap_or_default()));
                state.status_message.set("Extraction failed".to_string());
            }
            state.is_extracting.set(false);
        }
        VtResult::BatchDone {
            success_count,
            error_count,
            texture_count,
            results,
        } => {
            // Use batch update to avoid UI freezing with large result sets
            state.add_results_batch(results);

            let status = if error_count == 0 {
                format!(
                    "Extracted {} textures from {} GTS files!",
                    texture_count, success_count
                )
            } else {
                format!(
                    "Completed: {} succeeded, {} failed ({} textures)",
                    success_count, error_count, texture_count
                )
            };
            state.status_message.set(status);
            state.is_extracting.set(false);
        }
    }
}
