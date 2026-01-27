//! Batch PAK operations
//!
//! This module provides functions for batch PAK extraction and creation,
//! including parallel processing and file discovery.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;
use walkdir::WalkDir;

use super::PakOperations;
use super::lspk::{PakProgress, PakPhase};

/// Result of a batch PAK operation
#[derive(Debug, Clone)]
pub struct BatchPakResult {
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub fail_count: usize,
    /// Messages for each file processed
    pub results: Vec<String>,
}

/// Find all .pak files in a directory recursively
///
/// # Arguments
/// * `dir` - Directory to search for PAK files
///
/// # Returns
/// A sorted list of paths to .pak files found in the directory tree.
pub fn find_pak_files<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut pak_files: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"))
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    pak_files.sort();
    pak_files
}

/// Find all packable folders (immediate subdirectories that contain files)
///
/// Finds immediate subdirectories of the given directory that contain at least
/// one file (recursively), making them suitable for packing into PAK archives.
/// Each subdirectory becomes one PAK file containing all its contents.
///
/// # Arguments
/// * `dir` - Root directory to search
///
/// # Returns
/// A sorted list of immediate subdirectory paths that contain files.
pub fn find_packable_folders<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut folders: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .min_depth(1) // Skip the root directory itself
        .max_depth(1) // Only immediate subdirectories
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let path = e.path();
            if !path.is_dir() {
                return false;
            }
            // Check if this directory contains any files (recursively)
            contains_files_recursive(path)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    folders.sort();
    folders
}

/// Check if a directory contains any files (recursively)
fn contains_files_recursive(dir: &Path) -> bool {
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .any(|e| e.path().is_file())
}

/// Batch extract PAK files in parallel
///
/// Extracts multiple PAK files to a destination directory, preserving the
/// source directory structure. Each PAK is extracted into a subdirectory
/// named after the PAK file (without extension).
///
/// # Arguments
/// * `pak_files` - List of PAK files to extract
/// * `source_base` - Base directory of the source (for calculating relative paths)
/// * `dest_base` - Destination directory for extracted files
/// * `progress` - Callback for progress updates
///
/// # Returns
/// Summary of the batch extraction operation.
pub fn batch_extract<F>(
    pak_files: &[PathBuf],
    source_base: &Path,
    dest_base: &Path,
    progress: F,
) -> BatchPakResult
where
    F: Fn(&PakProgress) + Send + Sync,
{
    let success_counter = AtomicUsize::new(0);
    let fail_counter = AtomicUsize::new(0);
    let processed = AtomicUsize::new(0);
    let total = pak_files.len();

    // Parallel PAK extraction
    let results: Vec<String> = pak_files
        .par_iter()
        .map(|pak_path| {
            // Calculate relative path for display and output structure
            let relative_path = pak_path
                .strip_prefix(source_base)
                .unwrap_or(pak_path.as_path());
            let display_path = relative_path.to_string_lossy();

            // Update progress (atomic)
            let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
            progress(&PakProgress::with_file(
                PakPhase::DecompressingFiles,
                current,
                total,
                display_path.to_string(),
            ));

            // Preserve directory structure: create subfolder matching relative path
            let relative_parent = relative_path.parent().unwrap_or(Path::new(""));
            let pak_stem = pak_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let pak_dest = dest_base
                .join(relative_parent)
                .join(&pak_stem);

            if let Err(e) = std::fs::create_dir_all(&pak_dest) {
                fail_counter.fetch_add(1, Ordering::SeqCst);
                return format!("Failed to create folder for {display_path}: {e}");
            }

            let pak_str = pak_path.to_string_lossy().to_string();
            let dest_str = pak_dest.to_string_lossy().to_string();

            match PakOperations::extract(&pak_str, &dest_str) {
                Ok(()) => {
                    success_counter.fetch_add(1, Ordering::SeqCst);
                    format!("Extracted: {display_path}")
                }
                Err(e) => {
                    fail_counter.fetch_add(1, Ordering::SeqCst);
                    format!("Failed {display_path}: {e}")
                }
            }
        })
        .collect();

    BatchPakResult {
        success_count: success_counter.load(Ordering::SeqCst),
        fail_count: fail_counter.load(Ordering::SeqCst),
        results,
    }
}

/// Batch create PAK files in parallel
///
/// Creates PAK files from multiple folders, preserving the source directory
/// structure in the output. Each folder is packed into a PAK file named
/// after the folder.
///
/// # Arguments
/// * `folders` - List of folders to pack
/// * `source_base` - Base directory of the source (for calculating relative paths)
/// * `dest_base` - Destination directory for PAK files
/// * `progress` - Callback for progress updates
///
/// # Returns
/// Summary of the batch creation operation.
pub fn batch_create<F>(
    folders: &[PathBuf],
    source_base: &Path,
    dest_base: &Path,
    progress: F,
) -> BatchPakResult
where
    F: Fn(&PakProgress) + Send + Sync,
{
    let success_counter = AtomicUsize::new(0);
    let fail_counter = AtomicUsize::new(0);
    let processed = AtomicUsize::new(0);
    let total = folders.len();

    // Parallel PAK creation
    let results: Vec<String> = folders
        .par_iter()
        .map(|folder_path| {
            let folder_name = folder_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Calculate relative path for display and output structure
            let relative_path = folder_path
                .strip_prefix(source_base)
                .unwrap_or(folder_path.as_path());
            let display_path = relative_path.to_string_lossy();

            // Update progress (atomic)
            let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
            progress(&PakProgress::with_file(
                PakPhase::CompressingFiles,
                current,
                total,
                display_path.to_string(),
            ));

            // Preserve directory structure: create PAK in matching relative path
            let relative_parent = relative_path.parent().unwrap_or(Path::new(""));
            let pak_dest_dir = dest_base.join(relative_parent);

            // Create parent directories if needed (idempotent)
            if let Err(e) = std::fs::create_dir_all(&pak_dest_dir) {
                fail_counter.fetch_add(1, Ordering::SeqCst);
                return format!("Failed to create dir for {display_path}: {e}");
            }

            let pak_path = pak_dest_dir.join(format!("{folder_name}.pak"));
            let folder_str = folder_path.to_string_lossy().to_string();
            let pak_str = pak_path.to_string_lossy().to_string();

            match PakOperations::create(&folder_str, &pak_str) {
                Ok(()) => {
                    success_counter.fetch_add(1, Ordering::SeqCst);
                    format!("Created: {display_path}.pak")
                }
                Err(e) => {
                    fail_counter.fetch_add(1, Ordering::SeqCst);
                    format!("Failed {display_path}: {e}")
                }
            }
        })
        .collect();

    BatchPakResult {
        success_count: success_counter.load(Ordering::SeqCst),
        fail_count: fail_counter.load(Ordering::SeqCst),
        results,
    }
}

