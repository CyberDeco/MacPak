//! Types and result handling for PAK operations

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

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
}

/// Create a sender for background operations that updates UI on the main thread
pub fn create_result_sender(state: PakOpsState) -> impl FnOnce(PakResult) {
    create_ext_action(Scope::new(), move |result| {
        handle_pak_result(state, result);
    })
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
                state.add_result(&format!("✅ Found {} files in {}", files.len(), pak_name));
                state.add_result("------------------------------------------------------------");

                // Show first 20 files
                let max_display = 20;
                for file_path in files.iter().take(max_display) {
                    state.add_result(&format!("  {}", file_path));
                }

                if files.len() > max_display {
                    let remaining = files.len() - max_display;
                    state.add_result(&format!("  ... and {} more files", remaining));
                }

                state.add_result("------------------------------------------------------------");
                state.list_contents.set(files);
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
    }
}
