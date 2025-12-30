//! File operations for PAK handling

use floem::prelude::*;
use std::path::Path;
use std::thread;
use walkdir::WalkDir;

use crate::state::PakOpsState;
use super::types::{create_progress_sender, create_result_sender, get_shared_progress, PakResult};

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
    state.show_progress.set(true);
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
        let result = MacLarian::pak::PakOperations::list_with_progress(
            &pak_path,
            &|current, total, description| {
                progress_sender(current, total, description);
            },
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

/// Extract individual files (placeholder)
pub fn extract_individual_files(state: PakOpsState) {
    state.clear_results();
    state.add_result("📄 Extract Individual Files");
    state.add_result("This feature allows selecting specific files from a PAK.");
    state.add_result("Use 'List PAK Contents' first, then extract the full PAK.");
    state.add_result("Individual file extraction UI coming soon.");
    state.add_result("------------------------------------------------------------");
}

/// Create a PAK file from a folder via file dialog
pub fn create_pak_file(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new().set_title("Select Folder to Pack");

    let dialog = if let Some(dir) = state.working_dir.get() {
        dialog.set_directory(&dir)
    } else {
        dialog
    };

    let Some(source_dir) = dialog.pick_folder() else {
        return;
    };

    state
        .working_dir
        .set(Some(source_dir.to_string_lossy().to_string()));

    let suggested_name = format!(
        "{}.pak",
        source_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string())
    );

    let save_dialog = rfd::FileDialog::new()
        .set_title("Save PAK File As")
        .set_file_name(&suggested_name)
        .add_filter("PAK Files", &["pak"]);

    let save_dialog = if let Some(parent) = source_dir.parent() {
        save_dialog.set_directory(parent)
    } else {
        save_dialog
    };

    let Some(pak_file) = save_dialog.save_file() else {
        return;
    };

    // Store pending operation and show options dialog
    state.pending_create.set(Some((
        source_dir.to_string_lossy().to_string(),
        pak_file.to_string_lossy().to_string(),
    )));
    state.show_create_options.set(true);
}

/// Rebuild a modified PAK file
pub fn rebuild_pak_file(state: PakOpsState) {
    state.clear_results();

    let dialog = rfd::FileDialog::new().set_title("Select Modified PAK Folder");

    let dialog = if let Some(dir) = state.working_dir.get() {
        dialog.set_directory(&dir)
    } else {
        dialog
    };

    let Some(source_dir) = dialog.pick_folder() else {
        return;
    };

    state
        .working_dir
        .set(Some(source_dir.to_string_lossy().to_string()));

    let suggested_name = format!(
        "{}_rebuilt.pak",
        source_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string())
    );

    let save_dialog = rfd::FileDialog::new()
        .set_title("Save Rebuilt PAK As")
        .set_file_name(&suggested_name)
        .add_filter("PAK Files", &["pak"]);

    let save_dialog = if let Some(parent) = source_dir.parent() {
        save_dialog.set_directory(parent)
    } else {
        save_dialog
    };

    let Some(pak_file) = save_dialog.save_file() else {
        return;
    };

    // Store pending operation and show options dialog
    state.pending_create.set(Some((
        source_dir.to_string_lossy().to_string(),
        pak_file.to_string_lossy().to_string(),
    )));
    state.show_create_options.set(true);
}

