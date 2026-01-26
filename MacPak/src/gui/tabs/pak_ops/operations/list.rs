//! PAK content listing operations

use floem::prelude::*;
use std::path::Path;
use std::thread;

use crate::gui::state::{ActiveDialog, PakOpsState};
use super::super::types::{create_progress_sender, create_result_sender, get_shared_progress, PakResult};

/// List contents of a PAK file via file dialog
pub fn list_pak_contents(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new()
        .set_title("Select PAK File to List")
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

    state.add_result(&format!("Listing contents of {}...", pak_name));
    state.is_listing.set(true);
    state.active_dialog.set(ActiveDialog::Progress);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Reading {}...", pak_name));

    // Reset shared progress state
    get_shared_progress().reset();

    let pak_path = pak_file.to_string_lossy().to_string();
    let pak_name_clone = pak_name.clone();

    let send = create_result_sender(state.clone());
    let progress_sender = create_progress_sender(state);

    thread::spawn(move || {
        let result = maclarian::pak::PakOperations::list_with_progress(
            &pak_path,
            &progress_sender,
        );

        let pak_result = match result {
            Ok(files) => PakResult::ListDone {
                success: true,
                files,
                pak_name: pak_name_clone,
                error: None,
            },
            Err(e) => PakResult::ListDone {
                success: false,
                files: Vec::new(),
                pak_name: pak_name_clone,
                error: Some(e.to_string()),
            },
        };

        send(pak_result);
    });
}

/// List contents of a dropped PAK file
pub fn list_dropped_file(state: PakOpsState, pak_path: String) {
    state.clear_results();

    let pak_name = Path::new(&pak_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Listing contents of {}...", pak_name));
    state.is_listing.set(true);
    state.active_dialog.set(ActiveDialog::Progress);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Reading {}...", pak_name));
    state.dropped_file.set(None);

    // Reset shared progress state
    get_shared_progress().reset();

    let send = create_result_sender(state.clone());
    let progress_sender = create_progress_sender(state);

    thread::spawn(move || {
        let result = maclarian::pak::PakOperations::list_with_progress(
            &pak_path,
            &progress_sender,
        );

        let pak_result = match result {
            Ok(files) => PakResult::ListDone {
                success: true,
                files,
                pak_name,
                error: None,
            },
            Err(e) => PakResult::ListDone {
                success: false,
                files: Vec::new(),
                pak_name,
                error: Some(e.to_string()),
            },
        };

        send(pak_result);
    });
}
