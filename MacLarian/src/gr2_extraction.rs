//! Smart GR2 extraction with automatic GLB conversion and texture bundling
//!
//! When extracting GR2 files from a PAK, this module can automatically:
//! 1. Convert the GR2 to GLB format
//! 2. Look up associated textures in the embedded database
//! 3. Extract those textures from their source PAKs to the same output folder
//! 4. Extract and convert virtual textures (GTP/GTS) to DDS

use crate::converter::convert_gr2_to_glb;
use crate::error::{Error, Result};
use crate::virtual_texture::VirtualTextureExtractor;
use crate::merged::{bg3_data_path, embedded_database_cached, MergedDatabase, TextureRef, VirtualTextureRef};
use crate::pak::PakOperations;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Result of a smart GR2 extraction
#[derive(Debug, Clone)]
pub struct Gr2ExtractionResult {
    /// Path to the extracted GR2 file
    pub gr2_path: PathBuf,
    /// Path to the converted GLB file (if conversion succeeded)
    pub glb_path: Option<PathBuf>,
    /// Paths to extracted texture files
    pub texture_paths: Vec<PathBuf>,
    /// Any warnings or errors that occurred during extraction
    pub warnings: Vec<String>,
}

/// Options for smart GR2 extraction
#[derive(Debug, Clone)]
pub struct Gr2ExtractionOptions {
    /// Convert GR2 to GLB automatically
    pub convert_to_glb: bool,
    /// Extract associated textures
    pub extract_textures: bool,
    /// Path to BG3 game data folder (for finding Textures.pak, etc.)
    /// If None, uses the default macOS path
    pub game_data_path: Option<PathBuf>,
    /// Path to pre-extracted virtual textures (GTP/GTS files)
    /// If None, virtual textures will be skipped
    pub virtual_textures_path: Option<PathBuf>,
    /// Use embedded database for texture lookups
    pub use_embedded_db: bool,
}

impl Default for Gr2ExtractionOptions {
    fn default() -> Self {
        Self {
            convert_to_glb: true,
            extract_textures: true,
            game_data_path: None,
            virtual_textures_path: None,
            use_embedded_db: true,
        }
    }
}

impl Gr2ExtractionOptions {
    /// Create options with custom game data path
    pub fn with_game_data_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.game_data_path = Some(path.into());
        self
    }

    /// Set path to pre-extracted virtual textures (GTP/GTS files)
    pub fn with_virtual_textures_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.virtual_textures_path = Some(path.into());
        self
    }

    /// Disable GLB conversion
    #[must_use] 
    pub fn no_conversion(mut self) -> Self {
        self.convert_to_glb = false;
        self
    }

    /// Disable texture extraction
    #[must_use] 
    pub fn no_textures(mut self) -> Self {
        self.extract_textures = false;
        self
    }
}

/// Smart extraction of a GR2 file with automatic conversion and texture bundling
///
/// Given a GR2 file path (already extracted), this function will:
/// 1. Convert it to GLB format
/// 2. Look up associated textures in the database
/// 3. Extract those textures from their source PAKs
///
/// All output files are placed in the same directory as the GR2 file.
///
/// # Errors
/// Returns an error if conversion or texture extraction fails.
pub fn process_extracted_gr2(
    gr2_path: &Path,
    options: &Gr2ExtractionOptions,
) -> Result<Gr2ExtractionResult> {
    let mut result = Gr2ExtractionResult {
        gr2_path: gr2_path.to_path_buf(),
        glb_path: None,
        texture_paths: Vec::new(),
        warnings: Vec::new(),
    };

    let output_dir = gr2_path
        .parent()
        .ok_or_else(|| Error::InvalidPath(
            "GR2 path has no parent directory".to_string()
        ))?;

    // Step 1: Convert GR2 to GLB
    if options.convert_to_glb {
        let glb_path = gr2_path.with_extension("glb");
        match convert_gr2_to_glb(gr2_path, &glb_path) {
            Ok(()) => {
                result.glb_path = Some(glb_path);
            }
            Err(e) => {
                result.warnings.push(format!("Failed to convert to GLB: {e}"));
            }
        }
    }

    // Step 2: Extract associated textures
    if options.extract_textures {
        let db = if options.use_embedded_db {
            Some(embedded_database_cached())
        } else {
            None
        };

        if let Some(db) = db {
            let textures = extract_textures_for_gr2(gr2_path, db, output_dir, options)?;
            result.texture_paths = textures;
        }
    }

    Ok(result)
}

