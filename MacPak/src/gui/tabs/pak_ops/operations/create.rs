//! PAK creation and rebuild operations

use floem::prelude::*;
use std::path::Path;
use std::thread;

use crate::gui::state::PakOpsState;
use super::super::types::{create_result_sender, PakResult};

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
    let generate_info_json = state.generate_info_json.get();

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
    if generate_info_json {
        state.add_result("Will generate info.json for BaldursModManager");
    }

    state.is_creating.set(true);
    state.show_progress.set(true);
    state.progress.set(0.0);
    state
        .progress_message
        .set(format!("Creating {} (this may take a while)...", pak_name));
    state.pending_create.set(None);

    let send = create_result_sender(state);

    let pak_name_clone = pak_name.clone();

    // Convert GUI compression enum to MacLarian compression enum
    let mac_compression = match compression {
        crate::gui::state::PakCompression::Lz4Hc => MacLarian::pak::CompressionMethod::Lz4, // LZ4 HC not yet supported, fall back to LZ4
        crate::gui::state::PakCompression::Lz4 => MacLarian::pak::CompressionMethod::Lz4,
        crate::gui::state::PakCompression::None => MacLarian::pak::CompressionMethod::None,
    };

    thread::spawn(move || {
        // Collect source files for listing
        let source_path = std::path::Path::new(&source);
        let files: Vec<String> = walkdir::WalkDir::new(&source)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.path().strip_prefix(source_path).ok().map(|p| p.to_string_lossy().to_string()))
            .collect();

        let result = MacLarian::pak::PakOperations::create_with_compression(&source, &dest, mac_compression);

        let pak_result = match result {
            Ok(_) => {
                let mut info_json_result = None;

                // Generate info.json if requested
                if generate_info_json {
                    info_json_result = Some(generate_info_json_file(&source, &dest));
                }

                PakResult::CreateDone {
                    success: true,
                    message: info_json_result.unwrap_or_default(),
                    files,
                    pak_name: pak_name_clone,
                }
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

/// Generate info.json file for BaldursModManager compatibility
fn generate_info_json_file(source_dir: &str, pak_path: &str) -> String {
    // Use MacLarian's info.json generation
    let result = MacLarian::mods::generate_info_json(source_dir, pak_path);

    if !result.success {
        return format!("Warning: {}, skipped info.json generation", result.message);
    }

    let Some(info_json) = result.content else {
        return "Warning: Failed to generate info.json content".to_string();
    };

    // Write info.json next to the PAK file (don't move the PAK - user already chose location)
    let pak_path = Path::new(pak_path);
    let info_json_path = pak_path.parent()
        .map(|p| p.join("info.json"))
        .unwrap_or_else(|| Path::new("info.json").to_path_buf());

    match std::fs::write(&info_json_path, &info_json) {
        Ok(_) => format!("Generated {}", info_json_path.display()),
        Err(e) => format!("Warning: Failed to write info.json: {}", e),
    }
}
