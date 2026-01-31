//! GR2 Conversion operations

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use floem::prelude::*;
use rayon::prelude::*;

use super::types::{Gr2Result, create_result_sender, get_shared_progress};
use crate::gui::state::Gr2State;

/// Convert a single file with explicit options (for operation buttons UI)
/// Determines direction from input file extension
pub fn convert_single_with_options(state: Gr2State, to_glb: bool, game_data_path: String) {
    let Some(input_path) = state.input_file.get() else {
        state
            .status_message
            .set("No input file selected".to_string());
        return;
    };

    let input = Path::new(&input_path);
    let stem = input
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let parent = input.parent().unwrap_or(Path::new(".")).to_path_buf();
    let input_ext = input
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let input_name = input
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Determine direction from input file extension
    let is_gr2_input = input_ext == "gr2";
    let output_ext = if is_gr2_input {
        if to_glb { "glb" } else { "gltf" }
    } else {
        "gr2"
    };

    // Capture bundle options before spawning thread
    let extract_textures = state.extract_textures.get_untracked();
    let convert_to_png = state.convert_to_png.get_untracked();
    let keep_original_dds = state.keep_original_dds.get_untracked();
    let keep_original_gr2 = state.keep_original_gr2.get_untracked();

    // Use subdirectory when:
    // - Converting to glTF (always, because it outputs .gltf + .bin)
    // - Converting to GLB with texture extraction enabled
    let use_subdir = is_gr2_input && (!to_glb || extract_textures);
    let (output_dir, output_path) = if use_subdir {
        let subdir = parent.join(&stem);
        (
            subdir.clone(),
            subdir.join(format!("{}.{}", stem, output_ext)),
        )
    } else {
        (
            parent.clone(),
            parent.join(format!("{}.{}", stem, output_ext)),
        )
    };

    let output_str = output_path.to_string_lossy().to_string();
    let output_name = output_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Start conversion
    state.is_converting.set(true);
    state.clear_results();

    let input_str = input_path.clone();
    let send_result = create_result_sender(state);

    thread::spawn(move || {
        let shared = get_shared_progress();
        shared.update(1, 1, &input_name);

        // Create output directory if needed
        if use_subdir {
            let _ = std::fs::create_dir_all(&output_dir);
        }

        let result = if is_gr2_input {
            if to_glb {
                maclarian::converter::convert_gr2_to_glb_with_progress(
                    Path::new(&input_str),
                    Path::new(&output_str),
                    &|progress| {
                        shared.update(
                            progress.current,
                            progress.total + 1,
                            progress.phase.as_str(),
                        );
                    },
                )
            } else {
                maclarian::converter::convert_gr2_to_gltf_with_progress(
                    Path::new(&input_str),
                    Path::new(&output_str),
                    &|progress| {
                        shared.update(
                            progress.current,
                            progress.total + 1,
                            progress.phase.as_str(),
                        );
                    },
                )
            }
        } else {
            maclarian::converter::convert_gltf_to_gr2_with_progress(
                Path::new(&input_str),
                Path::new(&output_str),
                &|progress| {
                    shared.update(progress.current, progress.total, progress.phase.as_str());
                },
            )
        };

        // Handle texture extraction for GR2→GLB/glTF conversions
        let mut texture_info = String::new();
        if result.is_ok() && is_gr2_input && extract_textures {
            shared.update(1, 2, "Extracting textures...");

            let options = maclarian::gr2_extraction::Gr2ExtractionOptions {
                convert_to_glb: false, // Already converted
                extract_textures,
                extract_virtual_textures: false,
                bg3_path: if game_data_path.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&game_data_path))
                },
                virtual_textures_path: None,
                keep_original_gr2: true,
                convert_to_png,
                keep_original_dds,
            };

            match maclarian::gr2_extraction::process_extracted_gr2_to_dir(
                Path::new(&input_str),
                &output_dir,
                &options,
            ) {
                Ok(tex_result) => {
                    if !tex_result.texture_paths.is_empty() {
                        texture_info = format!(" + {} textures", tex_result.texture_paths.len());
                    }
                    for warning in tex_result.warnings {
                        // Log warnings but don't fail
                        eprintln!("Texture extraction warning: {}", warning);
                    }
                }
                Err(e) => {
                    texture_info = format!(" (texture extraction failed: {})", e);
                }
            }
        }

        // Copy original GR2 to subdirectory if requested (works for both GLB and glTF)
        if result.is_ok() && is_gr2_input && keep_original_gr2 && use_subdir {
            let gr2_dest = output_dir.join(&input_name);
            let _ = std::fs::copy(Path::new(&input_str), &gr2_dest);
        }

        shared.update(2, 2, "Complete");

        // Show the subdirectory in output name
        let display_output = if use_subdir {
            format!("{}/{}{}", stem, output_name, texture_info)
        } else {
            format!("{}{}", output_name, texture_info)
        };

        match result {
            Ok(()) => {
                send_result(Gr2Result::SingleDone {
                    success: true,
                    input_name,
                    output_name: display_output,
                    error: None,
                });
            }
            Err(e) => {
                send_result(Gr2Result::SingleDone {
                    success: false,
                    input_name,
                    output_name,
                    error: Some(e.to_string()),
                });
            }
        }
    });
}

