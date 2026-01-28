//! Virtual texture extraction and conversion

use crate::error::{Error, Result};
use crate::merged::{MergedDatabase, VirtualTextureRef};
use crate::pak::PakOperations;
use crate::virtual_texture::VirtualTextureExtractor;
use std::path::{Path, PathBuf};

/// Extract virtual textures and convert to DDS
///
/// This function can extract from:
/// 1. Pre-extracted GTP/GTS files (if vt_source_path points to extracted files)
/// 2. VirtualTextures.pak directly (if vt_source_path is None, uses game_data path)
pub fn extract_virtual_textures(
    virtual_textures: &[&VirtualTextureRef],
    db: &MergedDatabase,
    vt_source_path: Option<&Path>,
    game_data: &Path,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut extracted_paths = Vec::new();

    if virtual_textures.is_empty() {
        return Ok(extracted_paths);
    }

    // Determine if we're using pre-extracted files or extracting from PAK
    let use_pak = vt_source_path.is_none() || !vt_source_path.unwrap().exists();

    let vt_pak_path = game_data.join("VirtualTextures.pak");
    if use_pak && !vt_pak_path.exists() {
        tracing::warn!("VirtualTextures.pak not found: {}", vt_pak_path.display());
        return Ok(extracted_paths);
    }

    // Collect hashes for batch lookup
    let hashes: Vec<&str> = virtual_textures
        .iter()
        .filter(|vt| !vt.gtex_hash.is_empty())
        .map(|vt| vt.gtex_hash.as_str())
        .collect();

    if use_pak {
        // Use existing functionality to find GTP files in PAK
        let gtp_matches = find_gtp_files_in_pak(&vt_pak_path, &hashes)?;
        tracing::info!("Found {} GTP matches in VirtualTextures.pak", gtp_matches.len());

        for vt in virtual_textures {
            if vt.gtex_hash.is_empty() {
                continue;
            }

            // Find the GTP path for this hash
            let gtp_match = gtp_matches.iter().find(|(hash, _)| *hash == vt.gtex_hash);
            let Some((_, gtp_rel_path)) = gtp_match else {
                tracing::warn!("GTP not found for hash {}", vt.gtex_hash);
                continue;
            };

            // Derive the GTS path from the GTP path
            let gts_rel_path = derive_gts_path(gtp_rel_path);

            tracing::info!("Virtual texture {}: GTP={}, GTS={}", vt.name, gtp_rel_path, gts_rel_path);

            match extract_virtual_texture_from_pak(
                &vt_pak_path,
                gtp_rel_path,
                &gts_rel_path,
                &vt.name,
                output_dir,
            ) {
                Ok(paths) => extracted_paths.extend(paths),
                Err(e) => {
                    tracing::warn!("Failed to extract virtual texture {} from PAK: {}", vt.name, e);
                }
            }
        }
    } else {
        // Use pre-extracted files
        let vt_source = vt_source_path.unwrap();

        for vt in virtual_textures {
            if vt.gtex_hash.is_empty() {
                continue;
            }

            // Get the GTP path from the hash
            let gtp_rel_path = db.pak_paths.gtp_path_from_hash(&vt.gtex_hash);
            if gtp_rel_path.is_empty() {
                tracing::warn!("Could not derive GTP path for hash: {}", vt.gtex_hash);
                continue;
            }

            let gts_rel_path = derive_gts_path(&gtp_rel_path);

            // When extracted, VT files are organized into subdirectories
            let gtp_extracted_path = adjust_vt_path_for_extraction(&gtp_rel_path);
            let gts_extracted_path = adjust_vt_path_for_extraction(&gts_rel_path);

            let gtp_path = vt_source.join(&gtp_extracted_path);
            let gts_path = vt_source.join(&gts_extracted_path);

            if !gtp_path.exists() {
                tracing::warn!("GTP file not found: {}", gtp_path.display());
                continue;
            }

            if !gts_path.exists() {
                tracing::warn!("GTS file not found: {}", gts_path.display());
                continue;
            }

            match extract_and_rename_virtual_texture(&gtp_path, &gts_path, &vt.name, output_dir) {
                Ok(paths) => extracted_paths.extend(paths),
                Err(e) => {
                    tracing::warn!("Failed to extract virtual texture {}: {}", vt.name, e);
                }
            }
        }
    }

    Ok(extracted_paths)
}

/// Find GTP files in VirtualTextures.pak matching the given hashes
pub fn find_gtp_files_in_pak(pak_path: &Path, hashes: &[&str]) -> Result<Vec<(String, String)>> {
    let all_files = PakOperations::list(pak_path)?;
    let mut matches = Vec::new();

    for file_path in &all_files {
        if !file_path.to_lowercase().ends_with(".gtp") {
            continue;
        }

        let filename = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let stem = filename.strip_suffix(".gtp").unwrap_or(filename);

        for hash in hashes {
            if stem.ends_with(hash) {
                matches.push(((*hash).to_string(), file_path.clone()));
                break;
            }
        }
    }

    Ok(matches)
}

