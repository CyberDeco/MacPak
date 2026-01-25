//! Smart GR2 extraction with automatic GLB conversion and texture bundling
//!
//! When extracting GR2 files from a PAK, this module can automatically:
//! 1. Convert the GR2 to GLB format
//! 2. Look up associated textures via [`GameDataResolver`](crate::merged::GameDataResolver)
//! 3. Extract those textures from their source PAKs to the same output folder
//! 4. Extract and convert virtual textures (GTP/GTS) to DDS
//!
//! The texture database is built on-the-fly from the game's `Shared.pak` file.
//! Use `--bg3-path` CLI flag to specify the game installation path if auto-detection fails.

#![allow(
    clippy::struct_excessive_bools,
    clippy::collapsible_if,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::option_if_let_else,
    clippy::redundant_closure_for_method_calls,
    clippy::uninlined_format_args,
    clippy::return_self_not_must_use,
    clippy::map_unwrap_or
)]

use crate::converter::{convert_gr2_to_glb, convert_dds_to_png};
use crate::error::{Error, Result};
use crate::virtual_texture::VirtualTextureExtractor;
use crate::merged::{bg3_data_path, GameDataResolver, MergedDatabase, TextureRef, VirtualTextureRef};
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
    /// Extract virtual textures (GTex files) associated with each GR2 file
    pub extract_virtual_textures: bool,
    /// Path to BG3 install folder (for finding Textures.pak, etc.)
    /// If None, auto-detects using GameDataResolver
    pub game_data_path: Option<PathBuf>,
    /// Path to pre-extracted virtual textures (GTP/GTS files)
    /// If None, virtual textures will be skipped
    pub virtual_textures_path: Option<PathBuf>,
    /// Keep the original GR2 file after conversion to GLB (default: true)
    pub keep_original_gr2: bool,
    /// Convert extracted DDS textures to PNG format
    pub convert_to_png: bool,
    /// Keep the original DDS files when converting to PNG
    pub keep_original_dds: bool,
}

impl Default for Gr2ExtractionOptions {
    fn default() -> Self {
        Self {
            convert_to_glb: true,
            extract_textures: true,
            extract_virtual_textures: false,
            game_data_path: None,
            virtual_textures_path: None,
            keep_original_gr2: true,
            convert_to_png: false,
            keep_original_dds: false,
        }
    }
}

