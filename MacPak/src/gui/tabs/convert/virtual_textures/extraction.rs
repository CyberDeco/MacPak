//! Virtual Texture extraction operations

use std::path::{Path, PathBuf};

use floem_reactive::{SignalGet, SignalUpdate};

use walkdir::WalkDir;

use super::types::{VtResult, create_result_sender, get_shared_progress};
use crate::gui::state::{ConfigState, VirtualTexturesState};
use maclarian::virtual_texture::{extract_batch as vt_extract_batch, extract_by_gtex, extract_gts_file};

/// Extract textures from a single GTS file
pub fn extract_single(state: VirtualTexturesState, game_data_path: String) {
    let gts_path = match state.gts_file.get() {
        Some(path) => path,
        None => return,
    };

    let _layer = state.selected_layer.get();
    let output_dir = state.batch_output_dir.get();
    let _from_pak = state.from_pak.get_untracked();
    let convert_to_png = state.convert_to_png.get_untracked();
    let _game_data = if game_data_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(&game_data_path))
    };

    state.is_extracting.set(true);
    state.status_message.set("Extracting...".to_string());

    let send_result = create_result_sender(state.clone());

    std::thread::spawn(move || {
        let progress = get_shared_progress();
        progress.reset();

        let gts_name = Path::new(&gts_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        progress.update(1, 1, &gts_name);

        // Use maclarian's extraction function
        let output_path = output_dir.as_ref().map(|s| Path::new(s.as_str()));
        let result = extract_gts_file(
            &gts_path,
            output_path,
            |p: &maclarian::virtual_texture::VTexProgress| {
                let desc = p.current_file.as_deref().unwrap_or(p.phase.as_str());
                progress.update(p.current, p.total, desc);
            },
        );

        match result {
            Ok(extract_result) => {
                // Convert to PNG if requested (scan output directory for DDS files)
                let mut png_converted = 0;
                if convert_to_png {
                    let search_dir = output_path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
                        Path::new(&gts_path)
                            .parent()
                            .unwrap_or(Path::new("."))
                            .to_path_buf()
                    });

                    if let Ok(entries) = std::fs::read_dir(&search_dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let path = entry.path();
                            if let Some(ext) = path.extension() {
                                if ext.to_string_lossy().to_lowercase() == "dds" {
                                    let png_path = path.with_extension("png");
                                    if maclarian::converter::convert_dds_to_png(&path, &png_path)
                                        .is_ok()
                                    {
                                        png_converted += 1;
                                    }
                                }
                            }
                        }
                    }
                }

                progress.update(1, 1, "Complete");
                let texture_info = if png_converted > 0 {
                    format!(
                        "{} (converted {} to PNG)",
                        extract_result.texture_count, png_converted
                    )
                } else {
                    format!("{}", extract_result.texture_count)
                };
                send_result(VtResult::SingleDone {
                    success: true,
                    gts_name: format!("{} - {} textures", gts_name, texture_info),
                    texture_count: extract_result.texture_count,
                    error: None,
                });
            }
            Err(e) => {
                send_result(VtResult::SingleDone {
                    success: false,
                    gts_name,
                    texture_count: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    });
}

/// Extract textures from multiple GTS files
pub fn extract_batch(state: VirtualTexturesState, game_data_path: String) {
    let files = state.batch_gts_files.get();
    if files.is_empty() {
        return;
    }

    let _layer = state.selected_layer.get();
    let output_dir = state.batch_output_dir.get();
    let _from_pak = state.from_pak.get_untracked();
    let convert_to_png = state.convert_to_png.get_untracked();
    let _game_data = if game_data_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(&game_data_path))
    };

    state.is_extracting.set(true);
    state.status_message.set("Extracting...".to_string());

    let send_result = create_result_sender(state.clone());

    std::thread::spawn(move || {
        let progress = get_shared_progress();
        progress.reset();

        // Convert file list to PathBuf
        let gts_files: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        let output_path = output_dir.as_ref().map(|s| Path::new(s.as_str()));
        let total = gts_files.len();

        // Use maclarian's batch extraction function
        let result = vt_extract_batch(
            &gts_files,
            output_path,
            |p: &maclarian::virtual_texture::VTexProgress| {
                let desc = p.current_file.as_deref().unwrap_or(p.phase.as_str());
                progress.update(p.current, p.total, desc);
            },
        );

        // Convert to PNG if requested (scan output directory for DDS files)
        let mut png_converted = 0;
        if convert_to_png {
            if let Some(out_dir) = output_path {
                if let Ok(entries) = std::fs::read_dir(out_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext.to_string_lossy().to_lowercase() == "dds" {
                                let png_path = path.with_extension("png");
                                if maclarian::converter::convert_dds_to_png(&path, &png_path)
                                    .is_ok()
                                {
                                    png_converted += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        progress.update(total, total, "Complete");

        let mut results = result.results.clone();
        if png_converted > 0 {
            results.push(format!("Converted {} DDS files to PNG", png_converted));
        }

        send_result(VtResult::BatchDone {
            success_count: result.success_count,
            error_count: result.error_count,
            texture_count: result.texture_count,
            results,
        });
    });
}

/// Convert a single DDS file to PNG
pub fn convert_dds_to_png_file(state: VirtualTexturesState) {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Select DDS File")
        .add_filter("DDS Files", &["dds"]);

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(file) = dialog.pick_file() {
        if let Some(parent) = file.parent() {
            state
                .working_dir
                .set(Some(parent.to_string_lossy().to_string()));
        }

        let input_name = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let png_path = file.with_extension("png");
        let output_name = png_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        state.is_extracting.set(true);
        state.status_message.set("Converting DDS → PNG...".to_string());

        let send_result = create_result_sender(state);

        std::thread::spawn(move || {
            let progress = get_shared_progress();
            progress.reset();
            progress.update(0, 1, &input_name);

            match maclarian::converter::convert_dds_to_png(&file, &png_path) {
                Ok(()) => {
                    progress.update(1, 1, "Complete");
                    send_result(VtResult::DdsConvertDone {
                        success: true,
                        input_name,
                        output_name,
                        error: None,
                    });
                }
                Err(e) => {
                    send_result(VtResult::DdsConvertDone {
                        success: false,
                        input_name,
                        output_name,
                        error: Some(e.to_string()),
                    });
                }
            }
        });
    }
}

/// Convert a single PNG file to DDS
pub fn convert_png_to_dds_file(state: VirtualTexturesState) {
    let format = state.dds_format.get_untracked();

    let mut dialog = rfd::FileDialog::new()
        .set_title("Select PNG File")
        .add_filter("PNG Files", &["png"]);

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(file) = dialog.pick_file() {
        if let Some(parent) = file.parent() {
            state
                .working_dir
                .set(Some(parent.to_string_lossy().to_string()));
        }

        let input_name = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let dds_path = file.with_extension("dds");
        let output_name = dds_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        state.is_extracting.set(true);
        state
            .status_message
            .set(format!("Converting PNG → DDS ({:?})...", format));

        let send_result = create_result_sender(state);

        std::thread::spawn(move || {
            let progress = get_shared_progress();
            progress.reset();
            progress.update(0, 1, &input_name);

            match maclarian::converter::convert_png_to_dds_with_format(&file, &dds_path, format) {
                Ok(()) => {
                    progress.update(1, 1, "Complete");
                    send_result(VtResult::DdsConvertDone {
                        success: true,
                        input_name,
                        output_name,
                        error: None,
                    });
                }
                Err(e) => {
                    send_result(VtResult::DdsConvertDone {
                        success: false,
                        input_name,
                        output_name,
                        error: Some(e.to_string()),
                    });
                }
            }
        });
    }
}

/// Batch convert DDS↔PNG in a directory
pub fn convert_dds_png_batch(state: VirtualTexturesState) {
    let format = state.dds_format.get_untracked();

    let mut dialog = rfd::FileDialog::new().set_title("Select Directory with DDS/PNG Files");

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(dir) = dialog.pick_folder() {
        state
            .working_dir
            .set(Some(dir.to_string_lossy().to_string()));

        state.is_extracting.set(true);
        state
            .status_message
            .set("Batch converting DDS ↔ PNG...".to_string());

        let send_result = create_result_sender(state);

        std::thread::spawn(move || {
            let progress = get_shared_progress();
            progress.reset();

            let mut dds_files = Vec::new();
            let mut png_files = Vec::new();

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        match ext.to_string_lossy().to_lowercase().as_str() {
                            "dds" => dds_files.push(path.to_path_buf()),
                            "png" => png_files.push(path.to_path_buf()),
                            _ => {}
                        }
                    }
                }
            }

            let total = dds_files.len() + png_files.len();
            let mut success_count = 0;
            let mut error_count = 0;
            let mut results = Vec::new();

            // DDS → PNG
            for (i, dds_path) in dds_files.iter().enumerate() {
                let name = dds_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                progress.update(i, total, &name);

                let png_path = dds_path.with_extension("png");
                match maclarian::converter::convert_dds_to_png(dds_path, &png_path) {
                    Ok(()) => {
                        success_count += 1;
                        results.push(format!("DDS → PNG: {}", name));
                    }
                    Err(e) => {
                        error_count += 1;
                        results.push(format!("Error ({}): {}", name, e));
                    }
                }
            }

            // PNG → DDS
            for (i, png_path) in png_files.iter().enumerate() {
                let name = png_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                progress.update(dds_files.len() + i, total, &name);

                let dds_path = png_path.with_extension("dds");
                match maclarian::converter::convert_png_to_dds_with_format(
                    png_path, &dds_path, format,
                ) {
                    Ok(()) => {
                        success_count += 1;
                        results.push(format!("PNG → DDS ({:?}): {}", format, name));
                    }
                    Err(e) => {
                        error_count += 1;
                        results.push(format!("Error ({}): {}", name, e));
                    }
                }
            }

            progress.update(total, total, "Complete");

            send_result(VtResult::DdsBatchDone {
                success_count,
                error_count,
                results,
            });
        });
    }
}

/// Extract textures by GTex hash
pub fn extract_by_gtex_hash(state: VirtualTexturesState, config: ConfigState) {
    let hash_input = state.gtex_hash_input.get_untracked();
    let convert_to_png = state.convert_to_png.get_untracked();
    let game_data_path = config.bg3_data_path.get_untracked();
    let extra_paths = state.gtex_search_paths.get_untracked();

    // Parse comma/newline-separated hashes
    let hashes: Vec<String> = hash_input
        .split(|c: char| c == ',' || c == '\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if hashes.is_empty() {
        return;
    }

    // Prompt for output directory
    let mut dialog = rfd::FileDialog::new().set_title("Select Output Directory");
    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    let Some(output_dir) = dialog.pick_folder() else {
        return;
    };

    state
        .working_dir
        .set(Some(output_dir.to_string_lossy().to_string()));

    // Build search paths
    let mut search_paths: Vec<PathBuf> = Vec::new();
    if !game_data_path.is_empty() {
        search_paths.push(PathBuf::from(&game_data_path));
    }
    for p in &extra_paths {
        search_paths.push(PathBuf::from(p));
    }

    // Close dialog and start extraction
    state.show_gtex_dialog.set(false);
    state.is_extracting.set(true);
    state
        .status_message
        .set(format!("Extracting {} GTex hash(es)...", hashes.len()));

    let send_result = create_result_sender(state);

    std::thread::spawn(move || {
        let progress = get_shared_progress();
        progress.reset();

        let total = hashes.len();
        let mut success_count = 0;
        let mut error_count = 0;
        let mut texture_count = 0;
        let mut results = Vec::new();

        for (i, hash) in hashes.iter().enumerate() {
            progress.update(i, total, hash);

            match extract_by_gtex(hash, &search_paths, &output_dir) {
                Ok(extract_result) => {
                    success_count += 1;
                    texture_count += extract_result.extracted;
                    results.push(format!(
                        "Extracted {} textures for hash {}",
                        extract_result.extracted, hash
                    ));
                    if !extract_result.errors.is_empty() {
                        for err in &extract_result.errors {
                            results.push(format!("  Warning: {}", err));
                        }
                    }
                }
                Err(e) => {
                    error_count += 1;
                    results.push(format!("Error for hash {}: {}", hash, e));
                }
            }
        }

        // Convert to PNG if requested
        if convert_to_png {
            let mut png_converted = 0;
            if let Ok(entries) = std::fs::read_dir(&output_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().to_lowercase() == "dds" {
                            let png_path = path.with_extension("png");
                            if maclarian::converter::convert_dds_to_png(&path, &png_path).is_ok() {
                                png_converted += 1;
                            }
                        }
                    }
                }
            }
            if png_converted > 0 {
                results.push(format!("Converted {} DDS files to PNG", png_converted));
            }
        }

        progress.update(total, total, "Complete");

        send_result(VtResult::BatchDone {
            success_count,
            error_count,
            texture_count,
            results,
        });
    });
}
