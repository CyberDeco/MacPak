//! Types and shared state for GR2 operations

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::gui::state::Gr2State;

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

/// Shared progress state that can be updated from background threads
/// and polled from the UI thread
#[derive(Clone)]
pub struct SharedProgress {
    /// Progress as integer percentage (0-100)
    pub progress_pct: Arc<AtomicU32>,
    /// Current item index (1-based for display)
    pub current: Arc<AtomicU32>,
    /// Total items
    pub total: Arc<AtomicU32>,
    /// Current progress message (filename)
    pub message: Arc<Mutex<String>>,
}

impl SharedProgress {
    pub fn new() -> Self {
        Self {
            progress_pct: Arc::new(AtomicU32::new(0)),
            current: Arc::new(AtomicU32::new(0)),
            total: Arc::new(AtomicU32::new(0)),
            message: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Update progress from background thread
    pub fn update(&self, current: usize, total: usize, description: &str) {
        let pct = if total > 0 {
            ((current as f64 / total as f64) * 100.0) as u32
        } else {
            0
        };
        self.progress_pct.store(pct, Ordering::SeqCst);
        self.current.store((current + 1) as u32, Ordering::SeqCst); // 1-based for display
        self.total.store(total as u32, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            *msg = description.to_string();
        }
    }

    /// Get current progress percentage (0-100)
    pub fn get_pct(&self) -> u32 {
        self.progress_pct.load(Ordering::SeqCst)
    }

    /// Get current/total as (current, total)
    pub fn get_counts(&self) -> (u32, u32) {
        (
            self.current.load(Ordering::SeqCst),
            self.total.load(Ordering::SeqCst),
        )
    }

    /// Get current message
    pub fn get_message(&self) -> String {
        self.message.lock().map(|m| m.clone()).unwrap_or_default()
    }

    /// Reset progress
    pub fn reset(&self) {
        self.progress_pct.store(0, Ordering::SeqCst);
        self.current.store(0, Ordering::SeqCst);
        self.total.store(0, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            msg.clear();
        }
    }
}

impl Default for SharedProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Global shared progress instance for GR2 operations
static SHARED_PROGRESS: std::sync::OnceLock<SharedProgress> = std::sync::OnceLock::new();

/// Get or create the global shared progress instance
pub fn get_shared_progress() -> &'static SharedProgress {
    SHARED_PROGRESS.get_or_init(SharedProgress::new)
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
                format!("Completed: {} succeeded, {} failed", success_count, error_count)
            };
            state.status_message.set(status);
            state.is_converting.set(false);
        }
    }
}
