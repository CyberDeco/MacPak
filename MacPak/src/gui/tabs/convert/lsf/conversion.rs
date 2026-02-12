//! LSF/LSX/LSJ/LOCA Conversion operations

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use floem::prelude::*;
use rayon::prelude::*;

use super::types::{LsfResult, create_result_sender, get_shared_progress};
use crate::gui::state::LsfConvertState;

/// Determine the output extension for a given conversion
fn output_extension(source_ext: &str, target_format: &str) -> &'static str {
    match (source_ext, target_format) {
        ("lsf", "LSX") => "lsx",
        ("lsf", "LSJ") => "lsj",
        ("lsx", "LSF") => "lsf",
        ("lsx", "LSJ") => "lsj",
        ("lsj", "LSX") => "lsx",
        ("lsj", "LSF") => "lsf",
        ("loca", "XML") => "xml",
        ("xml", "LOCA") => "loca",
        _ => "lsx", // fallback
    }
}

/// Perform the actual file conversion
fn do_convert(
    source: &Path,
    dest: &Path,
    source_ext: &str,
    target_format: &str,
    progress_cb: &(dyn Fn(&maclarian::converter::ConvertProgress) + Sync + Send),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match (source_ext, target_format) {
        ("lsf", "LSX") => {
            maclarian::converter::lsf_to_lsx_with_progress(source, dest, &|p| progress_cb(p))?;
        }
        ("lsf", "LSJ") => {
            maclarian::converter::lsf_to_lsj_with_progress(source, dest, &|p| progress_cb(p))?;
        }
        ("lsx", "LSF") => {
            maclarian::converter::lsx_to_lsf_with_progress(source, dest, &|p| progress_cb(p))?;
        }
        ("lsx", "LSJ") => {
            maclarian::converter::lsx_to_lsj_with_progress(source, dest, &|p| progress_cb(p))?;
        }
        ("lsj", "LSX") => {
            maclarian::converter::lsj_to_lsx_with_progress(source, dest, &|p| progress_cb(p))?;
        }
        ("lsj", "LSF") => {
            maclarian::converter::lsj_to_lsf_with_progress(source, dest, &|p| progress_cb(p))?;
        }
        ("loca", "XML") => {
            maclarian::converter::convert_loca_to_xml(source, dest)?;
        }
        ("xml", "LOCA") => {
            maclarian::converter::convert_xml_to_loca(source, dest)?;
        }
        _ => {
            return Err(format!(
                "Unsupported conversion: {} -> {}",
                source_ext, target_format
            )
            .into());
        }
    }
    Ok(())
}

/// Convert a single file to the given output directory
pub fn convert_single(state: LsfConvertState, output_dir: String) {
    let Some(input_path) = state.input_file.get() else {
        state
            .status_message
            .set("No input file selected".to_string());
        return;
    };

    let target_format = state.target_format.get();
    if target_format.is_empty() {
        state
            .status_message
            .set("No target format selected".to_string());
        return;
    }

    let input = Path::new(&input_path);
    let source_ext = input
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let input_name = input
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let stem = input
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let out_ext = output_extension(&source_ext, &target_format);
    let output_path = Path::new(&output_dir).join(format!("{}.{}", stem, out_ext));
    let output_str = output_path.to_string_lossy().to_string();
    let output_name = output_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    state.is_converting.set(true);
    state.clear_results();

    let input_str = input_path.clone();
    let send_result = create_result_sender(state);

    thread::spawn(move || {
        let shared = get_shared_progress();
        shared.update(1, 1, &input_name);

        let result = do_convert(
            Path::new(&input_str),
            Path::new(&output_str),
            &source_ext,
            &target_format,
            &|progress| {
                shared.update(
                    progress.current,
                    progress.total + 1,
                    progress.phase.as_str(),
                );
            },
        );

        shared.update(2, 2, "Complete");

        match result {
            Ok(()) => {
                send_result(LsfResult::SingleDone {
                    success: true,
                    input_name,
                    output_name,
                    error: None,
                });
            }
            Err(e) => {
                send_result(LsfResult::SingleDone {
                    success: false,
                    input_name,
                    output_name,
                    error: Some(e.to_string()),
                });
            }
        }
    });
}

/// Convert a batch of files to the given output directory, preserving relative paths
pub fn convert_batch(state: LsfConvertState, output_dir: String) {
    let files = state.batch_files.get();
    if files.is_empty() {
        state.status_message.set("No files to convert".to_string());
        return;
    }

    let target_format = state.batch_target_format.get();
    let source_format = state.batch_source_format.get();
    let source_ext = source_format.to_lowercase();
    let input_base_dir = state.batch_input_dir.get();

    state.is_converting.set(true);
    state.clear_results();

    let total = files.len();
    let send_result = create_result_sender(state);

    thread::spawn(move || {
        let shared = get_shared_progress();
        let success_counter = AtomicUsize::new(0);
        let error_counter = AtomicUsize::new(0);
        let processed = AtomicUsize::new(0);

        let results: Vec<String> = files
            .par_iter()
            .map(|input_path| {
                let input = Path::new(input_path);
                let stem = input
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let input_name = input
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // Compute output parent from output_dir + relative path from input base
                let output_parent = if let Some(ref in_base) = input_base_dir {
                    let rel = input
                        .parent()
                        .unwrap_or(Path::new("."))
                        .strip_prefix(in_base)
                        .unwrap_or(Path::new("."));
                    std::path::PathBuf::from(&output_dir).join(rel)
                } else {
                    std::path::PathBuf::from(&output_dir)
                };

                let out_ext = output_extension(&source_ext, &target_format);
                let output_path = output_parent.join(format!("{}.{}", stem, out_ext));

                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                shared.update(current, total, &input_name);

                // Create output directory if needed
                let _ = std::fs::create_dir_all(&output_parent);

                let result = do_convert(
                    input,
                    &output_path,
                    &source_ext,
                    &target_format,
                    &|_| {}, // No per-file progress for batch
                );

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
                        format!("Converted {} -> {}", display_path, output_name)
                    }
                    Err(e) => {
                        error_counter.fetch_add(1, Ordering::SeqCst);
                        format!("Failed {}: {}", display_path, e)
                    }
                }
            })
            .collect();

        shared.update(total, total, "Complete");

        send_result(LsfResult::BatchDone {
            success_count: success_counter.load(Ordering::SeqCst),
            error_count: error_counter.load(Ordering::SeqCst),
            results,
        });
    });
}
