//! Single file extraction operations

use floem::prelude::*;
use std::path::Path;
use std::thread;

use crate::gui::state::{ActiveDialog, PakOpsState};
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
    state.active_dialog.set(ActiveDialog::Progress);
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
        let result = maclarian::pak::PakOperations::extract_with_progress(
            &pak_path,
            &dest_path,
            &progress_sender,
        );

        let files = maclarian::pak::PakOperations::list(&pak_path)
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
    state.active_dialog.set(ActiveDialog::Progress);
    state.progress.set(0.0);
    state.progress_message.set(format!("Reading {}...", pak_name));

    get_shared_progress().reset();

    let pak_path = pak_file.to_string_lossy().to_string();

    let send = create_result_sender(state.clone());
    let progress_sender = create_progress_sender(state);

    thread::spawn(move || {
        let result = maclarian::pak::PakOperations::list_with_progress(
            &pak_path,
            &progress_sender,
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

    // Capture GR2 options before closing dialog
    let extract_gr2 = state.gr2_extract_gr2.get();
    let convert_to_glb = state.gr2_convert_to_glb.get();
    let convert_to_gltf = state.gr2_convert_to_gltf.get();
    let extract_textures = state.gr2_extract_textures.get();
    let convert_to_png = state.gr2_convert_to_png.get();
    let game_data = state.game_data_path.get();

    // Check if any GR2 processing is enabled (beyond just extracting the GR2)
    let use_smart_extract = convert_to_glb || convert_to_gltf || extract_textures;

    // Close the dialog
    state.active_dialog.set(ActiveDialog::None);

    // Ask for destination
    let dest_dialog = rfd::FileDialog::new()
        .set_title("Select Extraction Destination")
        .set_directory(Path::new(&pak_path).parent().unwrap_or(Path::new("/")));

    let Some(dest_dir) = dest_dialog.pick_folder() else {
        return;
    };

    let dest_path = dest_dir.to_string_lossy().to_string();

    state.clear_results();

    if use_smart_extract {
        let gr2_count = selected.iter().filter(|f| f.to_lowercase().ends_with(".gr2")).count();
        state.add_result(&format!(
            "Extracting {} files ({} GR2 with processing)...",
            selected.len(),
            gr2_count
        ));
    } else {
        state.add_result(&format!("Extracting {} files...", selected.len()));
    }

    state.is_extracting.set(true);
    state.active_dialog.set(ActiveDialog::Progress);
    state.progress.set(0.0);
    state.progress_message.set("Extracting...".to_string());

    get_shared_progress().reset();

    let send = create_result_sender(state.clone());
    let progress_sender = create_progress_sender(state);

    thread::spawn(move || {
        let pak_result = if use_smart_extract {
            // Build extraction options
            // Note: glTF output uses the same converter but outputs .gltf + .bin instead of .glb
            let extraction_opts = maclarian::pak::Gr2ExtractionOptions::new()
                .with_convert_to_glb(convert_to_glb || convert_to_gltf)
                .with_extract_textures(extract_textures)
                .with_extract_virtual_textures(extract_textures) // Included with texture extraction
                .with_keep_original(extract_gr2)
                .with_png_conversion(convert_to_png)
                .with_keep_original_dds(extract_textures) // Keep DDS if "Extract textures DDS" is checked
                .with_game_data_path(game_data.map(std::path::PathBuf::from))
                .with_virtual_textures_path(None::<std::path::PathBuf>); // Uses game data path for VT lookup

            let result = maclarian::pak::extract_files_smart(
                &pak_path,
                &dest_path,
                &selected,
                extraction_opts,
                &progress_sender,
            );

            match result {
                Ok(smart_result) => {
                    let message = format!(
                        "{} GR2s processed, {} GLB created, {} textures extracted",
                        smart_result.gr2s_processed,
                        smart_result.glb_files_created,
                        smart_result.textures_extracted
                    );
                    PakResult::IndividualExtractDone {
                        success: true,
                        message,
                        files: selected,
                        dest: dest_path,
                    }
                }
                Err(e) => PakResult::IndividualExtractDone {
                    success: false,
                    message: e.to_string(),
                    files: Vec::new(),
                    dest: dest_path,
                },
            }
        } else {
            // Standard extraction
            let result = maclarian::pak::PakOperations::extract_files_with_progress(
                &pak_path,
                &dest_path,
                &selected,
                &progress_sender,
            );

            match result {
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
            }
        };

        send(pak_result);
    });
}

/// Extract individual files from a dropped PAK file (shows file selection dialog)
pub fn extract_individual_dropped_file(state: PakOpsState, pak_path: String) {
    state.clear_results();

    let pak_name = Path::new(&pak_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    state.add_result(&format!("Loading contents of {}...", pak_name));
    state.is_listing.set(true);
    state.active_dialog.set(ActiveDialog::Progress);
    state.progress.set(0.0);
    state.progress_message.set(format!("Reading {}...", pak_name));
    state.dropped_file.set(None);

    get_shared_progress().reset();

    let send = create_result_sender(state.clone());
    let progress_sender = create_progress_sender(state);

    thread::spawn(move || {
        let result = maclarian::pak::PakOperations::list_with_progress(
            &pak_path,
            &progress_sender,
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
        let result = maclarian::pak::PakOperations::extract_with_progress(
            &pak_path,
            &dest_path,
            &progress_sender,
        );

        let files = maclarian::pak::PakOperations::list(&pak_path)
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
