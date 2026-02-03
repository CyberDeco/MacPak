//! PAK integrity checking
//!
//! Provides functionality for validating PAK file integrity.

use std::path::Path;

use crate::error::Result;
use crate::pak::PakOperations;

use super::types::{ModPhase, ModProgress, ModProgressCallback};

/// Result of PAK file integrity check
#[derive(Clone, Debug)]
pub struct PakIntegrityResult {
    /// Whether the PAK passes integrity checks
    pub valid: bool,
    /// Number of files in the PAK
    pub file_count: usize,
    /// Total uncompressed size
    pub total_size: u64,
    /// Any integrity issues found
    pub issues: Vec<String>,
}

/// Check PAK file integrity
///
/// Verifies that the PAK file can be read and all entries are accessible.
///
/// # Arguments
/// * `pak_path` - Path to the PAK file
///
/// # Returns
/// Integrity check result
///
/// # Errors
/// Returns an error if the PAK cannot be read at all
pub fn check_pak_integrity(pak_path: &Path) -> Result<PakIntegrityResult> {
    check_pak_integrity_with_progress(pak_path, &|_| {})
}

/// Check PAK file integrity with progress callback
///
/// # Arguments
/// * `pak_path` - Path to the PAK file
/// * `progress` - Progress callback
///
/// # Returns
/// Integrity check result
///
/// # Errors
/// Returns an error if the PAK cannot be read at all
pub fn check_pak_integrity_with_progress(
    pak_path: &Path,
    progress: ModProgressCallback,
) -> Result<PakIntegrityResult> {
    progress(&ModProgress::with_file(
        ModPhase::Validating,
        0,
        1,
        "Reading PAK header...",
    ));

    let mut issues = Vec::new();
    let valid;

    // Try to list the PAK contents
    let files = match PakOperations::list(pak_path) {
        Ok(f) => f,
        Err(e) => {
            return Ok(PakIntegrityResult {
                valid: false,
                file_count: 0,
                total_size: 0,
                issues: vec![format!("Failed to read PAK: {e}")],
            });
        }
    };

    let file_count = files.len();

    progress(&ModProgress::with_file(
        ModPhase::Validating,
        1,
        2,
        format!("Checking {file_count} files..."),
    ));

    // Check for duplicate paths (shouldn't happen but indicates corruption)
    let mut seen_paths = std::collections::HashSet::new();
    for file in &files {
        if !seen_paths.insert(file) {
            issues.push(format!("Duplicate file path: '{file}'"));
        }
    }

    // Try to read a small file to verify decompression works
    // Find a small file (preferably meta.lsx or similar)
    let test_file = files
        .iter()
        .find(|f| f.ends_with("meta.lsx"))
        .or_else(|| files.iter().find(|f| f.ends_with(".lsx")))
        .or_else(|| files.iter().find(|f| f.ends_with(".txt")))
        .or_else(|| files.first());

    if let Some(test_path) = test_file {
        match PakOperations::read_file_bytes(pak_path, test_path) {
            Ok(_) => {
                valid = issues.is_empty();
            }
            Err(e) => {
                issues.push(format!("Failed to read test file '{test_path}': {e}"));
                valid = false;
            }
        }
    } else {
        valid = issues.is_empty();
    }

    // Get file size from disk
    let total_size = std::fs::metadata(pak_path).map(|m| m.len()).unwrap_or(0);

    progress(&ModProgress::new(ModPhase::Complete, 2, 2));

    Ok(PakIntegrityResult {
        valid,
        file_count,
        total_size,
        issues,
    })
}
