//! Single file extraction operations

use floem::prelude::*;
use std::path::Path;
use std::thread;

use crate::gui::state::PakOpsState;
use super::super::types::{create_progress_sender, create_result_sender, get_shared_progress, PakResult};

/// Extract a PAK file via file dialog
pub fn extract_pak_file(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new()
        .set_title("Select PAK File to Extract")
        .add_filter("PAK Files", &["pak"]);

    let dialog = if let Some(dir) = state.working_dir.get() {
        dialog.set_directory(&dir)
    } else {
        dialog
    };

    let Some(pak_file) = dialog.pick_file() else {
        return;
    };

    // Update working directory
    if let Some(parent) = pak_file.parent() {
        state
            .working_dir
            .set(Some(parent.to_string_lossy().to_string()));
    }

    let dest_dialog = rfd::FileDialog::new()
        .set_title("Select Extraction Destination")
        .set_directory(pak_file.parent().unwrap_or(Path::new("/")));

    let Some(dest_dir) = dest_dialog.pick_folder() else {
        return;
    };

    let pak_name = pak_file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Extracting {}...", pak_name));
    state.is_extracting.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Reading {}...", pak_name));

    // Reset shared progress state
    get_shared_progress().reset();

    let pak_path = pak_file.to_string_lossy().to_string();
    let dest_path = dest_dir.to_string_lossy().to_string();

    let send = create_result_sender(state.clone());
    let progress_sender = create_progress_sender(state);

    thread::spawn(move || {
        let result = MacLarian::pak::PakOperations::extract_with_progress(
            &pak_path,
            &dest_path,
            &|current, total, description| {
                progress_sender(current, total, description);
            },
        );

        let files = MacLarian::pak::PakOperations::list(&pak_path)
            .unwrap_or_default();

        let pak_result = match result {
            Ok(_) => PakResult::ExtractDone {
                success: true,
                message: String::new(),
                files,
                dest: dest_path,
            },
            Err(e) => PakResult::ExtractDone {
                success: false,
                message: e.to_string(),
                files: Vec::new(),
                dest: dest_path,
            },
        };

        send(pak_result);
    });
}

/// Extract individual files - opens PAK and shows file selection dialog
pub fn extract_individual_files(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new()
        .set_title("Select PAK File")
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

    state.add_result(&format!("Loading contents of {}...", pak_name));
    state.is_listing.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state.progress_message.set(format!("Reading {}...", pak_name));

    get_shared_progress().reset();

    let pak_path = pak_file.to_string_lossy().to_string();

    let send = create_result_sender(state.clone());
    let progress_sender = create_progress_sender(state);

    thread::spawn(move || {
        let result = MacLarian::pak::PakOperations::list_with_progress(
            &pak_path,
            &|current, total, description| {
                progress_sender(current, total, description);
            },
        );

        let pak_result = match result {
            Ok(files) => PakResult::FileSelectLoaded {
                success: true,
                files,
                pak_path,
                error: None,
            },
            Err(e) => PakResult::FileSelectLoaded {
                success: false,
                files: Vec::new(),
                pak_path,
                error: Some(e.to_string()),
            },
        };

        send(pak_result);
    });
}

/// Execute extraction of selected individual files
pub fn execute_individual_extract(state: PakOpsState) {
    let Some(pak_path) = state.file_select_pak.get() else {
        return;
    };

    let selected: Vec<String> = state.file_select_selected.get().into_iter().collect();
    if selected.is_empty() {
        state.status_message.set("No files selected".to_string());
        return;
    }

    // Close the dialog
    state.show_file_select.set(false);

    // Ask for destination
    let dest_dialog = rfd::FileDialog::new()
        .set_title("Select Extraction Destination")
        .set_directory(Path::new(&pak_path).parent().unwrap_or(Path::new("/")));

    let Some(dest_dir) = dest_dialog.pick_folder() else {
        return;
    };

    let dest_path = dest_dir.to_string_lossy().to_string();

    state.clear_results();
    state.add_result(&format!("Extracting {} files...", selected.len()));
    state.is_extracting.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state.progress_message.set("Extracting...".to_string());

    get_shared_progress().reset();

    let send = create_result_sender(state.clone());
    let progress_sender = create_progress_sender(state);

    thread::spawn(move || {
        let result = MacLarian::pak::PakOperations::extract_files_with_progress(
            &pak_path,
            &dest_path,
            &selected,
            &|current, total, description| {
                progress_sender(current, total, description);
            },
        );

        let pak_result = match result {
            Ok(_) => PakResult::IndividualExtractDone {
                success: true,
                message: String::new(),
                files: selected,
                dest: dest_path,
            },
            Err(e) => PakResult::IndividualExtractDone {
                success: false,
                message: e.to_string(),
                files: Vec::new(),
                dest: dest_path,
            },
        };

        send(pak_result);
    });
}

/// Extract a dropped PAK file
pub fn extract_dropped_file(state: PakOpsState, pak_path: String) {
    state.clear_results();

    let pak_name = Path::new(&pak_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Ask for destination
    let dest_dialog = rfd::FileDialog::new()
        .set_title("Select Extraction Destination")
        .set_directory(Path::new(&pak_path).parent().unwrap_or(Path::new("/")));

    let Some(dest_dir) = dest_dialog.pick_folder() else {
        state.dropped_file.set(None);
        return;
    };

    let dest_path = dest_dir.to_string_lossy().to_string();

    state.add_result(&format!("Extracting {}...", pak_name));
    state.is_extracting.set(true);
    state.show_progress.set(true);
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
        let result = MacLarian::pak::PakOperations::extract_with_progress(
            &pak_path,
            &dest_path,
            &|current, total, description| {
                progress_sender(current, total, description);
            },
        );

        let files = MacLarian::pak::PakOperations::list(&pak_path)
            .unwrap_or_default();

        let pak_result = match result {
            Ok(_) => PakResult::ExtractDone {
                success: true,
                message: String::new(),
                files,
                dest: dest_path,
            },
            Err(e) => PakResult::ExtractDone {
                success: false,
                message: e.to_string(),
                files: Vec::new(),
                dest: dest_path,
            },
        };

        send(pak_result);
    });
}