/// Extract a virtual texture directly from VirtualTextures.pak
///
/// Uses `read_file_bytes` which correctly handles split PAK archives via `archive_part`.
pub fn extract_virtual_texture_from_pak(
    vt_pak_path: &Path,
    gtp_rel_path: &str,
    gts_rel_path: &str,
    vt_name: &str,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    // Create a unique temp directory for this extraction (supports parallel processing)
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique_id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let temp_dir = std::env::temp_dir().join(format!(
        "maclarian_vt_{}_{}",
        std::process::id(),
        unique_id
    ));
    std::fs::create_dir_all(&temp_dir)?;

    tracing::info!("Reading virtual texture files from PAK: GTP={}, GTS={}", gtp_rel_path, gts_rel_path);

    // Read files using read_file_bytes which handles split PAKs correctly
    let gtp_data = match PakOperations::read_file_bytes(vt_pak_path, gtp_rel_path) {
        Ok(data) => data,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(Error::ConversionError(format!(
                "Failed to read GTP from PAK: {}", e
            )));
        }
    };

    let gts_data = match PakOperations::read_file_bytes(vt_pak_path, gts_rel_path) {
        Ok(data) => data,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(Error::ConversionError(format!(
                "Failed to read GTS from PAK: {}", e
            )));
        }
    };

    // Write to temp files
    let gtp_path = temp_dir.join(
        Path::new(gtp_rel_path).file_name().unwrap_or_default()
    );
    let gts_path = temp_dir.join(
        Path::new(gts_rel_path).file_name().unwrap_or_default()
    );

    std::fs::write(&gtp_path, &gtp_data)?;
    std::fs::write(&gts_path, &gts_data)?;

    tracing::info!("Wrote temp files: GTP={} ({} bytes), GTS={} ({} bytes)",
        gtp_path.display(), gtp_data.len(),
        gts_path.display(), gts_data.len()
    );

    // Extract and convert
    let result = extract_and_rename_virtual_texture(&gtp_path, &gts_path, vt_name, output_dir);

    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    result
}

/// Extract virtual texture and rename output files with the visual name
pub fn extract_and_rename_virtual_texture(
    gtp_path: &Path,
    gts_path: &Path,
    vt_name: &str,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut extracted_paths = Vec::new();

    // Extract and convert the virtual texture
    VirtualTextureExtractor::extract_with_gts(gtp_path, gts_path, output_dir)?;

    // The extractor creates BaseMap.dds, NormalMap.dds, PhysicalMap.dds
    // Rename them to include the visual name
    for layer in &["BaseMap", "NormalMap", "PhysicalMap"] {
        let src = output_dir.join(format!("{layer}.dds"));
        if src.exists() {
            let dest = output_dir.join(format!("{}_{}.dds", vt_name, layer));
            if std::fs::rename(&src, &dest).is_ok() {
                extracted_paths.push(dest);
            } else {
                extracted_paths.push(src);
            }
        }
    }

    Ok(extracted_paths)
}

/// Adjust a virtual texture path for the extracted folder structure
/// The pak contains: `Generated/Public/VirtualTextures/Albedo_Normal_Physical_5_xxx.gtp`
/// When extracted: `Generated/Public/VirtualTextures/Albedo_Normal_Physical_5/Albedo_Normal_Physical_5_xxx.gtp`
pub fn adjust_vt_path_for_extraction(path: &str) -> String {
    // Get the filename
    let path_obj = std::path::Path::new(path);
    let Some(filename) = path_obj.file_name().and_then(|f| f.to_str()) else {
        return path.to_string();
    };

    // Get the parent directory
    let Some(parent) = path_obj.parent().and_then(|p| p.to_str()) else {
        return path.to_string();
    };

    // Extract subfolder name from filename
    // e.g., "Albedo_Normal_Physical_5_xxx.gtp" -> "Albedo_Normal_Physical_5"
    // or "Albedo_Normal_Physical_5.gts" -> "Albedo_Normal_Physical_5"
    let stem = filename.trim_end_matches(".gtp").trim_end_matches(".gts");

    let subfolder = if let Some(last_underscore) = stem.rfind('_') {
        let suffix = &stem[last_underscore + 1..];
        // If suffix is a 32-char hash, remove it to get subfolder name
        if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            &stem[..last_underscore]
        } else {
            stem
        }
    } else {
        stem
    };

    // Build new path with subfolder
    format!("{parent}/{subfolder}/{filename}")
}

/// Derive the GTS path from a GTP path
/// GTP: "`Generated/Public/VirtualTextures/Albedo_Normal_Physical_0_abc123...def.gtp`"
/// GTS: "`Generated/Public/VirtualTextures/Albedo_Normal_Physical_0.gts`"
pub fn derive_gts_path(gtp_path: &str) -> String {
    // Remove the .gtp extension
    let without_ext = gtp_path.trim_end_matches(".gtp");

    // Find the last underscore (before the hash)
    if let Some(last_underscore) = without_ext.rfind('_') {
        let suffix = &without_ext[last_underscore + 1..];
        // Check if suffix looks like a hash (32 hex chars)
        if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            // Remove the hash and add .gts
            return format!("{}.gts", &without_ext[..last_underscore]);
        }
    }

    // Fallback: just replace extension
    format!("{without_ext}.gts")
}
