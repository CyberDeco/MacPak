//! Batch PAK operations

use std::thread;

use floem::prelude::*;
use maclarian::pak::{batch_create, batch_extract, find_packable_folders, find_pak_files};

use super::super::types::{PakResult, create_result_sender, get_shared_progress};
use crate::gui::state::{ActiveDialog, PakOpsState};

/// Batch extract multiple PAK files from a folder (recursively)
pub fn batch_extract_paks(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new().set_title("Select Folder Containing PAK Files");

    let dialog = if let Some(dir) = state.working_dir.get() {
        dialog.set_directory(&dir)
    } else {
        dialog
    };

    let Some(source_dir) = dialog.pick_folder() else {
        return;
    };

    // Update working directory
    state
        .working_dir
        .set(Some(source_dir.to_string_lossy().to_string()));

    // Find all .pak files using maclarian's utility
    let pak_files = find_pak_files(&source_dir);

    if pak_files.is_empty() {
        state.add_result("No .pak files found in the selected folder");
        return;
    }

    state.add_result(&format!("Found {} PAK files to extract", pak_files.len()));

    // Ask for destination folder
    let dest_dialog = rfd::FileDialog::new()
        .set_title("Select Destination Folder for Extracted Files")
        .set_directory(&source_dir);

    let Some(dest_dir) = dest_dialog.pick_folder() else {
        return;
    };

    let pak_count = pak_files.len();

    state.is_extracting.set(true);
    state.active_dialog.set(ActiveDialog::Progress);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Batch extracting {} PAK files...", pak_count));

    // Reset shared progress state
    get_shared_progress().reset();

    let send = create_result_sender(state.clone());
    let shared_progress = get_shared_progress();

    thread::spawn(move || {
        let result = batch_extract(
            &pak_files,
            &source_dir,
            &dest_dir,
            |progress: &maclarian::pak::PakProgress| {
                let desc = progress
                    .current_file
                    .as_deref()
                    .unwrap_or(progress.phase.as_str());
                shared_progress.update(progress.current, progress.total, desc);
            },
        );

        send(PakResult::BatchExtractDone {
            success_count: result.success_count,
            fail_count: result.fail_count,
            results: result.results,
            dest: dest_dir.to_string_lossy().to_string(),
        });
    });
}

/// Batch create PAK files from subfolders (recursively finds all packable folders)
pub fn batch_create_paks(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new().set_title("Select Folder Containing Subfolders to Pack");

    let dialog = if let Some(dir) = state.working_dir.get() {
        dialog.set_directory(&dir)
    } else {
        dialog
    };

    let Some(source_dir) = dialog.pick_folder() else {
        return;
    };

    // Update working directory
    state
        .working_dir
        .set(Some(source_dir.to_string_lossy().to_string()));

    // Find all packable folders using maclarian's utility
    let folders = find_packable_folders(&source_dir);

    if folders.is_empty() {
        state.add_result("No packable folders found (folders must contain files)");
        return;
    }

    state.add_result(&format!("Found {} folders to pack", folders.len()));

    // Ask for destination folder for PAK files
    let dest_dialog = rfd::FileDialog::new()
        .set_title("Select Destination Folder for PAK Files")
        .set_directory(&source_dir);

    let Some(dest_dir) = dest_dialog.pick_folder() else {
        return;
    };

    let folder_count = folders.len();

    state.is_creating.set(true);
    state.active_dialog.set(ActiveDialog::Progress);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Batch creating {} PAK files...", folder_count));

    // Reset shared progress state
    get_shared_progress().reset();

    let send = create_result_sender(state.clone());
    let shared_progress = get_shared_progress();

    thread::spawn(move || {
        let result = batch_create(
            &folders,
            &source_dir,
            &dest_dir,
            |progress: &maclarian::pak::PakProgress| {
                let desc = progress
                    .current_file
                    .as_deref()
                    .unwrap_or(progress.phase.as_str());
                shared_progress.update(progress.current, progress.total, desc);
            },
        );

        send(PakResult::BatchCreateDone {
            success_count: result.success_count,
            fail_count: result.fail_count,
            results: result.results,
            dest: dest_dir.to_string_lossy().to_string(),
        });
    });
}