impl Gr2ExtractionOptions {
    /// Create new options with all processing disabled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            convert_to_glb: false,
            extract_textures: false,
            extract_virtual_textures: false,
            game_data_path: None,
            virtual_textures_path: None,
            keep_original_gr2: true,
            convert_to_png: false,
            keep_original_dds: false,
        }
    }

    /// Create options with all GR2 processing enabled (bundle mode).
    ///
    /// This is equivalent to the `--bundle` CLI flag.
    #[must_use]
    pub fn bundle() -> Self {
        Self {
            convert_to_glb: true,
            extract_textures: true,
            extract_virtual_textures: true,
            game_data_path: None,
            virtual_textures_path: None,
            keep_original_gr2: true,
            convert_to_png: false,
            keep_original_dds: false,
        }
    }

    /// Check if any GR2 processing options are enabled.
    #[must_use]
    pub fn has_gr2_processing(&self) -> bool {
        self.convert_to_glb || self.extract_textures || self.extract_virtual_textures
    }

    /// Create options with custom game data path
    #[must_use]
    pub fn with_game_data_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.game_data_path = Some(path.into());
        self
    }

    /// Set path to pre-extracted virtual textures (GTP/GTS files)
    #[must_use]
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

    /// Enable PNG conversion for extracted DDS textures
    #[must_use]
    pub fn with_png_conversion(mut self, convert: bool) -> Self {
        self.convert_to_png = convert;
        self
    }

    /// Set whether to convert GR2 to GLB.
    #[must_use]
    pub fn with_convert_to_glb(mut self, convert: bool) -> Self {
        self.convert_to_glb = convert;
        self
    }

    /// Set whether to extract DDS textures.
    #[must_use]
    pub fn with_extract_textures(mut self, extract: bool) -> Self {
        self.extract_textures = extract;
        self
    }

    /// Set whether to extract virtual textures.
    #[must_use]
    pub fn with_extract_virtual_textures(mut self, extract: bool) -> Self {
        self.extract_virtual_textures = extract;
        self
    }

    /// Set whether to keep the original GR2 after conversion.
    #[must_use]
    pub fn with_keep_original(mut self, keep: bool) -> Self {
        self.keep_original_gr2 = keep;
        self
    }

    /// Set whether to keep original DDS files after PNG conversion.
    #[must_use]
    pub fn with_keep_original_dds(mut self, keep: bool) -> Self {
        self.keep_original_dds = keep;
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
        // Build resolver from game data path or auto-detect
        let resolver = if let Some(ref game_data) = options.game_data_path {
            GameDataResolver::new(game_data).ok()
        } else {
            GameDataResolver::auto_detect().ok()
        };

        if let Some(resolver) = resolver {
            let textures = extract_textures_for_gr2(gr2_path, resolver.database(), output_dir, options)?;

            // Convert DDS to PNG if requested
            if options.convert_to_png {
                let mut png_paths = Vec::new();
                for dds_path in &textures {
                    let is_dds = dds_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("dds"))
                        .unwrap_or(false);
                    if is_dds {
                        let png_path = dds_path.with_extension("png");
                        match convert_dds_to_png(dds_path, &png_path) {
                            Ok(()) => {
                                tracing::info!("Converted {} to PNG", dds_path.display());
                                // Remove the DDS file after successful conversion (unless keep_original_dds)
                                if !options.keep_original_dds {
                                    let _ = std::fs::remove_file(dds_path);
                                }
                                png_paths.push(png_path);
                            }
                            Err(e) => {
                                result.warnings.push(format!(
                                    "Failed to convert {} to PNG: {}",
                                    dds_path.display(),
                                    e
                                ));
                                // Keep the DDS file in the result
                                png_paths.push(dds_path.clone());
                            }
                        }
                    } else {
                        png_paths.push(dds_path.clone());
                    }
                }
                result.texture_paths = png_paths;
            } else {
                result.texture_paths = textures;
            }
        } else {
            result.warnings.push(
                "Could not find BG3 install path for texture lookup. Use --bg3-path to specify the path.".to_string()
            );
        }
    }

    Ok(result)
}