/// Extract textures associated with a GR2 file (both regular DDS and virtual textures)
fn extract_textures_for_gr2(
    gr2_path: &Path,
    db: &MergedDatabase,
    output_dir: &Path,
    options: &Gr2ExtractionOptions,
) -> Result<Vec<PathBuf>> {
    let mut extracted_paths = Vec::new();

    // Get the GR2 filename for database lookup
    let gr2_filename = gr2_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::ConversionError("Invalid GR2 filename".to_string()))?;

    // Look up all visuals that use this GR2
    let visuals = db.get_visuals_for_gr2(gr2_filename);
    if visuals.is_empty() {
        return Ok(extracted_paths);
    }

    // Get game data path
    let game_data = options
        .game_data_path
        .clone()
        .or_else(bg3_data_path)
        .ok_or_else(|| Error::ConversionError(
            "Could not determine BG3 game data path".to_string()
        ))?;

    // Collect unique textures from all visuals
    let mut seen_textures: HashSet<String> = HashSet::new();
    let mut textures_to_extract: Vec<&TextureRef> = Vec::new();
    let mut seen_virtual_textures: HashSet<String> = HashSet::new();
    let mut virtual_textures_to_extract: Vec<&VirtualTextureRef> = Vec::new();

    for visual in &visuals {
        for texture in &visual.textures {
            if seen_textures.insert(texture.id.clone()) {
                textures_to_extract.push(texture);
            }
        }
        for vt in &visual.virtual_textures {
            if seen_virtual_textures.insert(vt.id.clone()) {
                virtual_textures_to_extract.push(vt);
            }
        }
    }

    // Extract regular DDS textures
    extracted_paths.extend(extract_dds_textures(
        &textures_to_extract,
        &game_data,
        output_dir,
    )?);

    // Extract virtual textures (only if path is configured)
    if let Some(ref vt_path) = options.virtual_textures_path {
        extracted_paths.extend(extract_virtual_textures(
            &virtual_textures_to_extract,
            db,
            vt_path,
            output_dir,
        )?);
    }

    Ok(extracted_paths)
}

