//! Virtual Texture extraction operations

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use floem_reactive::{SignalGet, SignalUpdate};
use rayon::prelude::*;

use crate::gui::shared::SharedProgress;
use crate::gui::state::VirtualTexturesState;
use crate::operations::virtual_texture::find_gts_path;
use super::types::{VtResult, create_result_sender, get_shared_progress};

/// Extract textures from a single GTS file
pub fn extract_single(state: VirtualTexturesState) {
    let gts_path = match state.gts_file.get() {
        Some(path) => path,
        None => return,
    };

    let _layer = state.selected_layer.get();
    let output_dir = state.batch_output_dir.get();

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

        progress.update(0, 1, &gts_name);

        let result = extract_gts_file(&gts_path, output_dir.as_deref(), progress);

        match result {
            Ok(count) => {
                progress.update(1, 1, "Complete");
                send_result(VtResult::SingleDone {
                    success: true,
                    gts_name,
                    texture_count: count,
                    error: None,
                });
            }
            Err(e) => {
                send_result(VtResult::SingleDone {
                    success: false,
                    gts_name,
                    texture_count: 0,
                    error: Some(e),
                });
            }
        }
    });
}

/// Extract textures from multiple GTS files
pub fn extract_batch(state: VirtualTexturesState) {
    let files = state.batch_gts_files.get();
    if files.is_empty() {
        return;
    }

    let _layer = state.selected_layer.get();
    let output_dir = state.batch_output_dir.get();

    state.is_extracting.set(true);
    state.status_message.set("Extracting...".to_string());

    let send_result = create_result_sender(state.clone());

    std::thread::spawn(move || {
        let progress = get_shared_progress();
        progress.reset();

        let total = files.len();
        let success_counter = AtomicUsize::new(0);
        let error_counter = AtomicUsize::new(0);
        let texture_counter = AtomicUsize::new(0);
        let processed = AtomicUsize::new(0);

        // Parallel GTS extraction
        let results: Vec<String> = files
            .par_iter()
            .map(|gts_path| {
                let gts_name = Path::new(gts_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                progress.update(current, total, &gts_name);

                match extract_gts_file(gts_path, output_dir.as_deref(), progress) {
                    Ok(count) => {
                        success_counter.fetch_add(1, Ordering::SeqCst);
                        texture_counter.fetch_add(count, Ordering::SeqCst);
                        format!("Extracted {} textures from {}", count, gts_name)
                    }
                    Err(e) => {
                        error_counter.fetch_add(1, Ordering::SeqCst);
                        format!("Failed {}: {}", gts_name, e)
                    }
                }
            })
            .collect();

        progress.update(total, total, "Complete");

        send_result(VtResult::BatchDone {
            success_count: success_counter.load(Ordering::SeqCst),
            error_count: error_counter.load(Ordering::SeqCst),
            texture_count: texture_counter.load(Ordering::SeqCst),
            results,
        });
    });
}

/// Extract textures from a GTS/GTP file to the output directory
fn extract_gts_file(
    input_path: &str,
    output_dir: Option<&str>,
    progress: &SharedProgress,
) -> Result<usize, String> {
    use crate::operations::virtual_texture::{self, GtsFile};

    let input_ext = Path::new(input_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let is_single_gtp = input_ext == "gtp";

    // Find the GTS file (handles both .gts and .gtp inputs)
    let gts_path = find_gts_path(input_path).map_err(|e| e.to_string())?;

    // Determine output directory
    let output_path = match output_dir {
        Some(dir) => std::path::PathBuf::from(dir),
        None => {
            // Use directory next to input file
            Path::new(input_path)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        }
    };

    // Create subdirectory based on input filename
    let input_stem = Path::new(input_path)
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "textures".to_string());
    let texture_output_dir = output_path.join(&input_stem);

    std::fs::create_dir_all(&texture_output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    if is_single_gtp {
        // Single GTP mode: extract just this GTP file
        progress.update(0, 1, "Extracting GTP...");

        virtual_texture::extract_gtp(
            input_path,
            &gts_path,
            &texture_output_dir,
        ).map_err(|e| format!("Failed to extract: {}", e))?;

        progress.update(1, 1, "Complete");

        // Count output files
        let count = std::fs::read_dir(&texture_output_dir)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        Ok(count)
    } else {
        // Full GTS mode: extract all GTPs referenced by this GTS
        let gts = GtsFile::open(&gts_path)
            .map_err(|e| format!("Failed to parse GTS: {}", e))?;

        let gts_dir = Path::new(&gts_path).parent().unwrap_or(Path::new("."));
        let total_page_files = gts.page_files.len();

        if total_page_files == 0 {
            return Err("No page files found in GTS".to_string());
        }

        let mut extracted_count = 0;
        let mut failed_count = 0;

        for (i, page_file) in gts.page_files.iter().enumerate() {
            let gtp_path = gts_dir.join(&page_file.filename);

            progress.update(i, total_page_files, &format!("Extracting {}...", page_file.filename));

            if gtp_path.exists() {
                // Create a subdirectory for this GTP's output
                let gtp_stem = Path::new(&page_file.filename)
                    .file_stem()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("gtp_{}", i));
                let gtp_output_dir = texture_output_dir.join(&gtp_stem);

                match virtual_texture::extract_gtp(
                    &gtp_path,
                    &gts_path,
                    &gtp_output_dir,
                ) {
                    Ok(()) => extracted_count += 1,
                    Err(e) => {
                        eprintln!("Warning: Failed to extract {}: {}", page_file.filename, e);
                        failed_count += 1;
                    }
                }
            } else {
                eprintln!("Warning: GTP file not found: {}", gtp_path.display());
                failed_count += 1;
            }
        }

        if extracted_count == 0 && total_page_files > 0 {
            return Err(format!(
                "No GTP files could be extracted (0/{} succeeded, {} failed)",
                total_page_files, failed_count
            ));
        }

        Ok(extracted_count)
    }
}
