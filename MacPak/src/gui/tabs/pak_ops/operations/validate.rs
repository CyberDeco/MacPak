//! Mod structure validation

use std::path::Path;
use std::thread;

use floem::prelude::*;

use crate::gui::state::PakOpsState;
use super::super::types::{create_result_sender, PakResult};

/// Validate mod structure
pub fn validate_mod_structure(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new().set_title("Select Mod Folder to Validate");

    let dialog = if let Some(dir) = state.working_dir.get() {
        dialog.set_directory(&dir)
    } else {
        dialog
    };

    let Some(mod_dir) = dialog.pick_folder() else {
        return;
    };

    let mod_name = mod_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Validating mod structure: {}", mod_name));
    state.is_validating.set(true);

    let mod_path = mod_dir.to_string_lossy().to_string();

    let send = create_result_sender(state);

    thread::spawn(move || {
        // Use maclarian's validation
        let result = maclarian::mods::validate_mod_structure(Path::new(&mod_path));

        send(PakResult::ValidateDone {
            valid: result.valid,
            structure: result.structure,
            warnings: result.warnings,
        });
    });
}

/// Validate a dropped folder's mod structure (skips folder picker)
pub fn validate_dropped_folder(state: PakOpsState, folder_path: String) {
    state.clear_results();

    let mod_name = Path::new(&folder_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Validating mod structure: {}", mod_name));
    state.is_validating.set(true);

    let send = create_result_sender(state);

    thread::spawn(move || {
        // Use maclarian's validation
        let result = maclarian::mods::validate_mod_structure(Path::new(&folder_path));

        send(PakResult::ValidateDone {
            valid: result.valid,
            structure: result.structure,
            warnings: result.warnings,
        });
    });
}
