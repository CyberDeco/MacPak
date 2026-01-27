//! Mod structure validation

use std::path::Path;
use std::thread;

use floem::prelude::*;

use super::super::types::{create_result_sender, get_shared_progress, PakResult};
use crate::gui::state::{ActiveDialog, PakOpsState};

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

    get_shared_progress().reset();
    let shared_progress = get_shared_progress();
    let send = create_result_sender(state);

    thread::spawn(move || {
        // Use maclarian's validation with progress
        let result = maclarian::mods::validate_mod_structure_with_progress(
            Path::new(&mod_path),
            &|p| {
                let desc = p.current_file.as_deref().unwrap_or(p.phase.as_str());
                shared_progress.update(p.current, p.total, desc);
            },
        );

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

    get_shared_progress().reset();
    let shared_progress = get_shared_progress();
    let send = create_result_sender(state);

    thread::spawn(move || {
        // Use maclarian's validation with progress
        let result = maclarian::mods::validate_mod_structure_with_progress(
            Path::new(&folder_path),
            &|p| {
                let desc = p.current_file.as_deref().unwrap_or(p.phase.as_str());
                shared_progress.update(p.current, p.total, desc);
            },
        );

        send(PakResult::ValidateDone {
            valid: result.valid,
            structure: result.structure,
            warnings: result.warnings,
        });
    });
}

/// Validate mod structure in a PAK file
pub fn validate_pak_mod_structure(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new()
        .set_title("Select PAK File to Validate")
        .add_filter("PAK Files", &["pak"]);

    let dialog = if let Some(dir) = state.working_dir.get() {
        dialog.set_directory(&dir)
    } else {
        dialog
    };

    let Some(pak_file) = dialog.pick_file() else {
        return;
    };

    if let Some(parent) = pak_file.parent() {
        state
            .working_dir
            .set(Some(parent.to_string_lossy().to_string()));
    }

    let pak_name = pak_file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Validating mod structure in: {}", pak_name));
    state.is_validating.set(true);

    get_shared_progress().reset();
    let shared_progress = get_shared_progress();
    let pak_path = pak_file.to_string_lossy().to_string();
    let send = create_result_sender(state);

    thread::spawn(move || {
        match maclarian::mods::validate_pak_mod_structure_with_progress(
            Path::new(&pak_path),
            &|p| {
                let desc = p.current_file.as_deref().unwrap_or(p.phase.as_str());
                shared_progress.update(p.current, p.total, desc);
            },
        ) {
            Ok(result) => {
                send(PakResult::ValidateDone {
                    valid: result.valid,
                    structure: result.structure,
                    warnings: result.warnings,
                });
            }
            Err(e) => {
                send(PakResult::ValidateDone {
                    valid: false,
                    structure: Vec::new(),
                    warnings: vec![format!("Failed to read PAK: {}", e)],
                });
            }
        }
    });
}

/// Validate mod structure in a dropped PAK file
pub fn validate_dropped_pak(state: PakOpsState, pak_path: String) {
    state.clear_results();

    let pak_name = Path::new(&pak_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Validating mod structure in: {}", pak_name));
    state.is_validating.set(true);
    state.dropped_file.set(None);
    state.active_dialog.set(ActiveDialog::None);

    get_shared_progress().reset();
    let shared_progress = get_shared_progress();
    let send = create_result_sender(state);

    thread::spawn(move || {
        match maclarian::mods::validate_pak_mod_structure_with_progress(
            Path::new(&pak_path),
            &|p| {
                let desc = p.current_file.as_deref().unwrap_or(p.phase.as_str());
                shared_progress.update(p.current, p.total, desc);
            },
        ) {
            Ok(result) => {
                send(PakResult::ValidateDone {
                    valid: result.valid,
                    structure: result.structure,
                    warnings: result.warnings,
                });
            }
            Err(e) => {
                send(PakResult::ValidateDone {
                    valid: false,
                    structure: Vec::new(),
                    warnings: vec![format!("Failed to read PAK: {}", e)],
                });
            }
        }
    });
}
