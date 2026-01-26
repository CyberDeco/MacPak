//! Types and result handling for PAK operations

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use maclarian::pak::PakProgress;
use crate::gui::state::{ActiveDialog, PakOpsState};

/// Result type for background PAK operations
pub enum PakResult {
    ExtractDone {
        success: bool,
        message: String,
        files: Vec<String>,
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
        files: Vec<String>,
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
    /// File list loaded for individual file selection
    FileSelectLoaded {
        success: bool,
        files: Vec<String>,
        pak_path: String,
        error: Option<String>,
    },
    /// Individual files extracted
    IndividualExtractDone {
        success: bool,
        message: String,
        files: Vec<String>,
        dest: String,
    },
}

/// Shared progress state that can be updated from background threads
/// and polled from the UI thread
#[derive(Clone)]
pub struct SharedProgress {
    /// Progress as integer percentage (0-100), stored as u32 for atomic access
    pub progress_pct: Arc<AtomicU32>,
    /// Current item index (1-based for display)
    pub current: Arc<AtomicU32>,
    /// Total items count
    pub total: Arc<AtomicU32>,
    /// Current progress message
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

    /// Update progress from background thread (lock-free for percentage)
    pub fn update(&self, current: usize, total: usize, description: &str) {
        let pct = if total > 0 {
            ((current as f64 / total as f64) * 100.0) as u32
        } else {
            0
        };
        self.progress_pct.store(pct, Ordering::SeqCst);
        self.current.store(current as u32, Ordering::SeqCst);
        self.total.store(total as u32, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            *msg = description.to_string();
        }
    }

    /// Get current progress percentage (0-100)
    pub fn get_pct(&self) -> u32 {
        self.progress_pct.load(Ordering::SeqCst)
    }

    /// Get current and total counts
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

    /// Reset progress to initial state (call when starting a new operation)
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

/// Global shared progress instance
static SHARED_PROGRESS: std::sync::OnceLock<SharedProgress> = std::sync::OnceLock::new();

/// Get or create the global shared progress instance
pub fn get_shared_progress() -> &'static SharedProgress {
    SHARED_PROGRESS.get_or_init(SharedProgress::new)
}

/// Global scope for ext_action callbacks - reused to prevent scope accumulation
static EXT_ACTION_SCOPE: std::sync::OnceLock<Scope> = std::sync::OnceLock::new();

/// Get or create the global scope for ext_action calls
fn get_ext_action_scope() -> Scope {
    *EXT_ACTION_SCOPE.get_or_init(Scope::new)
}

/// Create a sender for background operations that updates UI on the main thread
pub fn create_result_sender(state: PakOpsState) -> impl FnOnce(PakResult) {
    create_ext_action(get_ext_action_scope(), move |result| {
        handle_pak_result(state, result);
    })
}

/// Create a reusable sender for progress updates that uses shared atomic state
pub fn create_progress_sender(_state: PakOpsState) -> impl Fn(&PakProgress) + Send + Sync + Clone {
    let shared = get_shared_progress().clone();

    move |progress: &PakProgress| {
        let description = progress.current_file.as_deref().unwrap_or(progress.phase.as_str());
        shared.update(progress.current, progress.total, description);
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
                // Clear results and batch add all files
                state.clear_results();
                state.add_results_batch(files.clone());
                state.status_message.set(format!("Loaded {} ({} files)", pak_name, files.len()));
            } else {
                state.status_message.set(format!("Failed: {}", error.unwrap_or_default()));
            }

            state.is_listing.set(false);
            state.active_dialog.set(ActiveDialog::None);
        }

        PakResult::ExtractDone {
            success,
            message,
            files,
            dest,
        } => {
            state.progress.set(1.0);

            if success {
                // Clear results and batch add extracted files
                state.clear_results();
                state.add_results_batch(files.clone());
                state.status_message.set(format!("Extracted {} files to {}", files.len(), dest));
            } else {
                state.status_message.set(format!("Extraction failed: {}", message));
            }

            state.is_extracting.set(false);
            state.active_dialog.set(ActiveDialog::None);
        }

        PakResult::CreateDone {
            success,
            message,
            files,
            pak_name,
        } => {
            state.progress.set(1.0);

            if success {
                // Clear results and batch add source files
                state.clear_results();
                state.add_results_batch(files.clone());
                state.status_message.set(format!("Created {} ({} files)", pak_name, files.len()));
            } else {
                state.status_message.set(format!("Creation failed: {}", message));
            }

            state.is_creating.set(false);
            state.active_dialog.set(ActiveDialog::None);
        }

        PakResult::ValidateDone {
            valid,
            structure,
            warnings,
        } => {
            let mut results = Vec::new();
            if valid {
                results.push("✓ Mod structure is valid!".to_string());
            } else {
                results.push("⚠ Mod structure has issues:".to_string());
            }

            if !structure.is_empty() {
                results.push("Structure found:".to_string());
                for item in &structure {
                    results.push(format!("  {}", item));
                }
            }

            if !warnings.is_empty() {
                results.push("Warnings:".to_string());
                for warning in &warnings {
                    results.push(format!("  - {}", warning));
                }
            }

            results.push("------------------------------------------------------------".to_string());
            state.add_results_batch(results);
            state.is_validating.set(false);
        }

        PakResult::BatchExtractDone {
            success_count,
            fail_count,
            results,
            dest,
        } => {
            state.progress.set(1.0);

            let mut all_results = vec![
                format!("Batch extraction complete: {} succeeded, {} failed", success_count, fail_count),
                format!("Destination: {}", dest),
                "------------------------------------------------------------".to_string(),
            ];
            all_results.extend(results);
            all_results.push("------------------------------------------------------------".to_string());

            state.add_results_batch(all_results);
            state.is_extracting.set(false);
            state.active_dialog.set(ActiveDialog::None);
        }

        PakResult::BatchCreateDone {
            success_count,
            fail_count,
            results,
            dest,
        } => {
            state.progress.set(1.0);

            let mut all_results = vec![
                format!("Batch creation complete: {} succeeded, {} failed", success_count, fail_count),
                format!("Destination: {}", dest),
                "------------------------------------------------------------".to_string(),
            ];
            all_results.extend(results);
            all_results.push("------------------------------------------------------------".to_string());

            state.add_results_batch(all_results);
            state.is_creating.set(false);
            state.active_dialog.set(ActiveDialog::None);
        }

        PakResult::FileSelectLoaded {
            success,
            files,
            pak_path,
            error,
        } => {
            state.is_listing.set(false);
            state.active_dialog.set(ActiveDialog::None);

            if success {
                state.file_select_pak.set(Some(pak_path));
                state.file_select_list.set(files);
                state.file_select_selected.set(std::collections::HashSet::new());
                state.file_select_filter.set(String::new());
                state.active_dialog.set(ActiveDialog::FileSelect);
            } else {
                state.status_message.set(format!("Failed to load PAK: {}", error.unwrap_or_default()));
            }
        }

        PakResult::IndividualExtractDone {
            success,
            message,
            files,
            dest,
        } => {
            state.progress.set(1.0);
            state.is_extracting.set(false);
            state.active_dialog.set(ActiveDialog::None);

            if success {
                state.clear_results();
                state.add_results_batch(files.clone());
                state.status_message.set(format!("Extracted {} files to {}", files.len(), dest));
            } else {
                state.status_message.set(format!("Extraction failed: {}", message));
            }
        }
    }
}
