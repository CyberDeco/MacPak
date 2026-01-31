//! Smart GR2 extraction with automatic GLB conversion and texture bundling
//!
//! When extracting GR2 files from a PAK, this module can automatically:
//! 1. Convert the GR2 to GLB format
//! 2. Look up associated textures via [`GameDataResolver`]
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

mod dds;
mod types;
mod virtual_textures;

pub use dds::{
    cleanup_empty_dirs, extract_dds_textures, extract_textures_from_pak, find_texture_paks,
};
pub use types::{
    Gr2ExtractionOptions, Gr2ExtractionPhase, Gr2ExtractionProgress, Gr2ExtractionProgressCallback,
    Gr2ExtractionResult,
};
pub use virtual_textures::{
    adjust_vt_path_for_extraction, derive_gts_path, extract_and_rename_virtual_texture,
    extract_virtual_texture_from_pak, extract_virtual_textures, find_gtp_files_in_pak,
};

use crate::converter::{convert_dds_to_png, convert_gr2_to_glb};
use crate::error::{Error, Result};
use crate::merged::{
    GameDataResolver, MergedDatabase, TextureRef, VirtualTextureRef, bg3_data_path,
};
use crate::pak::PakOperations;
use std::collections::HashSet;
use std::path::Path;

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
        .ok_or_else(|| Error::InvalidPath("GR2 path has no parent directory".to_string()))?;

    // Step 1: Convert GR2 to GLB
    if options.convert_to_glb {
        let glb_path = gr2_path.with_extension("glb");
        match convert_gr2_to_glb(gr2_path, &glb_path) {
            Ok(()) => {
                result.glb_path = Some(glb_path);
            }
            Err(e) => {
                result
                    .warnings
                    .push(format!("Failed to convert to GLB: {e}"));
            }
        }
    }

    // Step 2: Extract associated textures
    if options.extract_textures {
        // Build resolver from game data path or auto-detect
        let resolver = if let Some(ref game_data) = options.bg3_path {
            GameDataResolver::new(game_data).ok()
        } else {
            GameDataResolver::auto_detect().ok()
        };

        if let Some(resolver) = resolver {
            let textures =
                extract_textures_for_gr2(gr2_path, resolver.database(), output_dir, options)?;

            // Convert DDS to PNG if requested
            if options.convert_to_png {
                result.texture_paths =
                    convert_textures_to_png(&textures, options, &mut result.warnings);
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
    std::fs::create_dir_all(output_dir)
        .map_err(|e| Error::ConversionError(format!("Failed to create output directory: {e}")))?;

    // Step 1: Convert GR2 to GLB (if requested)
    if options.convert_to_glb {
        let glb_name = gr2_path.file_stem().unwrap_or_default();
        let glb_path = output_dir.join(format!("{}.glb", glb_name.to_string_lossy()));
        match convert_gr2_to_glb(gr2_path, &glb_path) {
            Ok(()) => {
                result.glb_path = Some(glb_path);
            }
            Err(e) => {
                result
                    .warnings
                    .push(format!("Failed to convert to GLB: {e}"));
            }
        }
    }

    // Step 2: Extract associated textures
    if options.extract_textures {
        // Build resolver from game data path or auto-detect
        let resolver = if let Some(ref game_data) = options.bg3_path {
            GameDataResolver::new(game_data).ok()
        } else {
            GameDataResolver::auto_detect().ok()
        };

        if let Some(resolver) = resolver {
            let textures =
                extract_textures_for_gr2(gr2_path, resolver.database(), output_dir, options)?;

            // Convert DDS to PNG if requested
            if options.convert_to_png {
                result.texture_paths =
                    convert_textures_to_png(&textures, options, &mut result.warnings);
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

/// Extract textures associated with a GR2 file (both regular DDS and virtual textures)
fn extract_textures_for_gr2(
    gr2_path: &Path,
    db: &MergedDatabase,
    output_dir: &Path,
    options: &Gr2ExtractionOptions,
) -> Result<Vec<std::path::PathBuf>> {
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
        .bg3_path
        .clone()
        .or_else(bg3_data_path)
        .ok_or_else(|| {
            Error::ConversionError("Could not determine BG3 install path".to_string())
        })?;

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

/// Convert DDS textures to PNG format
fn convert_textures_to_png(
    textures: &[std::path::PathBuf],
    options: &Gr2ExtractionOptions,
    warnings: &mut Vec<String>,
) -> Vec<std::path::PathBuf> {
    let mut png_paths = Vec::new();
    for dds_path in textures {
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
                    warnings.push(format!(
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
    png_paths
}
