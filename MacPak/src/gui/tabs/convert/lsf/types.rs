//! Types and shared state for LSF conversion operations

use floem::ext_event::create_ext_action;
use floem_reactive::{Scope, SignalUpdate};

use crate::gui::state::LsfConvertState;

// Re-export get_shared_progress for use by conversion.rs
pub use crate::gui::state::lsf_convert::get_shared_progress;

/// Result type for background LSF conversion operations
pub enum LsfResult {
    SingleDone {
        success: bool,
        input_name: String,
        output_name: String,
        error: Option<String>,
    },
    BatchDone {
        success_count: usize,
        error_count: usize,
        results: Vec<String>,
    },
}

/// Create a sender for background operations that updates UI on the main thread
pub fn create_result_sender(state: LsfConvertState) -> impl FnOnce(LsfResult) {
    create_ext_action(Scope::new(), move |result| {
        handle_lsf_result(state, result);
    })
}

/// Handle results from background LSF conversion operations
pub fn handle_lsf_result(state: LsfConvertState, result: LsfResult) {
    match result {
        LsfResult::SingleDone {
            success,
            input_name,
            output_name,
            error,
        } => {
            if success {
                state.add_result(&format!("Converted {} -> {}", input_name, output_name));
                state.status_message.set("Conversion complete!".to_string());
            } else {
                state.add_result(&format!("Error: {}", error.unwrap_or_default()));
                state.status_message.set("Conversion failed".to_string());
            }
            state.is_converting.set(false);
        }
        LsfResult::BatchDone {
            success_count,
            error_count,
            results,
        } => {
            state.add_results_batch(results);

            let status = if error_count == 0 {
                format!("Converted {} files successfully!", success_count)
            } else {
                format!(
                    "Completed: {} succeeded, {} failed",
                    success_count, error_count
                )
            };
            state.status_message.set(status);
            state.is_converting.set(false);
        }
    }
}
