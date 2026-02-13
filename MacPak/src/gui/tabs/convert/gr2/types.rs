//! Types and shared state for GR2 operations

use floem::ext_event::create_ext_action;
use floem_reactive::{Scope, SignalUpdate};

use crate::gui::state::Gr2State;

// Re-export get_shared_progress for use by conversion.rs
pub use crate::gui::state::gr2::get_shared_progress;

/// Result type for background GR2 operations
pub enum Gr2Result {
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
pub fn create_result_sender(state: Gr2State) -> impl FnOnce(Gr2Result) {
    create_ext_action(Scope::new(), move |result| {
        handle_gr2_result(state, result);
    })
}

/// Handle results from background GR2 operations
pub fn handle_gr2_result(state: Gr2State, result: Gr2Result) {
    match result {
        Gr2Result::SingleDone {
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
        Gr2Result::BatchDone {
            success_count,
            error_count,
            results,
        } => {
            // Use batch update to avoid UI freezing with large result sets
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
