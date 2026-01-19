//! Batch virtual texture extraction operations
//!
//! This module provides high-level functions for extracting GTS/GTP virtual textures,
//! including parallel batch extraction.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;

use super::{GtsFile, VirtualTextureExtractor};
use super::utils::find_gts_path;
use crate::error::Error;

/// Result of extracting a GTS/GTP file
#[derive(Debug, Clone)]
pub struct GtsExtractResult {
    /// Number of textures extracted
    pub texture_count: usize,
    /// Number of GTP files processed
    pub gtp_count: usize,
}

/// Result of batch extraction
#[derive(Debug, Clone)]
pub struct BatchExtractResult {
    /// Number of successful extractions
    pub success_count: usize,
    /// Number of failed extractions
    pub error_count: usize,
    /// Total textures extracted
    pub texture_count: usize,
    /// Messages for each file processed
    pub results: Vec<String>,
}

/// Extract textures from a GTS file (or single GTP with its GTS)
///
/// Handles both .gts (full extraction) and .gtp (single file) inputs.
/// For GTS files, extracts all referenced GTP page files.
/// For GTP files, extracts only that single page file.
///
/// # Arguments
/// * `input_path` - Path to either a .gts or .gtp file
/// * `output_dir` - Optional output directory. If None, uses the input file's parent directory
/// * `progress` - Callback for progress updates (current, total, description)
///
/// # Returns
/// Information about the extraction, or an error.
///
/// # Errors
/// Returns an error if the GTS/GTP file cannot be read or extraction fails.
pub fn extract_gts_file<P, F>(
    input_path: P,
    output_dir: Option<&Path>,
    progress: F,
) -> Result<GtsExtractResult, Error>
where
    P: AsRef<Path>,
    F: Fn(usize, usize, &str),
{
    let input_path = input_path.as_ref();
    let input_path_str = input_path.to_string_lossy();
    let input_ext = input_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let is_single_gtp = input_ext == "gtp";

    // Find the GTS file (handles both .gts and .gtp inputs)
    let gts_path_str = find_gts_path(&input_path_str)?;
    let gts_path = PathBuf::from(&gts_path_str);

    // Determine output directory
    let output_path = match output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input_path
            .parent().map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf),
    };

    // Create subdirectory based on input filename
    let input_stem = input_path
        .file_stem().map_or_else(|| "textures".to_string(), |n| n.to_string_lossy().to_string());
    let texture_output_dir = output_path.join(&input_stem);

    std::fs::create_dir_all(&texture_output_dir)
        .map_err(|e| Error::Io(std::io::Error::other(
            format!("Failed to create output directory: {e}")
        )))?;

    if is_single_gtp {
        // Single GTP mode: extract just this GTP file
        progress(0, 1, "Extracting GTP...");

        VirtualTextureExtractor::extract_with_gts(
            input_path,
            &gts_path,
            &texture_output_dir,
        )?;

        progress(1, 1, "Complete");

        // Count output files
        let count = std::fs::read_dir(&texture_output_dir)
            .map(|entries| entries.filter_map(std::result::Result::ok).count())
            .unwrap_or(0);

        Ok(GtsExtractResult {
            texture_count: count,
            gtp_count: 1,
        })
    } else {
        // Full GTS mode: extract all GTPs referenced by this GTS
        let gts = GtsFile::open(&gts_path)?;

        let gts_dir = gts_path.parent().unwrap_or(Path::new("."));
        let total_page_files = gts.page_files.len();

        if total_page_files == 0 {
            return Err(Error::InvalidFormat("No page files found in GTS".to_string()));
        }

        let mut extracted_count = 0;
        let mut failed_count = 0;

        for (i, page_file) in gts.page_files.iter().enumerate() {
            let gtp_path = gts_dir.join(&page_file.filename);

            progress(i, total_page_files, &format!("Extracting {}...", page_file.filename));

            if gtp_path.exists() {
                // Create a subdirectory for this GTP's output
                let gtp_stem = Path::new(&page_file.filename)
                    .file_stem().map_or_else(|| format!("gtp_{i}"), |n| n.to_string_lossy().to_string());
                let gtp_output_dir = texture_output_dir.join(&gtp_stem);

                match VirtualTextureExtractor::extract_with_gts(
                    &gtp_path,
                    &gts_path,
                    &gtp_output_dir,
                ) {
                    Ok(()) => extracted_count += 1,
                    Err(e) => {
                        tracing::warn!("Failed to extract {}: {}", page_file.filename, e);
                        failed_count += 1;
                    }
                }
            } else {
                tracing::warn!("GTP file not found: {}", gtp_path.display());
                failed_count += 1;
            }
        }

        if extracted_count == 0 && total_page_files > 0 {
            return Err(Error::InvalidFormat(format!(
                "No GTP files could be extracted (0/{total_page_files} succeeded, {failed_count} failed)"
            )));
        }

        Ok(GtsExtractResult {
            texture_count: extracted_count,
            gtp_count: total_page_files,
        })
    }
}

/// Batch extract multiple GTS files in parallel
///
/// # Arguments
/// * `gts_files` - List of GTS file paths to extract
/// * `output_dir` - Optional output directory. If None, uses each file's parent directory
/// * `progress` - Callback for progress updates (current, total, description)
///
/// # Returns
/// Summary of the batch extraction.
pub fn extract_batch<F>(
    gts_files: &[PathBuf],
    output_dir: Option<&Path>,
    progress: F,
) -> BatchExtractResult
where
    F: Fn(usize, usize, &str) + Send + Sync,
{
    let total = gts_files.len();
    let success_counter = AtomicUsize::new(0);
    let error_counter = AtomicUsize::new(0);
    let texture_counter = AtomicUsize::new(0);
    let processed = AtomicUsize::new(0);

    // Parallel GTS extraction
    let results: Vec<String> = gts_files
        .par_iter()
        .map(|gts_path| {
            let gts_name = gts_path
                .file_name().map_or_else(|| "unknown".to_string(), |n| n.to_string_lossy().to_string());

            // Update progress (atomic)
            let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
            progress(current, total, &gts_name);

            // Create a no-op progress callback for individual extractions
            let noop_progress = |_: usize, _: usize, _: &str| {};

            match extract_gts_file(gts_path, output_dir, noop_progress) {
                Ok(result) => {
                    success_counter.fetch_add(1, Ordering::SeqCst);
                    texture_counter.fetch_add(result.texture_count, Ordering::SeqCst);
                    format!("Extracted {} textures from {}", result.texture_count, gts_name)
                }
                Err(e) => {
                    error_counter.fetch_add(1, Ordering::SeqCst);
                    format!("Failed {gts_name}: {e}")
                }
            }
        })
        .collect();

    BatchExtractResult {
        success_count: success_counter.load(Ordering::SeqCst),
        error_count: error_counter.load(Ordering::SeqCst),
        texture_count: texture_counter.load(Ordering::SeqCst),
        results,
    }
}