/// Extract regular DDS textures from Textures.pak
fn extract_dds_textures(
    textures: &[&TextureRef],
    game_data: &Path,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut extracted_paths = Vec::new();

    // Group textures by source pak
    let mut by_pak: std::collections::HashMap<&str, Vec<&TextureRef>> = std::collections::HashMap::new();
    for texture in textures {
        let pak = if texture.source_pak.is_empty() {
            "Textures.pak"
        } else {
            &texture.source_pak
        };
        by_pak.entry(pak).or_default().push(texture);
    }

    // Extract textures from each pak
    for (pak_name, textures) in by_pak {
        let pak_path = game_data.join(pak_name);
        if !pak_path.exists() {
            tracing::warn!("Source pak not found: {}", pak_path.display());
            continue;
        }

        let dds_paths: Vec<&str> = textures.iter().map(|t| t.dds_path.as_str()).collect();

        let output_dir_buf = output_dir.to_path_buf();
        match PakOperations::extract_files(&pak_path, &output_dir_buf, &dds_paths) {
            Ok(()) => {
                for texture in &textures {
                    // The texture is extracted to output_dir/dds_path
                    // We want to flatten it to just the filename in output_dir
                    let full_extracted_path = output_dir.join(&texture.dds_path);
                    if full_extracted_path.exists() {
                        // Move/copy to output_dir with just the filename
                        let dest_path = output_dir.join(
                            Path::new(&texture.dds_path)
                                .file_name()
                                .unwrap_or_default()
                        );
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
                tracing::warn!("Failed to extract textures from {}: {}", pak_name, e);
            }
        }
    }

    Ok(extracted_paths)
}

/// Extract virtual textures from pre-extracted GTP/GTS files and convert to DDS
fn extract_virtual_textures(
    virtual_textures: &[&VirtualTextureRef],
    db: &MergedDatabase,
    vt_source_path: &Path,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut extracted_paths = Vec::new();

    if virtual_textures.is_empty() {
        return Ok(extracted_paths);
    }

    if !vt_source_path.exists() {
        tracing::warn!("Virtual textures path not found: {}", vt_source_path.display());
        return Ok(extracted_paths);
    }

    for vt in virtual_textures {
        if vt.gtex_hash.is_empty() {
            continue;
        }

        // Get the GTP path from the hash (relative path)
        let gtp_rel_path = db.pak_paths.gtp_path_from_hash(&vt.gtex_hash);
        if gtp_rel_path.is_empty() {
            tracing::warn!("Could not derive GTP path for hash: {}", vt.gtex_hash);
            continue;
        }

        // Derive the GTS path (same directory, base name without hash + .gts)
        let gts_rel_path = derive_gts_path(&gtp_rel_path);

        // When extracted, VT files are organized into subdirectories
        // e.g., Albedo_Normal_Physical_5/Albedo_Normal_Physical_5_xxx.gtp
        let gtp_extracted_path = adjust_vt_path_for_extraction(&gtp_rel_path);
        let gts_extracted_path = adjust_vt_path_for_extraction(&gts_rel_path);

        // Look for the files in the pre-extracted location
        let gtp_path = vt_source_path.join(&gtp_extracted_path);
        let gts_path = vt_source_path.join(&gts_extracted_path);

        if !gtp_path.exists() {
            tracing::warn!("GTP file not found: {}", gtp_path.display());
            continue;
        }

        if !gts_path.exists() {
            tracing::warn!("GTS file not found: {}", gts_path.display());
            continue;
        }

        // Extract and convert the virtual texture
        match VirtualTextureExtractor::extract_with_gts(&gtp_path, &gts_path, output_dir) {
            Ok(()) => {
                // The extractor creates Albedo.dds, Normal.dds, Physical.dds
                // Rename them to include the visual name
                for layer in &["Albedo", "Normal", "Physical"] {
                    let src = output_dir.join(format!("{layer}.dds"));
                    if src.exists() {
                        let dest = output_dir.join(format!("{}_{}.dds", vt.name, layer));
                        if std::fs::rename(&src, &dest).is_ok() {
                            extracted_paths.push(dest);
                        } else {
                            extracted_paths.push(src);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to extract virtual texture {}: {}", vt.name, e);
            }
        }
    }

    Ok(extracted_paths)
}

/// Adjust a virtual texture path for the extracted folder structure
/// The pak contains: `Generated/Public/VirtualTextures/Albedo_Normal_Physical_5_xxx.gtp`
/// When extracted: `Generated/Public/VirtualTextures/Albedo_Normal_Physical_5/Albedo_Normal_Physical_5_xxx.gtp`
fn adjust_vt_path_for_extraction(path: &str) -> String {
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
fn derive_gts_path(gtp_path: &str) -> String {
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

/// Recursively remove empty directories
fn cleanup_empty_dirs(dir: &Path) {
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

/// Extract a GR2 file from a PAK with automatic conversion and texture bundling
///
/// This is a convenience function that:
/// 1. Extracts the specified GR2 from the source pak
/// 2. Converts it to GLB
/// 3. Extracts associated textures
///
/// # Errors
/// Returns an error if extraction or conversion fails.
pub fn extract_gr2_with_textures(
    source_pak: &Path,
    gr2_path_in_pak: &str,
    output_dir: &Path,
    options: &Gr2ExtractionOptions,
) -> Result<Gr2ExtractionResult> {
    // Extract the GR2 file
    PakOperations::extract_files(source_pak, output_dir, &[gr2_path_in_pak])?;

    // The GR2 was extracted with its full path structure
    let extracted_gr2 = output_dir.join(gr2_path_in_pak);

    if !extracted_gr2.exists() {
        return Err(Error::ConversionError(format!(
            "GR2 file not found after extraction: {}",
            extracted_gr2.display()
        )));
    }

    // Process the extracted GR2
    process_extracted_gr2(&extracted_gr2, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = Gr2ExtractionOptions::default();
        assert!(opts.convert_to_glb);
        assert!(opts.extract_textures);
        assert!(opts.use_embedded_db);
        assert!(opts.game_data_path.is_none());
    }

    #[test]
    fn test_options_builder() {
        let opts = Gr2ExtractionOptions::default()
            .no_conversion()
            .no_textures();
        assert!(!opts.convert_to_glb);
        assert!(!opts.extract_textures);
    }
}
