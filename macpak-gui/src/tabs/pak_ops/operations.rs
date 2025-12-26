//! File operations for PAK handling

use floem::prelude::*;
use std::path::Path;
use std::thread;

use crate::state::PakOpsState;
use super::types::{create_result_sender, PakResult};

/// Extract a PAK file via file dialog
pub fn extract_pak_file(state: PakOpsState) {
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
        .set(format!("Extracting {} (this may take a while)...", pak_name));

    let pak_path = pak_file.to_string_lossy().to_string();
    let dest_path = dest_dir.to_string_lossy().to_string();

    let send = create_result_sender(state);

    thread::spawn(move || {
        // Get file count from PAK before extraction
        let file_count = MacLarian::pak::PakOperations::list(&pak_path)
            .map(|files| files.len())
            .unwrap_or(0);

        let result = MacLarian::pak::PakOperations::extract(&pak_path, &dest_path);

        let pak_result = match result {
            Ok(_) => PakResult::ExtractDone {
                success: true,
                message: String::new(),
                file_count,
                dest: dest_path,
            },
            Err(e) => PakResult::ExtractDone {
                success: false,
                message: e.to_string(),
                file_count: 0,
                dest: dest_path,
            },
        };

        send(pak_result);
    });
}

/// List contents of a PAK file via file dialog
pub fn list_pak_contents(state: PakOpsState) {
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
    state.add_result("(Large PAK files may take a moment to read)");
    state.is_listing.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Reading {} (this may take a while)...", pak_name));

    let pak_path = pak_file.to_string_lossy().to_string();
    let pak_name_clone = pak_name.clone();

    let send = create_result_sender(state);

    thread::spawn(move || {
        let result = MacLarian::pak::PakOperations::list(&pak_path);

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
    state.add_result("📄 Extract Individual Files");
    state.add_result("This feature allows selecting specific files from a PAK.");
    state.add_result("Use 'List PAK Contents' first, then extract the full PAK.");
    state.add_result("Individual file extraction UI coming soon.");
    state.add_result("------------------------------------------------------------");
}

/// Create a PAK file from a folder via file dialog
pub fn create_pak_file(state: PakOpsState) {
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
        // TODO: Use compression and priority options when MacLarian supports them
        let result = MacLarian::pak::PakOperations::create(&source, &dest);

        let pak_result = match result {
            Ok(_) => PakResult::CreateDone {
                success: true,
                message: String::new(),
                pak_name: pak_name_clone,
            },
            Err(e) => PakResult::CreateDone {
                success: false,
                message: e.to_string(),
                pak_name: pak_name_clone,
            },
        };

        send(pak_result);
    });
}

/// Validate mod structure
pub fn validate_mod_structure(state: PakOpsState) {
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
        .set(format!("Extracting {} (this may take a while)...", pak_name));
    state.dropped_file.set(None);

    let send = create_result_sender(state);

    thread::spawn(move || {
        // Get file count from PAK before extraction
        let file_count = MacLarian::pak::PakOperations::list(&pak_path)
            .map(|files| files.len())
            .unwrap_or(0);

        let result = MacLarian::pak::PakOperations::extract(&pak_path, &dest_path);

        let pak_result = match result {
            Ok(_) => PakResult::ExtractDone {
                success: true,
                message: String::new(),
                file_count,
                dest: dest_path,
            },
            Err(e) => PakResult::ExtractDone {
                success: false,
                message: e.to_string(),
                file_count: 0,
                dest: dest_path,
            },
        };

        send(pak_result);
    });
}

/// List contents of a dropped PAK file
pub fn list_dropped_file(state: PakOpsState, pak_path: String) {
    let pak_name = Path::new(&pak_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Listing contents of {}...", pak_name));
    state.add_result("(Large PAK files may take a moment to read)");
    state.is_listing.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Reading {} (this may take a while)...", pak_name));
    state.dropped_file.set(None);

    let send = create_result_sender(state);

    thread::spawn(move || {
        let result = MacLarian::pak::PakOperations::list(&pak_path);

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
