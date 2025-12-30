//! Virtual Texture extraction operations

use std::path::Path;

use floem_reactive::{SignalGet, SignalUpdate};

use crate::state::VirtualTexturesState;
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
        let mut success_count = 0;
        let mut error_count = 0;
        let mut texture_count = 0;
        let mut results = Vec::new();

        for (i, gts_path) in files.iter().enumerate() {
            let gts_name = Path::new(&gts_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            progress.update(i, total, &gts_name);

            match extract_gts_file(gts_path, output_dir.as_deref(), progress) {
                Ok(count) => {
                    results.push(format!("Extracted {} textures from {}", count, gts_name));
                    success_count += 1;
                    texture_count += count;
                }
                Err(e) => {
                    results.push(format!("Failed {}: {}", gts_name, e));
                    error_count += 1;
                }
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

/// Find the GTS file for a given path (handles both .gts and .gtp files)
fn find_gts_path(input_path: &str) -> Result<String, String> {
    let path = Path::new(input_path);
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let parent = path.parent().unwrap_or(Path::new("."));

    if ext == "gts" {
        // Check if this GTS file has valid GRPG header
        if let Ok(data) = std::fs::read(input_path) {
            if data.len() >= 4 && &data[0..4] == b"GRPG" {
                return Ok(input_path.to_string());
            }
            // NULL-padded GTS - try to find the _0.gts version
            let stem = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(base) = find_base_name(&stem) {
                let gts_0_path = parent.join(format!("{}_0.gts", base));
                if gts_0_path.exists() {
                    return Ok(gts_0_path.to_string_lossy().to_string());
                }
            }
        }
        return Ok(input_path.to_string());
    }

    if ext == "gtp" {
        // Find associated GTS file
        let stem = path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // GTP files have pattern: BaseName_N_<hash>.gtp where N is index, hash is 32 hex chars
        // Example: Albedo_Normal_Physical_0_0a0d1854395eb40436bec69fe14aa92b.gtp
        // Each tile set index has its own GTS file: BaseName_N.gts

        // Strip the hash suffix first
        let name_without_hash = if let Some(last_underscore) = stem.rfind('_') {
            let suffix = &stem[last_underscore + 1..];
            if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
                &stem[..last_underscore]
            } else {
                stem.as_str()
            }
        } else {
            stem.as_str()
        };

        // First try the exact matching GTS file (e.g., Albedo_Normal_Physical_1.gts for _1_*.gtp)
        let gts_path = parent.join(format!("{}.gts", name_without_hash));
        if gts_path.exists() {
            // Check if it has a valid GRPG header
            if let Ok(data) = std::fs::read(&gts_path) {
                if data.len() >= 4 && &data[0..4] == b"GRPG" {
                    return Ok(gts_path.to_string_lossy().to_string());
                }
                // NULL-padded GTS - can't use it directly, try _0.gts instead
            }
        }

        // Try _0.gts as fallback (has full metadata but may not list this GTP)
        if let Some(base) = find_base_name(name_without_hash) {
            let gts_0_path = parent.join(format!("{}_0.gts", base));
            if gts_0_path.exists() {
                return Ok(gts_0_path.to_string_lossy().to_string());
            }
        }

        // Look for any valid GTS file in the same directory that shares the base prefix
        if let Ok(entries) = std::fs::read_dir(parent) {
            let gtp_prefix = stem.split('_').take(3).collect::<Vec<_>>().join("_");
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.extension().map(|e| e.to_string_lossy().to_lowercase()) == Some("gts".to_string()) {
                    if let Some(gts_stem) = entry_path.file_stem() {
                        let gts_name = gts_stem.to_string_lossy();
                        if gts_name.starts_with(&gtp_prefix) {
                            // Check for valid GRPG header
                            if let Ok(data) = std::fs::read(&entry_path) {
                                if data.len() >= 4 && &data[0..4] == b"GRPG" {
                                    return Ok(entry_path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        return Err(format!("Could not find associated GTS file for {}", input_path));
    }

    Err(format!("Unsupported file type: {}", ext))
}

/// Extract the base name from a virtual texture filename
/// e.g., "Albedo_Normal_Physical_1" -> Some("Albedo_Normal_Physical")
fn find_base_name(name: &str) -> Option<&str> {
    // Check if name ends with _N where N is a digit
    if let Some(last_underscore) = name.rfind('_') {
        let suffix = &name[last_underscore + 1..];
        if suffix.chars().all(|c| c.is_ascii_digit()) {
            return Some(&name[..last_underscore]);
        }
    }
    None
}

/// Extract textures from a GTS/GTP file to the output directory
fn extract_gts_file(
    input_path: &str,
    output_dir: Option<&str>,
    progress: &super::types::SharedProgress,
) -> Result<usize, String> {
    use MacLarian::formats::virtual_texture::{VirtualTextureExtractor, GtsFile};

    let input_ext = Path::new(input_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let is_single_gtp = input_ext == "gtp";

    // Find the GTS file (handles both .gts and .gtp inputs)
    let gts_path = find_gts_path(input_path)?;

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

        VirtualTextureExtractor::extract_with_gts(
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

                match VirtualTextureExtractor::extract_with_gts(
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