/// Convert batch files with explicit options (for operation buttons UI)
/// Determines direction from input file extensions
pub fn convert_batch_with_options(state: Gr2State, to_glb: bool, game_data_path: String) {
    let files = state.batch_files.get();
    if files.is_empty() {
        state.status_message.set("No files to convert".to_string());
        return;
    }

    let input_base_dir = state.batch_input_dir.get();

    // Capture bundle options before spawning thread
    let extract_textures = state.extract_textures.get_untracked();
    let convert_to_png = state.convert_to_png.get_untracked();
    let keep_original_dds = state.keep_original_dds.get_untracked();
    let keep_original_gr2 = state.keep_original_gr2.get_untracked();

    // Start conversion
    state.is_converting.set(true);
    state.clear_results();

    let total = files.len();
    let send_result = create_result_sender(state);

    thread::spawn(move || {
        let shared = get_shared_progress();
        let success_counter = AtomicUsize::new(0);
        let error_counter = AtomicUsize::new(0);
        let processed = AtomicUsize::new(0);

        // Parallel conversion
        let results: Vec<String> = files
            .par_iter()
            .map(|input_path| {
                let input = Path::new(input_path);
                let stem = input
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let parent = input.parent().unwrap_or(Path::new("."));
                let input_ext = input
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let input_name = input
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // Determine direction from input file extension
                let is_gr2_input = input_ext == "gr2";
                let output_ext = if is_gr2_input {
                    if to_glb { "glb" } else { "gltf" }
                } else {
                    "gr2"
                };

                // Use subdirectory when:
                // - Converting to glTF (always, because it outputs .gltf + .bin)
                // - Converting to GLB with texture extraction enabled
                let use_subdir = is_gr2_input && (!to_glb || extract_textures);
                let (output_dir, output_path) = if use_subdir {
                    let subdir = parent.join(&stem);
                    (
                        Some(subdir.clone()),
                        subdir.join(format!("{}.{}", stem, output_ext)),
                    )
                } else {
                    (None, parent.join(format!("{}.{}", stem, output_ext)))
                };

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                shared.update(current, total, &input_name);

                // Create output directory if needed
                if let Some(ref dir) = output_dir {
                    let _ = std::fs::create_dir_all(dir);
                }

                // Perform conversion (no per-file progress for batch - just count files)
                let result = if is_gr2_input {
                    if to_glb {
                        maclarian::converter::convert_gr2_to_glb(input, &output_path)
                    } else {
                        maclarian::converter::convert_gr2_to_gltf(input, &output_path)
                    }
                } else {
                    maclarian::converter::convert_gltf_to_gr2(input, &output_path)
                };

                // Show relative path in results
                let display_path = if let Some(ref in_base) = input_base_dir {
                    input
                        .strip_prefix(in_base)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| input_name.clone())
                } else {
                    input_name.clone()
                };

                match result {
                    Ok(()) => {
                        success_counter.fetch_add(1, Ordering::SeqCst);
                        let output_name = output_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();

                        // Handle texture extraction for GR2→GLB/glTF conversions
                        let texture_info = if is_gr2_input && extract_textures {
                            let options = maclarian::gr2_extraction::Gr2ExtractionOptions {
                                convert_to_glb: false,
                                extract_textures,
                                extract_virtual_textures: false,
                                bg3_path: if game_data_path.is_empty() {
                                    None
                                } else {
                                    Some(PathBuf::from(&game_data_path))
                                },
                                virtual_textures_path: None,
                                keep_original_gr2: true,
                                convert_to_png,
                                keep_original_dds,
                            };

                            let parent_buf = parent.to_path_buf();
                            let tex_output_dir = output_dir.as_ref().unwrap_or(&parent_buf);
                            match maclarian::gr2_extraction::process_extracted_gr2_to_dir(
                                input,
                                tex_output_dir,
                                &options,
                            ) {
                                Ok(tex_result) if !tex_result.texture_paths.is_empty() => {
                                    format!(" + {} textures", tex_result.texture_paths.len())
                                }
                                _ => String::new(),
                            }
                        } else {
                            String::new()
                        };

                        // Copy original GR2 to subdirectory if requested (works for both GLB and glTF)
                        if keep_original_gr2 {
                            if let Some(ref dir) = output_dir {
                                let gr2_dest = dir.join(&input_name);
                                let _ = std::fs::copy(input, &gr2_dest);
                            }
                        }

                        // Show subdirectory in output
                        let display_output = if output_dir.is_some() {
                            format!("{}/{}{}", stem, output_name, texture_info)
                        } else {
                            format!("{}{}", output_name, texture_info)
                        };

                        format!("Converted {} -> {}", display_path, display_output)
                    }
                    Err(e) => {
                        error_counter.fetch_add(1, Ordering::SeqCst);
                        format!("Failed {}: {}", display_path, e)
                    }
                }
            })
            .collect();

        // Final progress update
        shared.update(total, total, "Complete");

        send_result(Gr2Result::BatchDone {
            success_count: success_counter.load(Ordering::SeqCst),
            error_count: error_counter.load(Ordering::SeqCst),
            results,
        });
    });
}
