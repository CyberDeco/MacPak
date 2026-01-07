//! Batch PAK operations

use rayon::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use walkdir::WalkDir;

use floem::prelude::*;

use crate::gui::state::PakOpsState;
use super::super::types::{create_result_sender, get_shared_progress, PakResult};

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

    // Recursively find all .pak files
    let mut pak_files: Vec<_> = WalkDir::new(&source_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .extension()
                    .map(|ext| ext.to_ascii_lowercase() == "pak")
                    .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    pak_files.sort();

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

    let source_path = source_dir.to_string_lossy().to_string();
    let dest_path = dest_dir.to_string_lossy().to_string();
    let pak_count = pak_files.len();

    state.is_extracting.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Batch extracting {} PAK files...", pak_count));

    // Reset shared progress state
    get_shared_progress().reset();

    let send = create_result_sender(state.clone());

    thread::spawn(move || {
        let success_counter = AtomicUsize::new(0);
        let fail_counter = AtomicUsize::new(0);
        let processed = AtomicUsize::new(0);
        let source_base = Path::new(&source_path);
        let shared_progress = get_shared_progress();

        // Parallel PAK extraction
        let results: Vec<String> = pak_files
            .par_iter()
            .map(|pak_path| {
                // Calculate relative path for display and output structure
                let relative_path = pak_path
                    .strip_prefix(source_base)
                    .unwrap_or(pak_path.as_path());
                let display_path = relative_path.to_string_lossy();

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                shared_progress.update(current, pak_count, &display_path);

                // Preserve directory structure: create subfolder matching relative path
                let relative_parent = relative_path.parent().unwrap_or(Path::new(""));
                let pak_stem = pak_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let pak_dest = Path::new(&dest_path)
                    .join(relative_parent)
                    .join(&pak_stem);

                if let Err(e) = std::fs::create_dir_all(&pak_dest) {
                    fail_counter.fetch_add(1, Ordering::SeqCst);
                    return format!("Failed to create folder for {}: {}", display_path, e);
                }

                let pak_str = pak_path.to_string_lossy().to_string();
                let dest_str = pak_dest.to_string_lossy().to_string();

                match MacLarian::pak::PakOperations::extract(&pak_str, &dest_str) {
                    Ok(()) => {
                        success_counter.fetch_add(1, Ordering::SeqCst);
                        format!("Extracted: {}", display_path)
                    }
                    Err(e) => {
                        fail_counter.fetch_add(1, Ordering::SeqCst);
                        format!("Failed {}: {}", display_path, e)
                    }
                }
            })
            .collect();

        send(PakResult::BatchExtractDone {
            success_count: success_counter.load(Ordering::SeqCst),
            fail_count: fail_counter.load(Ordering::SeqCst),
            results,
            dest: dest_path,
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

    // Recursively find all directories that contain files (leaf directories with content)
    // We look for directories that have at least one file in them
    let mut folders: Vec<_> = WalkDir::new(&source_dir)
        .follow_links(true)
        .min_depth(1) // Skip the root directory itself
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if !path.is_dir() {
                return false;
            }
            // Check if this directory contains any files (not just subdirs)
            std::fs::read_dir(path)
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .any(|e| e.path().is_file())
                })
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    folders.sort();

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

    let source_path = source_dir.to_string_lossy().to_string();
    let dest_path = dest_dir.to_string_lossy().to_string();
    let folder_count = folders.len();

    state.is_creating.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Batch creating {} PAK files...", folder_count));

    // Reset shared progress state
    get_shared_progress().reset();

    let send = create_result_sender(state.clone());

    thread::spawn(move || {
        let success_counter = AtomicUsize::new(0);
        let fail_counter = AtomicUsize::new(0);
        let processed = AtomicUsize::new(0);
        let source_base = Path::new(&source_path);
        let shared_progress = get_shared_progress();

        // Parallel PAK creation
        let results: Vec<String> = folders
            .par_iter()
            .map(|folder_path| {
                let folder_name = folder_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Calculate relative path for display and output structure
                let relative_path = folder_path
                    .strip_prefix(source_base)
                    .unwrap_or(folder_path.as_path());
                let display_path = relative_path.to_string_lossy();

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                shared_progress.update(current, folder_count, &display_path);

                // Preserve directory structure: create PAK in matching relative path
                let relative_parent = relative_path.parent().unwrap_or(Path::new(""));
                let pak_dest_dir = Path::new(&dest_path).join(relative_parent);

                // Create parent directories if needed (idempotent)
                if let Err(e) = std::fs::create_dir_all(&pak_dest_dir) {
                    fail_counter.fetch_add(1, Ordering::SeqCst);
                    return format!("Failed to create dir for {}: {}", display_path, e);
                }

                let pak_path = pak_dest_dir.join(format!("{}.pak", folder_name));
                let folder_str = folder_path.to_string_lossy().to_string();
                let pak_str = pak_path.to_string_lossy().to_string();

                match MacLarian::pak::PakOperations::create(&folder_str, &pak_str) {
                    Ok(()) => {
                        success_counter.fetch_add(1, Ordering::SeqCst);
                        format!("Created: {}.pak", display_path)
                    }
                    Err(e) => {
                        fail_counter.fetch_add(1, Ordering::SeqCst);
                        format!("Failed {}: {}", display_path, e)
                    }
                }
            })
            .collect();

        send(PakResult::BatchCreateDone {
            success_count: success_counter.load(Ordering::SeqCst),
            fail_count: fail_counter.load(Ordering::SeqCst),
            results,
            dest: dest_path,
        });
    });
}
