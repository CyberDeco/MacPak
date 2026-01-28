//! Virtual Texture extraction operations

use std::path::{Path, PathBuf};

use floem_reactive::{SignalGet, SignalUpdate};

use super::types::{VtResult, create_result_sender, get_shared_progress};
use crate::gui::state::VirtualTexturesState;
use maclarian::virtual_texture::{extract_batch as vt_extract_batch, extract_gts_file};

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
