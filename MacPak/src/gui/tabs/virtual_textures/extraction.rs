//! Virtual Texture extraction operations

use std::path::{Path, PathBuf};

use floem_reactive::{SignalGet, SignalUpdate};

use crate::gui::state::VirtualTexturesState;
use maclarian::virtual_texture::{extract_gts_file, extract_batch as vt_extract_batch};
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

        // Use maclarian's extraction function
        let output_path = output_dir.as_ref().map(|s| Path::new(s.as_str()));
        let result = extract_gts_file(
            &gts_path,
            output_path,
            |current, total, desc| progress.update(current, total, desc),
        );

        match result {
            Ok(extract_result) => {
                progress.update(1, 1, "Complete");
                send_result(VtResult::SingleDone {
                    success: true,
                    gts_name,
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

        // Convert file list to PathBuf
        let gts_files: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        let output_path = output_dir.as_ref().map(|s| Path::new(s.as_str()));
        let total = gts_files.len();

        // Use maclarian's batch extraction function
        let result = vt_extract_batch(
            &gts_files,
            output_path,
            |current, total, desc| progress.update(current, total, desc),
        );

        progress.update(total, total, "Complete");

        send_result(VtResult::BatchDone {
            success_count: result.success_count,
            error_count: result.error_count,
            texture_count: result.texture_count,
            results: result.results,
        });
    });
}
