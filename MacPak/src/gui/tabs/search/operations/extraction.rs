//! File extraction operations

use std::collections::HashMap;
use std::path::PathBuf;

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;
use maclarian::pak::PakOperations;

use crate::gui::state::SearchState;

use super::progress::SEARCH_PROGRESS;

/// Open extraction dialog for selected search results
pub fn extract_selected_results(state: SearchState) {
    let selected_paths = state.selected_results.get();
    if selected_paths.is_empty() {
        return;
    }

    let all_results = state.results.get();

    // Filter to selected results and collect (internal_path, pak_path)
    let to_extract: Vec<(String, PathBuf)> = all_results
        .into_iter()
        .filter(|r| selected_paths.contains(&r.path))
        .map(|r| (r.path, r.pak_path))
        .collect();

    if to_extract.is_empty() {
        return;
    }

    // Set pending files and show dialog
    state.pending_extract_files.set(to_extract);
    state.show_extract_dialog.set(true);
}

/// Open extraction dialog for a single search result
pub fn extract_single_result(state: SearchState, internal_path: String, pak_path: PathBuf) {
    // Set pending files and show dialog
    state
        .pending_extract_files
        .set(vec![(internal_path, pak_path)]);
    state.show_extract_dialog.set(true);
}

/// Execute extraction with options from dialog
pub fn execute_extraction(state: SearchState, config: crate::gui::state::ConfigState) {
    let pending_files = state.pending_extract_files.get();
    if pending_files.is_empty() {
        return;
    }

    // Capture GR2 options
    let extract_gr2 = state.gr2_extract_gr2.get();
    let convert_to_glb = state.gr2_convert_to_glb.get();
    let convert_to_gltf = state.gr2_convert_to_gltf.get();
    let extract_textures = state.gr2_extract_textures.get();
    let convert_to_png = state.gr2_convert_to_png.get();
    let game_data = config.bg3_data_path.get();

    // Check if any GR2 processing is enabled (beyond just extracting the GR2)
    let use_smart_extract = convert_to_glb || convert_to_gltf || extract_textures;

    // Close the dialog
    state.show_extract_dialog.set(false);

    // Get destination folder
    let dest = match rfd::FileDialog::new()
        .set_title("Extract Files To...")
        .pick_folder()
    {
        Some(d) => d,
        None => {
            state.pending_extract_files.set(Vec::new());
            return;
        }
    };

    // Group by PAK file for efficient extraction
    let mut by_pak: HashMap<PathBuf, Vec<String>> = HashMap::new();
    for (internal_path, pak_path) in &pending_files {
        by_pak
            .entry(pak_path.clone())
            .or_default()
            .push(internal_path.clone());
    }

    let total_files = pending_files.len();
    let show_progress = state.show_progress;
    let selected_results = state.selected_results;

    show_progress.set(true);
    SEARCH_PROGRESS.reset();
    SEARCH_PROGRESS.set(0, total_files, "Extracting files...".to_string());

    // Clear pending files
    state.pending_extract_files.set(Vec::new());

    let send = create_ext_action(Scope::new(), move |result: Result<String, String>| {
        show_progress.set(false);
        match result {
            Ok(msg) => {
                selected_results.set(std::collections::HashSet::new()); // Clear selection
                rfd::MessageDialog::new()
                    .set_title("Extraction Complete")
                    .set_description(&msg)
                    .show();
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Extraction Failed")
                    .set_description(&e)
                    .show();
            }
        }
    });

    std::thread::spawn(move || {
        let mut total_extracted = 0;
        let mut gr2s_processed = 0;
        let mut glb_created = 0;
        let mut textures_extracted = 0;

        for (pak_path, file_paths) in by_pak {
            // Separate GR2 and non-GR2 files
            let (gr2_files, other_files): (Vec<_>, Vec<_>) = file_paths
                .iter()
                .partition(|p| p.to_lowercase().ends_with(".gr2"));

            // Extract non-GR2 files with standard extraction
            if !other_files.is_empty() {
                let paths: Vec<&str> = other_files.iter().map(|s| s.as_str()).collect();
                match PakOperations::extract_files_with_progress(
                    &pak_path,
                    &dest,
                    &paths,
                    &|progress| {
                        SEARCH_PROGRESS.set(
                            total_extracted + progress.current,
                            total_files,
                            format!(
                                "Extracting from {}",
                                pak_path.file_name().unwrap_or_default().to_string_lossy()
                            ),
                        );
                    },
                ) {
                    Ok(_) => total_extracted += paths.len(),
                    Err(e) => {
                        send(Err(e.to_string()));
                        return;
                    }
                }
            }

            // Extract GR2 files with smart extraction if options enabled
            if !gr2_files.is_empty() {
                let gr2_paths: Vec<&str> = gr2_files.iter().map(|s| s.as_str()).collect();

                if use_smart_extract {
                    // Build extraction options
                    // Note: glTF output uses the same converter but outputs .gltf + .bin instead of .glb
                    let extraction_opts = maclarian::pak::Gr2ExtractionOptions::new()
                        .with_convert_to_glb(convert_to_glb || convert_to_gltf)
                        .with_extract_textures(extract_textures)
                        .with_extract_virtual_textures(extract_textures) // Included with texture extraction
                        .with_keep_original(extract_gr2)
                        .with_png_conversion(convert_to_png)
                        .with_keep_original_dds(extract_textures) // Keep DDS if "Extract textures DDS" is checked
                        .with_game_data_path(if game_data.is_empty() {
                            None
                        } else {
                            Some(PathBuf::from(&game_data))
                        })
                        .with_virtual_textures_path(None::<std::path::PathBuf>); // Uses game data path for VT lookup

                    match maclarian::pak::extract_files_smart(
                        &pak_path,
                        &dest,
                        &gr2_paths.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                        extraction_opts,
                        &|progress| {
                            let desc = progress
                                .current_file
                                .as_deref()
                                .unwrap_or(progress.phase.as_str());
                            SEARCH_PROGRESS.set(
                                total_extracted + progress.current,
                                total_files,
                                desc.to_string(),
                            );
                        },
                    ) {
                        Ok(result) => {
                            total_extracted += gr2_paths.len();
                            gr2s_processed += result.gr2s_processed;
                            glb_created += result.glb_files_created;
                            textures_extracted += result.textures_extracted;
                        }
                        Err(e) => {
                            send(Err(e.to_string()));
                            return;
                        }
                    }
                } else {
                    // Standard extraction for GR2 files
                    match PakOperations::extract_files_with_progress(
                        &pak_path,
                        &dest,
                        &gr2_paths,
                        &|progress| {
                            SEARCH_PROGRESS.set(
                                total_extracted + progress.current,
                                total_files,
                                format!(
                                    "Extracting from {}",
                                    pak_path.file_name().unwrap_or_default().to_string_lossy()
                                ),
                            );
                        },
                    ) {
                        Ok(_) => total_extracted += gr2_paths.len(),
                        Err(e) => {
                            send(Err(e.to_string()));
                            return;
                        }
                    }
                }
            }
        }

        // Build result message
        let msg = if gr2s_processed > 0 {
            format!(
                "Extracted {} files\n{} GR2s processed, {} GLB created, {} textures extracted",
                total_extracted, gr2s_processed, glb_created, textures_extracted
            )
        } else {
            format!("Extracted {} files", total_extracted)
        };

        send(Ok(msg));
    });
}
