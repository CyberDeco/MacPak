//! DDS texture extraction from PAK files

use crate::error::Result;
use crate::merged::TextureRef;
use crate::pak::PakOperations;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Extract regular DDS textures from Textures.pak (or Textures_*.pak on macOS)
pub fn extract_dds_textures(
    textures: &[&TextureRef],
    game_data: &Path,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut extracted_paths = Vec::new();

    if textures.is_empty() {
        return Ok(extracted_paths);
    }

    // Log the textures we're trying to extract
    for tex in textures {
        tracing::info!(
            "Looking for texture: {} -> {} (source_pak: '{}')",
            tex.name,
            tex.dds_path,
            tex.source_pak
        );
    }

    // Find all texture PAK files in game_data folder
    // On macOS: Textures_1.pak, Textures_2.pak, Textures_3.pak
    // On other platforms: Textures.pak
    let texture_paks = find_texture_paks(game_data);
    tracing::info!(
        "Found {} texture PAK files: {:?}",
        texture_paks.len(),
        texture_paks
    );
    if texture_paks.is_empty() {
        tracing::warn!("No texture PAK files found in: {}", game_data.display());
        return Ok(extracted_paths);
    }

    // Group textures by source pak (if known)
    let mut by_pak: std::collections::HashMap<&str, Vec<&TextureRef>> =
        std::collections::HashMap::new();
    let mut unknown_pak: Vec<&TextureRef> = Vec::new();

    for texture in textures {
        if texture.source_pak.is_empty() {
            unknown_pak.push(texture);
        } else {
            by_pak.entry(&texture.source_pak).or_default().push(texture);
        }
    }

    // Extract textures with known source PAK
    for (pak_name, textures) in by_pak {
        let pak_path = game_data.join(pak_name);
        if !pak_path.exists() {
            tracing::warn!("Source pak not found: {}", pak_path.display());
            continue;
        }
        extract_textures_from_pak(&pak_path, &textures, output_dir, &mut extracted_paths);
    }

    // Extract textures with unknown source PAK - search across all texture PAKs
    if !unknown_pak.is_empty() {
        tracing::info!(
            "Searching {} texture PAKs for {} textures",
            texture_paks.len(),
            unknown_pak.len()
        );

        // Try each texture PAK until target file(s) found
        for pak_path in &texture_paks {
            // Check which files exist in this PAK
            let pak_files: HashSet<String> = match PakOperations::list(pak_path) {
                Ok(files) => {
                    tracing::debug!("PAK {} has {} files", pak_path.display(), files.len());
                    files.into_iter().collect()
                }
                Err(e) => {
                    tracing::warn!("Failed to list PAK {}: {}", pak_path.display(), e);
                    continue;
                }
            };

            // Find textures that exist in this PAK
            let textures_in_pak: Vec<&TextureRef> = unknown_pak
                .iter()
                .filter(|t| {
                    let found = pak_files.contains(&t.dds_path);
                    if found {
                        tracing::info!("Found texture '{}' in {}", t.dds_path, pak_path.display());
                    }
                    found
                })
                .copied()
                .collect();

            if !textures_in_pak.is_empty() {
                tracing::info!(
                    "Extracting {} textures from {}",
                    textures_in_pak.len(),
                    pak_path.display()
                );
                extract_textures_from_pak(
                    pak_path,
                    &textures_in_pak,
                    output_dir,
                    &mut extracted_paths,
                );
            }
        }

        // Log if any textures weren't found
        let found_paths: HashSet<&str> = extracted_paths
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .collect();
        for tex in &unknown_pak {
            let tex_filename = Path::new(&tex.dds_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&tex.dds_path);
            if !found_paths.contains(tex_filename) {
                tracing::warn!("Texture not found in any PAK: {}", tex.dds_path);
            }
        }
    }

    Ok(extracted_paths)
}

/// Find all texture PAK files in game data folder
pub fn find_texture_paks(game_data: &Path) -> Vec<PathBuf> {
    let mut paks = Vec::new();

    // Try single Textures.pak first (Windows/Linux)
    let single_pak = game_data.join("Textures.pak");
    if single_pak.exists() {
        paks.push(single_pak);
        return paks;
    }

    // Look for split texture PAKs (macOS: Textures_1.pak, Textures_2.pak, etc.)
    if let Ok(entries) = std::fs::read_dir(game_data) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("Textures_") && name.ends_with(".pak") {
                    paks.push(path);
                }
            }
        }
    }

    // Sort for consistent ordering
    paks.sort();
    paks
}

/// Extract textures from a single PAK file
pub fn extract_textures_from_pak(
    pak_path: &Path,
    textures: &[&TextureRef],
    output_dir: &Path,
    extracted_paths: &mut Vec<PathBuf>,
) {
    let dds_paths: Vec<&str> = textures.iter().map(|t| t.dds_path.as_str()).collect();

    match PakOperations::extract_files(pak_path, output_dir, &dds_paths) {
        Ok(()) => {
            for texture in textures {
                // The texture is extracted to output_dir/dds_path
                // We want to flatten it to just the filename in output_dir
                let full_extracted_path = output_dir.join(&texture.dds_path);
                if full_extracted_path.exists() {
                    // Move/copy to output_dir with just the filename
                    let dest_path = output_dir
                        .join(Path::new(&texture.dds_path).file_name().unwrap_or_default());
                    if let Err(e) = std::fs::rename(&full_extracted_path, &dest_path) {
                        // If rename fails (cross-device), try copy
                        if let Err(e2) = std::fs::copy(&full_extracted_path, &dest_path) {
                            tracing::warn!(
                                "Failed to move texture {} to output: {} / {}",
                                texture.name,
                                e,
                                e2
                            );
                            continue;
                        }
                        let _ = std::fs::remove_file(&full_extracted_path);
                    }
                    extracted_paths.push(dest_path);
                }
            }

            // Clean up any empty intermediate directories
            cleanup_empty_dirs(output_dir);
        }
        Err(e) => {
            tracing::warn!(
                "Failed to extract textures from {}: {}",
                pak_path.display(),
                e
            );
        }
    }
}

/// Recursively remove empty directories
pub fn cleanup_empty_dirs(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                cleanup_empty_dirs(&path);
                // Try to remove the directory if it's empty
                let _ = std::fs::remove_dir(&path);
            }
        }
    }
}
