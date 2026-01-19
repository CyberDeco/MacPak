//! GR2 Conversion operations

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use floem::prelude::*;
use rayon::prelude::*;

use crate::gui::state::Gr2State;
use super::types::{create_result_sender, get_shared_progress, Gr2Result};

/// Convert a single file with explicit options (for operation buttons UI)
/// Determines direction from input file extension
pub fn convert_single_with_options(state: Gr2State, to_glb: bool) {
    let Some(input_path) = state.input_file.get() else {
        state.status_message.set("No input file selected".to_string());
        return;
    };

    let input = Path::new(&input_path);
    let stem = input.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let parent = input.parent().unwrap_or(Path::new(".")).to_path_buf();
    let input_ext = input.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let input_name = input.file_name().unwrap_or_default().to_string_lossy().to_string();

    // Determine direction from input file extension
    let is_gr2_input = input_ext == "gr2";
    let output_ext = if is_gr2_input {
        if to_glb { "glb" } else { "gltf" }
    } else {
        "gr2"
    };

    let output_path = parent.join(format!("{}.{}", stem, output_ext));
    let output_str = output_path.to_string_lossy().to_string();
    let output_name = output_path.file_name().unwrap_or_default().to_string_lossy().to_string();

    // Start conversion
    state.is_converting.set(true);
    state.clear_results();

    let input_str = input_path.clone();
    let send_result = create_result_sender(state);

    thread::spawn(move || {
        let shared = get_shared_progress();
        shared.update(0, 1, &input_name);

        let result = if is_gr2_input {
            if to_glb {
                maclarian::converter::convert_gr2_to_glb(Path::new(&input_str), Path::new(&output_str))
            } else {
                maclarian::converter::convert_gr2_to_gltf(Path::new(&input_str), Path::new(&output_str))
            }
        } else {
            maclarian::converter::convert_gltf_to_gr2(Path::new(&input_str), Path::new(&output_str))
        };

        shared.update(1, 1, "Complete");

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

/// Convert batch files with explicit options (for operation buttons UI)
/// Determines direction from input file extensions
pub fn convert_batch_with_options(state: Gr2State, to_glb: bool) {
    let files = state.batch_files.get();
    if files.is_empty() {
        state.status_message.set("No files to convert".to_string());
        return;
    }

    let input_base_dir = state.batch_input_dir.get();

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
                let stem = input.file_stem().unwrap_or_default().to_string_lossy();
                let parent = input.parent().unwrap_or(Path::new("."));
                let input_ext = input
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                // Determine direction from input file extension
                let is_gr2_input = input_ext == "gr2";
                let output_ext = if is_gr2_input {
                    if to_glb { "glb" } else { "gltf" }
                } else {
                    "gr2"
                };

                let output_path = parent.join(format!("{}.{}", stem, output_ext));

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                let input_name = input.file_name().unwrap_or_default().to_string_lossy();
                shared.update(current, total, &input_name);

                // Perform conversion
                let result = if is_gr2_input {
                    if to_glb {
                        maclarian::converter::convert_gr2_to_glb(input, &output_path)
                    } else {
                        maclarian::converter::convert_gr2_to_gltf(input, &output_path)
                    }
                } else {
                    maclarian::converter::convert_gltf_to_gr2(input, &output_path)
                };

                // Show relative path in results if we have a base dir
                let display_path = if let Some(ref in_base) = input_base_dir {
                    input
                        .strip_prefix(in_base)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| input_name.to_string())
                } else {
                    input_name.to_string()
                };

                match result {
                    Ok(()) => {
                        success_counter.fetch_add(1, Ordering::SeqCst);
                        let output_name = output_path.file_name().unwrap_or_default().to_string_lossy();
                        format!("Converted {} -> {}", display_path, output_name)
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