/// Process an extracted GR2 file with a custom output directory.
///
/// Same as `process_extracted_gr2` but outputs to the specified directory
/// instead of the GR2 file's parent directory.
pub fn process_extracted_gr2_to_dir(
    gr2_path: &Path,
    output_dir: &Path,
    options: &Gr2ExtractionOptions,
) -> Result<Gr2ExtractionResult> {
    let mut result = Gr2ExtractionResult {
        gr2_path: gr2_path.to_path_buf(),
        glb_path: None,
        texture_paths: Vec::new(),
        warnings: Vec::new(),
    };

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir).map_err(|e| {
        Error::ConversionError(format!("Failed to create output directory: {e}"))
    })?;

    // Step 1: Convert GR2 to GLB (if requested)
    if options.convert_to_glb {
        let glb_name = gr2_path.file_stem().unwrap_or_default();
        let glb_path = output_dir.join(format!("{}.glb", glb_name.to_string_lossy()));
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
        // Build resolver from game data path or auto-detect
        let resolver = if let Some(ref game_data) = options.game_data_path {
            GameDataResolver::new(game_data).ok()
        } else {
            GameDataResolver::auto_detect().ok()
        };

        if let Some(resolver) = resolver {
            let textures = extract_textures_for_gr2(gr2_path, resolver.database(), output_dir, options)?;

            // Convert DDS to PNG if requested
            if options.convert_to_png {
                let mut png_paths = Vec::new();
                for dds_path in &textures {
                    let is_dds = dds_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("dds"))
                        .unwrap_or(false);
                    if is_dds {
                        let png_path = dds_path.with_extension("png");
                        match convert_dds_to_png(dds_path, &png_path) {
                            Ok(()) => {
                                tracing::info!("Converted {} to PNG", dds_path.display());
                                if !options.keep_original_dds {
                                    let _ = std::fs::remove_file(dds_path);
                                }
                                png_paths.push(png_path);
                            }
                            Err(e) => {
                                result.warnings.push(format!(
                                    "Failed to convert {} to PNG: {}",
                                    dds_path.display(),
                                    e
                                ));
                                png_paths.push(dds_path.clone());
                            }
                        }
                    } else {
                        png_paths.push(dds_path.clone());
                    }
                }
                result.texture_paths = png_paths;
            } else {
                result.texture_paths = textures;
            }
        } else {
            result.warnings.push(
                "Could not find BG3 install path for texture lookup. Use --bg3-path to specify the path.".to_string()
            );
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
    tracing::info!(
        "Found {} visuals for GR2 '{}' in database",
        visuals.len(),
        gr2_filename
    );
    if visuals.is_empty() {
        return Ok(extracted_paths);
    }

    // Get game data path
    let game_data = options
        .game_data_path
        .clone()
        .or_else(bg3_data_path)
        .ok_or_else(|| Error::ConversionError(
            "Could not determine BG3 install path".to_string()
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

    tracing::info!(
        "Textures to extract: {} regular, {} virtual",
        textures_to_extract.len(),
        virtual_textures_to_extract.len()
    );

    // Extract regular DDS textures
    extracted_paths.extend(extract_dds_textures(
        &textures_to_extract,
        &game_data,
        output_dir,
    )?);

    // Extract virtual textures (from PAK or pre-extracted path)
    if !virtual_textures_to_extract.is_empty() {
        extracted_paths.extend(extract_virtual_textures(
            &virtual_textures_to_extract,
            db,
            options.virtual_textures_path.as_deref(),
            &game_data,
            output_dir,
        )?);
    }

    Ok(extracted_paths)
}

/// Extract regular DDS textures from Textures.pak (or Textures_*.pak on macOS)
fn extract_dds_textures(
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
    tracing::info!("Found {} texture PAK files: {:?}", texture_paks.len(), texture_paks);
    if texture_paks.is_empty() {
        tracing::warn!("No texture PAK files found in: {}", game_data.display());
        return Ok(extracted_paths);
    }

    // Group textures by source pak (if known)
    let mut by_pak: std::collections::HashMap<&str, Vec<&TextureRef>> = std::collections::HashMap::new();
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
        tracing::info!("Searching {} texture PAKs for {} textures", texture_paks.len(), unknown_pak.len());

        // Try each texture PAK until we find the files
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
                tracing::info!("Extracting {} textures from {}", textures_in_pak.len(), pak_path.display());
                extract_textures_from_pak(pak_path, &textures_in_pak, output_dir, &mut extracted_paths);
            }
        }

        // Log if any textures weren't found
        let found_paths: HashSet<&str> = extracted_paths.iter()
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
fn find_texture_paks(game_data: &Path) -> Vec<std::path::PathBuf> {
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
fn extract_textures_from_pak(
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
            tracing::warn!("Failed to extract textures from {}: {}", pak_path.display(), e);
        }
    }
}

/// Extract virtual textures and convert to DDS
///
/// This function can extract from:
/// 1. Pre-extracted GTP/GTS files (if vt_source_path points to extracted files)
/// 2. VirtualTextures.pak directly (if vt_source_path is None, uses game_data path)
fn extract_virtual_textures(
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
fn find_gtp_files_in_pak(pak_path: &Path, hashes: &[&str]) -> Result<Vec<(String, String)>> {
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
fn extract_virtual_texture_from_pak(
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
fn extract_and_rename_virtual_texture(
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
        assert!(opts.game_data_path.is_none());
        assert!(opts.virtual_textures_path.is_none());
    }

    #[test]
    fn test_options_builder() {
        let opts = Gr2ExtractionOptions::default()
            .no_conversion()
            .no_textures();
        assert!(!opts.convert_to_glb);
        assert!(!opts.extract_textures);
    }

    #[test]
    fn test_options_with_game_data() {
        let opts = Gr2ExtractionOptions::default()
            .with_game_data_path("/path/to/game");
        assert_eq!(opts.game_data_path, Some(PathBuf::from("/path/to/game")));
    }
}
