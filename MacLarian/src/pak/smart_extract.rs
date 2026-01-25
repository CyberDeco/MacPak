//! Smart PAK extraction with automatic GR2 processing
//!
//! This module provides enhanced extraction operations that can automatically
//! process GR2 files after extraction:
//! - Convert GR2 to GLB format
//! - Extract associated DDS textures
//! - Extract and convert virtual textures
//!
//! # Usage
//!
//! ```no_run
//! use maclarian::pak::extract_files_smart;
//! use maclarian::gr2_extraction::Gr2ExtractionOptions;
//!
//! // Extract with full GR2 processing
//! let result = extract_files_smart(
//!     "path/to/source.pak",
//!     "output/directory",
//!     &["Models/Characters/Human/HUM_M_ARM_Leather_A_Body.GR2"],
//!     Gr2ExtractionOptions::bundle(),
//!     &|current, total, name| println!("{current}/{total}: {name}"),
//! ).unwrap();
//!
//! println!("Extracted {} files, processed {} GR2s", result.files_extracted, result.gr2s_processed);
//! ```

#![allow(clippy::needless_pass_by_value, clippy::collapsible_if)]

use rayon::prelude::*;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::gr2_extraction::{
    Gr2ExtractionOptions,
    Gr2ExtractionResult,
    process_extracted_gr2,
};
use super::pak_tools::{PakOperations, ProgressCallback};

/// Result of a smart extraction operation
#[derive(Debug, Clone)]
pub struct SmartExtractionResult {
    /// Number of files extracted from the PAK
    pub files_extracted: usize,
    /// Number of GR2 files processed
    pub gr2s_processed: usize,
    /// Number of GLB files created
    pub glb_files_created: usize,
    /// Number of texture files extracted
    pub textures_extracted: usize,
    /// Paths to extracted GR2 folders (each GR2 gets its own folder)
    pub gr2_folders: Vec<PathBuf>,
    /// Warnings/errors encountered during processing
    pub warnings: Vec<String>,
}