/// Execute the actual PAK creation after options are set
pub fn execute_create_pak(state: PakOpsState, source: String, dest: String) {
    let compression = state.compression.get();
    let priority = state.priority.get();

    let source_name = Path::new(&source)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let pak_name = Path::new(&dest)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Creating PAK from {}...", source_name));
    state.add_result(&format!(
        "Compression: {}, Priority: {}",
        compression.as_str(),
        priority
    ));

    state.is_creating.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Creating {} (this may take a while)...", pak_name));
    state.pending_create.set(None);

    let send = create_result_sender(state);

    let pak_name_clone = pak_name.clone();

    thread::spawn(move || {
        // Collect source files for listing
        let source_path = std::path::Path::new(&source);
        let files: Vec<String> = walkdir::WalkDir::new(&source)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.path().strip_prefix(source_path).ok().map(|p| p.to_string_lossy().to_string()))
            .collect();

        // TODO: Use compression and priority options when MacLarian supports them
        let result = MacLarian::pak::PakOperations::create(&source, &dest);

        let pak_result = match result {
            Ok(_) => PakResult::CreateDone {
                success: true,
                message: String::new(),
                files,
                pak_name: pak_name_clone,
            },
            Err(e) => PakResult::CreateDone {
                success: false,
                message: e.to_string(),
                files: Vec::new(),
                pak_name: pak_name_clone,
            },
        };

        send(pak_result);
    });
}

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
        // Basic structure validation
        let mut valid = true;
        let mut structure = Vec::new();
        let mut warnings = Vec::new();

        // Check for common mod directories
        let expected_dirs = ["Mods", "Public", "Localization"];
        for dir_name in expected_dirs {
            let dir_path = Path::new(&mod_path).join(dir_name);
            if dir_path.exists() {
                structure.push(format!("+ {}/", dir_name));
            }
        }

        // Check for meta.lsx
        let meta_paths = [
            Path::new(&mod_path).join("Mods"),
            Path::new(&mod_path).to_path_buf(),
        ];

        let mut found_meta = false;
        for base in meta_paths {
            if base.exists() && base.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&base) {
                    for entry in entries.flatten() {
                        let meta_path = entry.path().join("meta.lsx");
                        if meta_path.exists() {
                            found_meta = true;
                            structure.push(format!(
                                "+ {}/meta.lsx",
                                entry.file_name().to_string_lossy()
                            ));
                        }
                    }
                }
            }
        }

        if !found_meta {
            warnings.push("No meta.lsx found - mod may not load properly".to_string());
            valid = false;
        }

        if structure.is_empty() {
            warnings
                .push("No standard mod directories found (Mods/, Public/, Localization/)".to_string());
            valid = false;
        }

        send(PakResult::ValidateDone {
            valid,
            structure,
            warnings,
        });
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
    let progress = create_progress_sender(state.clone());

    thread::spawn(move || {
        let mut success_count = 0;
        let mut fail_count = 0;
        let mut results = Vec::new();
        let source_base = Path::new(&source_path);

        for (i, pak_path) in pak_files.iter().enumerate() {
            let pak_name = pak_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Calculate relative path for display and output structure
            let relative_path = pak_path
                .strip_prefix(source_base)
                .unwrap_or(pak_path.as_path());
            let display_path = relative_path.to_string_lossy();

            // Update progress
            progress(i + 1, pak_count, &display_path);

            // Preserve directory structure: create subfolder matching relative path
            // e.g., source/subdir/file.pak -> dest/subdir/file/
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
                results.push(format!("Failed to create folder for {}: {}", display_path, e));
                fail_count += 1;
                continue;
            }

            let pak_str = pak_path.to_string_lossy().to_string();
            let dest_str = pak_dest.to_string_lossy().to_string();

            match MacLarian::pak::PakOperations::extract(&pak_str, &dest_str) {
                Ok(_) => {
                    results.push(format!("Extracted: {}", display_path));
                    success_count += 1;
                }
                Err(e) => {
                    results.push(format!("Failed {}: {}", display_path, e));
                    fail_count += 1;
                }
            }
        }

        send(PakResult::BatchExtractDone {
            success_count,
            fail_count,
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
    let progress = create_progress_sender(state.clone());

    thread::spawn(move || {
        let mut success_count = 0;
        let mut fail_count = 0;
        let mut results = Vec::new();
        let source_base = Path::new(&source_path);

        for (i, folder_path) in folders.iter().enumerate() {
            let folder_name = folder_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Calculate relative path for display and output structure
            let relative_path = folder_path
                .strip_prefix(source_base)
                .unwrap_or(folder_path.as_path());
            let display_path = relative_path.to_string_lossy();

            // Update progress
            progress(i + 1, folder_count, &display_path);

            // Preserve directory structure: create PAK in matching relative path
            // e.g., source/subdir/folder -> dest/subdir/folder.pak
            let relative_parent = relative_path.parent().unwrap_or(Path::new(""));
            let pak_dest_dir = Path::new(&dest_path).join(relative_parent);

            // Create parent directories if needed
            if !pak_dest_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&pak_dest_dir) {
                    results.push(format!("Failed to create dir for {}: {}", display_path, e));
                    fail_count += 1;
                    continue;
                }
            }

            let pak_path = pak_dest_dir.join(format!("{}.pak", folder_name));
            let folder_str = folder_path.to_string_lossy().to_string();
            let pak_str = pak_path.to_string_lossy().to_string();

            match MacLarian::pak::PakOperations::create(&folder_str, &pak_str) {
                Ok(_) => {
                    results.push(format!("Created: {}.pak", display_path));
                    success_count += 1;
                }
                Err(e) => {
                    results.push(format!("Failed {}: {}", display_path, e));
                    fail_count += 1;
                }
            }
        }

        send(PakResult::BatchCreateDone {
            success_count,
            fail_count,
            results,
            dest: dest_path,
        });
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
        let result = MacLarian::pak::PakOperations::list_with_progress(
            &pak_path,
            &|current, total, description| {
                progress_sender(current, total, description);
            },
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
