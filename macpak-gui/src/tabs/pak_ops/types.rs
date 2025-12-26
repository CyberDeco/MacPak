//! Types and result handling for PAK operations

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::state::PakOpsState;

/// Result type for background PAK operations
pub enum PakResult {
    ExtractDone {
        success: bool,
        message: String,
        file_count: usize,
        dest: String,
    },
    ListDone {
        success: bool,
        files: Vec<String>,
        pak_name: String,
        error: Option<String>,
    },
    CreateDone {
        success: bool,
        message: String,
        pak_name: String,
    },
    ValidateDone {
        valid: bool,
        structure: Vec<String>,
        warnings: Vec<String>,
    },
    BatchExtractDone {
        success_count: usize,
        fail_count: usize,
        results: Vec<String>,
        dest: String,
    },
    BatchCreateDone {
        success_count: usize,
        fail_count: usize,
        results: Vec<String>,
        dest: String,
    },
}

/// Shared progress state that can be updated from background threads
/// and polled from the UI thread
#[derive(Clone)]
pub struct SharedProgress {
    /// Progress as integer percentage (0-100), stored as u32 for atomic access
    pub progress_pct: Arc<AtomicU32>,
    /// Current progress message
    pub message: Arc<Mutex<String>>,
}

impl SharedProgress {
    pub fn new() -> Self {
        Self {
            progress_pct: Arc::new(AtomicU32::new(0)),
            message: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Update progress from background thread (lock-free for percentage)
    pub fn update(&self, current: usize, total: usize, description: &str) {
        let pct = if total > 0 {
            ((current as f64 / total as f64) * 100.0) as u32
        } else {
            0
        };
        self.progress_pct.store(pct, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            *msg = description.to_string();
        }
    }

    /// Get current progress percentage (0-100)
    pub fn get_pct(&self) -> u32 {
        self.progress_pct.load(Ordering::SeqCst)
    }

    /// Get current message
    pub fn get_message(&self) -> String {
        self.message.lock().map(|m| m.clone()).unwrap_or_default()
    }

    /// Reset progress to initial state (call when starting a new operation)
    pub fn reset(&self) {
        self.progress_pct.store(0, Ordering::SeqCst);
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

/// Global shared progress instance
static SHARED_PROGRESS: std::sync::OnceLock<SharedProgress> = std::sync::OnceLock::new();

/// Get or create the global shared progress instance
pub fn get_shared_progress() -> &'static SharedProgress {
    SHARED_PROGRESS.get_or_init(SharedProgress::new)
}

/// Create a sender for background operations that updates UI on the main thread
pub fn create_result_sender(state: PakOpsState) -> impl FnOnce(PakResult) {
    create_ext_action(Scope::new(), move |result| {
        handle_pak_result(state, result);
    })
}

/// Create a reusable sender for progress updates that uses shared atomic state
pub fn create_progress_sender(_state: PakOpsState) -> impl Fn(usize, usize, &str) + Send + Clone {
    let shared = get_shared_progress().clone();

    move |current: usize, total: usize, description: &str| {
        shared.update(current, total, description);
    }
}

/// Handle results from background PAK operations
pub fn handle_pak_result(state: PakOpsState, result: PakResult) {
    match result {
        PakResult::ListDone {
            success,
            files,
            pak_name,
            error,
        } => {
            state.progress.set(1.0);

            if success {
                state.add_result(&format!("✅ Loaded {} files from {}", files.len(), pak_name));
                // Clear search and set file list (convert Vec to im::Vector for virtual_list)
                state.file_search.set(String::new());
                state.list_contents.set(files.into_iter().collect());
            } else {
                state.add_result(&format!(
                    "❌ Failed to list PAK contents: {}",
                    error.unwrap_or_default()
                ));
            }

            state.is_listing.set(false);
            state.show_progress.set(false);
        }

        PakResult::ExtractDone {
            success,
            message,
            file_count,
            dest,
        } => {
            state.progress.set(1.0);

            if success {
                state.add_result(&format!(
                    "✅ Successfully extracted {} files to {}",
                    file_count, dest
                ));
            } else {
                state.add_result(&format!("❌ Extraction failed: {}", message));
            }
            state.add_result("------------------------------------------------------------");

            state.is_extracting.set(false);
            state.show_progress.set(false);
        }

        PakResult::CreateDone {
            success,
            message,
            pak_name,
        } => {
            state.progress.set(1.0);

            if success {
                state.add_result(&format!("✅ Successfully created {}", pak_name));
            } else {
                state.add_result(&format!("❌ PAK creation failed: {}", message));
            }
            state.add_result("------------------------------------------------------------");

            state.is_creating.set(false);
            state.show_progress.set(false);
        }

        PakResult::ValidateDone {
            valid,
            structure,
            warnings,
        } => {
            if valid {
                state.add_result("✓ Mod structure is valid!");
            } else {
                state.add_result("⚠ Mod structure has issues:");
            }

            if !structure.is_empty() {
                state.add_result("Structure found:");
                for item in &structure {
                    state.add_result(&format!("  {}", item));
                }
            }

            if !warnings.is_empty() {
                state.add_result("Warnings:");
                for warning in &warnings {
                    state.add_result(&format!("  - {}", warning));
                }
            }

            state.add_result("------------------------------------------------------------");
            state.is_validating.set(false);
        }

        PakResult::BatchExtractDone {
            success_count,
            fail_count,
            results,
            dest,
        } => {
            state.progress.set(1.0);

            state.add_result(&format!(
                "Batch extraction complete: {} succeeded, {} failed",
                success_count, fail_count
            ));
            state.add_result(&format!("Destination: {}", dest));
            state.add_result("------------------------------------------------------------");

            for result in &results {
                state.add_result(result);
            }

            state.add_result("------------------------------------------------------------");
            state.is_extracting.set(false);
            state.show_progress.set(false);
        }

        PakResult::BatchCreateDone {
            success_count,
            fail_count,
            results,
            dest,
        } => {
            state.progress.set(1.0);

            state.add_result(&format!(
                "Batch creation complete: {} succeeded, {} failed",
                success_count, fail_count
            ));
            state.add_result(&format!("Destination: {}", dest));
            state.add_result("------------------------------------------------------------");

            for result in &results {
                state.add_result(result);
            }

            state.add_result("------------------------------------------------------------");
            state.is_creating.set(false);
            state.show_progress.set(false);
        }
    }
}