impl SmartExtractionResult {
    /// Create a new empty result
    fn new() -> Self {
        Self {
            files_extracted: 0,
            gr2s_processed: 0,
            glb_files_created: 0,
            textures_extracted: 0,
            gr2_folders: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Extract specific files from a PAK with optional GR2 processing.
///
/// When GR2 processing options are enabled, this function will:
/// 1. Extract all requested files from the PAK
/// 2. For each GR2 file, create a dedicated subfolder
/// 3. Move the GR2 into its subfolder
/// 4. Optionally convert to GLB
/// 5. Optionally extract associated textures
///
/// # Output Structure
///
/// When GR2 processing is enabled, each GR2 file gets its own folder:
/// ```text
/// output_dir/
///   HUM_M_ARM_Leather_A_Body/
///     HUM_M_ARM_Leather_A_Body.GR2     # Original (if keep_original)
///     HUM_M_ARM_Leather_A_Body.glb     # Converted model
///     HUM_M_ARM_Leather_A_Body_BC.dds  # Texture (basecolor)
///     HUM_M_ARM_Leather_A_Body_NM.dds  # Texture (normal)
///   HUM_F_ARM_Scale_A_Body/
///     ...
/// ```
///
/// # Arguments
///
/// * `pak_path` - Path to the source PAK file
/// * `output_dir` - Directory where files will be extracted
/// * `file_paths` - List of file paths within the PAK to extract
/// * `options` - GR2 processing options
/// * `progress` - Progress callback (current, total, description)
///
/// # Errors
///
/// Returns an error if PAK extraction fails. Individual GR2 processing
/// errors are collected in `SmartExtractionResult::warnings` and do not
/// cause the entire operation to fail.
pub fn extract_files_smart<P: AsRef<Path>, S: AsRef<str>>(
    pak_path: P,
    output_dir: P,
    file_paths: &[S],
    options: Gr2ExtractionOptions,
    progress: ProgressCallback,
) -> Result<SmartExtractionResult> {
    let pak_path = pak_path.as_ref();
    let output_dir = output_dir.as_ref();
    let mut result = SmartExtractionResult::new();

    if file_paths.is_empty() {
        return Ok(result);
    }

    // Phase 1: Extract all files normally
    progress(0, file_paths.len(), "Extracting files from PAK...");
    PakOperations::extract_files_with_progress(pak_path, output_dir, file_paths, progress)?;
    result.files_extracted = file_paths.len();

    // If no GR2 processing is enabled, we're done
    if !options.has_gr2_processing() {
        return Ok(result);
    }

    // Phase 2: Find extracted GR2 files
    let gr2_paths: Vec<PathBuf> = file_paths
        .iter()
        .filter(|p| {
            let path = p.as_ref().to_lowercase();
            path.ends_with(".gr2")
        })
        .map(|p| output_dir.join(p.as_ref()))
        .filter(|p| p.exists())
        .collect();

    if gr2_paths.is_empty() {
        return Ok(result);
    }

    // Phase 3: Process GR2 files
    progress(0, gr2_paths.len(), "Processing GR2 files...");

    // Process GR2 files in parallel
    let processing_results: Vec<(PathBuf, std::result::Result<Gr2ExtractionResult, String>)> =
        gr2_paths
            .par_iter()
            .map(|gr2_path| {
                let folder_result = process_single_gr2(
                    gr2_path,
                    output_dir,
                    &options,
                );
                (gr2_path.clone(), folder_result)
            })
            .collect();

    // Collect results
    for (gr2_path, process_result) in processing_results {
        match process_result {
            Ok(proc_result) => {
                result.gr2s_processed += 1;
                if proc_result.glb_path.is_some() {
                    result.glb_files_created += 1;
                }
                result.textures_extracted += proc_result.texture_paths.len();

                // Add the folder to our results
                if let Some(folder) = gr2_path.parent() {
                    if !result.gr2_folders.contains(&folder.to_path_buf()) {
                        result.gr2_folders.push(folder.to_path_buf());
                    }
                }

                // Collect warnings
                result.warnings.extend(proc_result.warnings);
            }
            Err(e) => {
                let path_display = gr2_path.display();
                result.warnings.push(format!("Failed to process {path_display}: {e}"));
            }
        }
    }

    Ok(result)
}

/// Process a single GR2 file: move to subfolder, convert, extract textures
fn process_single_gr2(
    gr2_path: &Path,
    output_base: &Path,
    options: &Gr2ExtractionOptions,
) -> std::result::Result<Gr2ExtractionResult, String> {
    // Get the GR2 filename without extension for the folder name
    let gr2_filename = gr2_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid GR2 filename".to_string())?;

    let folder_name = gr2_filename.trim_end_matches(".GR2").trim_end_matches(".gr2");

    // Create a dedicated subfolder for this GR2
    let gr2_folder = output_base.join(folder_name);
    std::fs::create_dir_all(&gr2_folder)
        .map_err(|e| format!("Failed to create folder: {e}"))?;

    // Move the GR2 file into its subfolder
    let new_gr2_path = gr2_folder.join(gr2_filename);
    if gr2_path != new_gr2_path {
        std::fs::rename(gr2_path, &new_gr2_path)
            .map_err(|e| format!("Failed to move GR2: {e}"))?;

        // Clean up empty parent directories
        cleanup_empty_parent_dirs(gr2_path, output_base);
    }

    // Process the GR2 file
    let result = process_extracted_gr2(&new_gr2_path, options)
        .map_err(|e| e.to_string())?;

    // Optionally delete the original GR2 after conversion
    if !options.keep_original_gr2 && result.glb_path.is_some() {
        let _ = std::fs::remove_file(&new_gr2_path);
    }

    Ok(result)
}

/// Clean up empty directories between a file path and the base directory
fn cleanup_empty_parent_dirs(file_path: &Path, base_dir: &Path) {
    let mut current = file_path.parent();
    while let Some(dir) = current {
        if dir == base_dir || dir.as_os_str().is_empty() {
            break;
        }
        // Try to remove the directory (only succeeds if empty)
        if std::fs::remove_dir(dir).is_err() {
            break; // Directory not empty or other error
        }
        current = dir.parent();
    }
}

/// Extract all files from a PAK with optional GR2 processing.
///
/// Similar to `extract_files_smart` but extracts the entire PAK contents.
///
/// # Errors
///
/// Returns an error if PAK extraction fails.
pub fn extract_pak_smart<P: AsRef<Path>>(
    pak_path: P,
    output_dir: P,
    options: Gr2ExtractionOptions,
    progress: ProgressCallback,
) -> Result<SmartExtractionResult> {
    let pak_path = pak_path.as_ref();
    let output_dir = output_dir.as_ref();

    // List all files in the PAK
    let all_files = PakOperations::list(pak_path)?;

    // Extract with smart processing
    extract_files_smart(pak_path, output_dir, &all_files, options, progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_extraction_result_new() {
        let result = SmartExtractionResult::new();
        assert_eq!(result.files_extracted, 0);
        assert_eq!(result.gr2s_processed, 0);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_gr2_extraction_options_bundle() {
        let opts = Gr2ExtractionOptions::bundle();
        assert!(opts.has_gr2_processing());
        assert!(opts.convert_to_glb);
        assert!(opts.extract_textures);
        assert!(opts.extract_virtual_textures);
        assert!(opts.keep_original_gr2);
    }

    #[test]
    fn test_gr2_extraction_options_default() {
        let opts = Gr2ExtractionOptions::new();
        assert!(!opts.has_gr2_processing());
    }
}
