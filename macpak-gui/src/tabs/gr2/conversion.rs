//! GR2 Conversion operations

use std::path::Path;
use std::thread;

use floem::prelude::*;

use crate::state::{Gr2ConversionDirection, Gr2OutputFormat, Gr2State};
use super::types::{create_result_sender, get_shared_progress, Gr2Result};

/// Convert a single file
pub fn convert_single(state: Gr2State) {
    let Some(input_path) = state.input_file.get() else {
        state.status_message.set("No input file selected".to_string());
        return;
    };

    let direction = state.direction.get();
    let output_format = state.output_format.get();

    // Generate output path
    let input = Path::new(&input_path);
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let parent = input.parent().unwrap_or(Path::new("."));

    let output_ext = match direction {
        Gr2ConversionDirection::Gr2ToGltf => output_format.extension(),
        Gr2ConversionDirection::GltfToGr2 => "gr2",
    };

    let output_path = parent.join(format!("{}.{}", stem, output_ext));

    // Ask user to confirm output location
    let title = match direction {
        Gr2ConversionDirection::Gr2ToGltf => "Save Converted File As",
        Gr2ConversionDirection::GltfToGr2 => "Save GR2 File As",
    };

    let mut dialog = rfd::FileDialog::new()
        .set_title(title)
        .set_file_name(output_path.file_name().unwrap_or_default().to_string_lossy());

    match direction {
        Gr2ConversionDirection::Gr2ToGltf => match output_format {
            Gr2OutputFormat::Glb => {
                dialog = dialog.add_filter("GLB Files", &["glb"]);
            }
            Gr2OutputFormat::Gltf => {
                dialog = dialog.add_filter("glTF Files", &["gltf"]);
            }
        },
        Gr2ConversionDirection::GltfToGr2 => {
            dialog = dialog.add_filter("GR2 Files", &["gr2"]);
        }
    }

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    let Some(save_path) = dialog.save_file() else {
        return;
    };

    // Store the output path
    state.output_file.set(Some(save_path.to_string_lossy().to_string()));

    // Start conversion
    state.is_converting.set(true);
    state.clear_results();

    let input_str = input_path.clone();
    let output_str = save_path.to_string_lossy().to_string();

    // Create result sender for updating UI from background thread
    let send_result = create_result_sender(state);

    thread::spawn(move || {
        let shared = get_shared_progress();
        shared.update(0, 100, "Converting...");

        let result = match direction {
            Gr2ConversionDirection::Gr2ToGltf => match output_format {
                Gr2OutputFormat::Glb => {
                    MacLarian::converter::convert_gr2_to_glb(
                        Path::new(&input_str),
                        Path::new(&output_str),
                    )
                }
                Gr2OutputFormat::Gltf => {
                    MacLarian::converter::convert_gr2_to_gltf(
                        Path::new(&input_str),
                        Path::new(&output_str),
                    )
                }
            },
            Gr2ConversionDirection::GltfToGr2 => {
                MacLarian::converter::convert_gltf_to_gr2(
                    Path::new(&input_str),
                    Path::new(&output_str),
                )
            }
        };

        shared.update(100, 100, "Complete");

        let input_name = Path::new(&input_str)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let output_name = Path::new(&output_str)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        match result {
            Ok(()) => {
                send_result(Gr2Result::SingleDone {
                    success: true,
                    input_name,
                    output_name,
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

/// Convert all files in batch
pub fn convert_batch(state: Gr2State) {
    let files = state.batch_files.get();
    if files.is_empty() {
        state.status_message.set("No files to convert".to_string());
        return;
    }

    let direction = state.direction.get();
    let output_format = state.output_format.get();
    let output_dir = state.batch_output_dir.get();
    let input_base_dir = state.batch_input_dir.get();

    // Start conversion
    state.is_converting.set(true);
    state.clear_results();

    let total = files.len();

    // Create result sender for updating UI from background thread
    let send_result = create_result_sender(state);

    thread::spawn(move || {
        let shared = get_shared_progress();
        let mut success_count = 0;
        let mut error_count = 0;
        let mut results = Vec::new();

        for (i, input_path) in files.iter().enumerate() {
            let input = Path::new(input_path);
            let stem = input.file_stem().unwrap_or_default().to_string_lossy();

            let output_ext = match direction {
                Gr2ConversionDirection::Gr2ToGltf => output_format.extension(),
                Gr2ConversionDirection::GltfToGr2 => "gr2",
            };

            // Determine output path, preserving directory structure
            let output_path = if let Some(ref out_base) = output_dir {
                // If we have an output directory and input base, preserve structure
                if let Some(ref in_base) = input_base_dir {
                    let in_base_path = Path::new(in_base);
                    // Get relative path from input base to this file's parent
                    if let Ok(relative) = input.parent().unwrap_or(Path::new(".")).strip_prefix(in_base_path) {
                        let out_subdir = Path::new(out_base).join(relative);
                        // Create subdirectory if needed
                        if !out_subdir.exists() {
                            let _ = std::fs::create_dir_all(&out_subdir);
                        }
                        out_subdir.join(format!("{}.{}", stem, output_ext))
                    } else {
                        // Fallback: put in output root
                        Path::new(out_base).join(format!("{}.{}", stem, output_ext))
                    }
                } else {
                    // No input base, put in output root
                    Path::new(out_base).join(format!("{}.{}", stem, output_ext))
                }
            } else {
                // No output dir specified, put next to input file
                input.parent().unwrap_or(Path::new(".")).join(format!("{}.{}", stem, output_ext))
            };

            // Update progress via shared state
            let input_name = input.file_name().unwrap_or_default().to_string_lossy();
            shared.update(i, total, &input_name);

            // Perform conversion
            let result = match direction {
                Gr2ConversionDirection::Gr2ToGltf => match output_format {
                    Gr2OutputFormat::Glb => {
                        MacLarian::converter::convert_gr2_to_glb(input, &output_path)
                    }
                    Gr2OutputFormat::Gltf => {
                        MacLarian::converter::convert_gr2_to_gltf(input, &output_path)
                    }
                },
                Gr2ConversionDirection::GltfToGr2 => {
                    MacLarian::converter::convert_gltf_to_gr2(input, &output_path)
                }
            };

            // Show relative path in results if we have a base dir
            let display_path = if let Some(ref in_base) = input_base_dir {
                input.strip_prefix(in_base)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| input_name.to_string())
            } else {
                input_name.to_string()
            };

            match result {
                Ok(()) => {
                    let output_name = output_path.file_name().unwrap_or_default().to_string_lossy();
                    results.push(format!("Converted {} -> {}", display_path, output_name));
                    success_count += 1;
                }
                Err(e) => {
                    results.push(format!("Failed {}: {}", display_path, e));
                    error_count += 1;
                }
            }
        }

        // Final progress update
        shared.update(total, total, "Complete");

        // Send results back to UI thread
        send_result(Gr2Result::BatchDone {
            success_count,
            error_count,
            results,
        });
    });
}
