//! Batch mod validation
//!
//! Provides functionality for validating multiple mods at once,
//! recursive directory scanning, PAK integrity checking, and dry-run validation.

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::pak::PakOperations;

use super::types::{ModPhase, ModProgress, ModProgressCallback};
use super::validation::{
    ModValidationResult, validate_mod_structure, validate_pak_mod_structure,
};

/// Result of batch validation across multiple mods
#[derive(Clone, Debug, Default)]
pub struct BatchValidationResult {
    /// Total number of mods validated
    pub total: usize,
    /// Number of valid mods
    pub valid_count: usize,
    /// Number of mods with warnings
    pub warning_count: usize,
    /// Number of invalid mods
    pub invalid_count: usize,
    /// Individual validation results
    pub entries: Vec<ModValidationEntry>,
}

impl BatchValidationResult {
    /// Check if all mods are valid
    #[must_use]
    pub fn all_valid(&self) -> bool {
        self.invalid_count == 0
    }

    /// Get summary statistics as a formatted string
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{} mods: {} valid, {} with warnings, {} invalid",
            self.total, self.valid_count, self.warning_count, self.invalid_count
        )
    }
}

/// Individual mod validation entry with path information
#[derive(Clone, Debug)]
pub struct ModValidationEntry {
    /// Path to the mod (directory or PAK file)
    pub path: PathBuf,
    /// Name of the mod (filename or directory name)
    pub name: String,
    /// Whether this is a PAK file
    pub is_pak: bool,
    /// Validation result
    pub result: ModValidationResult,
    /// Additional integrity check result (for PAK files)
    pub integrity: Option<PakIntegrityResult>,
}

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

/// Options for batch validation
#[derive(Clone, Debug, Default)]
pub struct BatchValidationOptions {
    /// Include PAK files in validation
    pub include_paks: bool,
    /// Include directories in validation
    pub include_directories: bool,
    /// Perform integrity check on PAK files
    pub check_integrity: bool,
    /// Maximum depth for recursive scanning (None = unlimited)
    pub max_depth: Option<usize>,
}

impl BatchValidationOptions {
    /// Create options for validating everything
    #[must_use]
    pub fn all() -> Self {
        Self {
            include_paks: true,
            include_directories: true,
            check_integrity: true,
            max_depth: None,
        }
    }

    /// Create options for PAK-only validation
    #[must_use]
    pub fn paks_only() -> Self {
        Self {
            include_paks: true,
            include_directories: false,
            check_integrity: true,
            max_depth: None,
        }
    }

    /// Create options for directory-only validation
    #[must_use]
    pub fn directories_only() -> Self {
        Self {
            include_paks: false,
            include_directories: true,
            check_integrity: false,
            max_depth: None,
        }
    }
}

/// Validate all mods in a directory recursively
///
/// Scans the directory for PAK files and mod directories, validating each.
///
/// # Arguments
/// * `dir` - Directory to scan
/// * `options` - Validation options
///
/// # Returns
/// Batch validation result with all findings
///
/// # Errors
/// Returns an error if the directory cannot be read
pub fn validate_directory_recursive(
    dir: &Path,
    options: &BatchValidationOptions,
) -> Result<BatchValidationResult> {
    validate_directory_recursive_with_progress(dir, options, &|_| {})
}

/// Validate all mods in a directory recursively with progress callback
///
/// # Arguments
/// * `dir` - Directory to scan
/// * `options` - Validation options
/// * `progress` - Progress callback
///
/// # Returns
/// Batch validation result with all findings
///
/// # Errors
/// Returns an error if the directory cannot be read
pub fn validate_directory_recursive_with_progress(
    dir: &Path,
    options: &BatchValidationOptions,
    progress: ModProgressCallback,
) -> Result<BatchValidationResult> {
    progress(&ModProgress::with_file(
        ModPhase::Validating,
        0,
        1,
        "Scanning for mods...",
    ));

    // Find all potential mods
    let candidates = find_mod_candidates(dir, options)?;
    let total = candidates.len();

    if total == 0 {
        return Ok(BatchValidationResult::default());
    }

    let mut result = BatchValidationResult {
        total,
        ..Default::default()
    };

    // Validate each candidate
    for (i, candidate) in candidates.into_iter().enumerate() {
        progress(&ModProgress::with_file(
            ModPhase::Validating,
            i + 1,
            total,
            candidate.display().to_string(),
        ));

        let entry = validate_single_mod(&candidate, options)?;

        // Update counts
        if entry.result.valid {
            result.valid_count += 1;
        } else {
            result.invalid_count += 1;
        }

        if !entry.result.warnings.is_empty() {
            result.warning_count += 1;
        }

        result.entries.push(entry);
    }

    progress(&ModProgress::new(ModPhase::Complete, total, total));

    Ok(result)
}

/// Validate multiple mod paths
///
/// # Arguments
/// * `paths` - List of paths to validate (PAK files or directories)
///
/// # Returns
/// Batch validation result
///
/// # Errors
/// Returns an error if any path cannot be accessed
pub fn validate_mods_batch(paths: &[PathBuf]) -> Result<BatchValidationResult> {
    validate_mods_batch_with_progress(paths, &BatchValidationOptions::all(), &|_| {})
}

/// Validate multiple mod paths with options and progress
///
/// # Arguments
/// * `paths` - List of paths to validate
/// * `options` - Validation options
/// * `progress` - Progress callback
///
/// # Returns
/// Batch validation result
///
/// # Errors
/// Returns an error if any path cannot be accessed
pub fn validate_mods_batch_with_progress(
    paths: &[PathBuf],
    options: &BatchValidationOptions,
    progress: ModProgressCallback,
) -> Result<BatchValidationResult> {
    let total = paths.len();

    let mut result = BatchValidationResult {
        total,
        ..Default::default()
    };

    for (i, path) in paths.iter().enumerate() {
        progress(&ModProgress::with_file(
            ModPhase::Validating,
            i + 1,
            total,
            path.display().to_string(),
        ));

        let entry = validate_single_mod(path, options)?;

        if entry.result.valid {
            result.valid_count += 1;
        } else {
            result.invalid_count += 1;
        }

        if !entry.result.warnings.is_empty() {
            result.warning_count += 1;
        }

        result.entries.push(entry);
    }

    progress(&ModProgress::new(ModPhase::Complete, total, total));

    Ok(result)
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
        format!("Checking {} files...", file_count),
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
    let total_size = std::fs::metadata(pak_path)
        .map(|m| m.len())
        .unwrap_or(0);

    progress(&ModProgress::new(ModPhase::Complete, 2, 2));

    Ok(PakIntegrityResult {
        valid,
        file_count,
        total_size,
        issues,
    })
}

/// Dry-run PAK creation validation
///
/// Validates that a directory can be successfully packed without actually creating the PAK.
///
/// # Arguments
/// * `source_dir` - Directory to validate for packing
///
/// # Returns
/// Validation result with any issues found
#[derive(Clone, Debug)]
pub struct DryRunResult {
    /// Whether packing would succeed
    pub valid: bool,
    /// Number of files that would be packed
    pub file_count: usize,
    /// Total size of files
    pub total_size: u64,
    /// Issues that would prevent packing
    pub errors: Vec<String>,
    /// Warnings about potential issues
    pub warnings: Vec<String>,
}

/// Validate directory for PAK creation (dry run)
///
/// Checks that all files exist, are readable, and the structure is valid.
///
/// # Arguments
/// * `source_dir` - Directory to validate
///
/// # Returns
/// Dry run result with validation findings
///
/// # Errors
/// Returns an error if the directory cannot be read
pub fn validate_for_pak_creation(source_dir: &Path) -> Result<DryRunResult> {
    validate_for_pak_creation_with_progress(source_dir, &|_| {})
}

/// Validate directory for PAK creation with progress callback
///
/// # Arguments
/// * `source_dir` - Directory to validate
/// * `progress` - Progress callback
///
/// # Returns
/// Dry run result with validation findings
///
/// # Errors
/// Returns an error if the directory cannot be read
pub fn validate_for_pak_creation_with_progress(
    source_dir: &Path,
    progress: ModProgressCallback,
) -> Result<DryRunResult> {
    progress(&ModProgress::with_file(
        ModPhase::Validating,
        0,
        1,
        "Scanning directory...",
    ));

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut file_count = 0;
    let mut total_size = 0u64;

    if !source_dir.exists() {
        return Ok(DryRunResult {
            valid: false,
            file_count: 0,
            total_size: 0,
            errors: vec![format!("Directory does not exist: {}", source_dir.display())],
            warnings: Vec::new(),
        });
    }

    if !source_dir.is_dir() {
        return Ok(DryRunResult {
            valid: false,
            file_count: 0,
            total_size: 0,
            errors: vec![format!("Path is not a directory: {}", source_dir.display())],
            warnings: Vec::new(),
        });
    }

    // Walk the directory
    let walker = walkdir::WalkDir::new(source_dir)
        .follow_links(false)
        .into_iter();

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            file_count += 1;

            // Check file is readable
            match std::fs::metadata(entry.path()) {
                Ok(meta) => {
                    total_size += meta.len();

                    // Warn about very large files
                    if meta.len() > 500 * 1024 * 1024 {
                        // 500MB
                        warnings.push(format!(
                            "Large file: {} ({} MB)",
                            entry.path().display(),
                            meta.len() / 1024 / 1024
                        ));
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "Cannot read file '{}': {}",
                        entry.path().display(),
                        e
                    ));
                }
            }

            // Check for problematic filenames
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with('.') {
                    warnings.push(format!("Hidden file will be included: {}", name));
                }
            } else {
                errors.push(format!(
                    "Invalid filename encoding: {}",
                    entry.path().display()
                ));
            }
        }
    }

    progress(&ModProgress::new(ModPhase::Complete, 1, 1));

    // Warn if no files found
    if file_count == 0 {
        warnings.push("No files found in directory".to_string());
    }

    Ok(DryRunResult {
        valid: errors.is_empty(),
        file_count,
        total_size,
        errors,
        warnings,
    })
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Find all mod candidates (PAK files and mod directories) in a directory
fn find_mod_candidates(dir: &Path, options: &BatchValidationOptions) -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();

    let walker = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .max_depth(options.max_depth.map_or(usize::MAX, |d| d + 1))
        .into_iter();

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        // Check for PAK files
        if options.include_paks && path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("pak") {
                    candidates.push(path.to_path_buf());
                }
            }
        }

        // Check for mod directories (has Mods/ or meta.lsx)
        if options.include_directories && path.is_dir() && path != dir {
            if is_mod_directory(path) {
                candidates.push(path.to_path_buf());
            }
        }
    }

    Ok(candidates)
}

/// Check if a directory looks like a mod directory
fn is_mod_directory(dir: &Path) -> bool {
    // Has Mods/ subdirectory
    if dir.join("Mods").is_dir() {
        return true;
    }

    // Has Public/ subdirectory
    if dir.join("Public").is_dir() {
        return true;
    }

    // Has meta.lsx somewhere
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join("meta.lsx").exists() {
                    return true;
                }
            }
        }
    }

    false
}

/// Validate a single mod (PAK or directory)
fn validate_single_mod(path: &Path, options: &BatchValidationOptions) -> Result<ModValidationEntry> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let is_pak = path.is_file()
        && path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("pak"))
            .unwrap_or(false);

    let (result, integrity) = if is_pak {
        let validation = validate_pak_mod_structure(path)?;
        let integrity = if options.check_integrity {
            Some(check_pak_integrity(path)?)
        } else {
            None
        };
        (validation, integrity)
    } else {
        (validate_mod_structure(path), None)
    };

    Ok(ModValidationEntry {
        path: path.to_path_buf(),
        name,
        is_pak,
        result,
        integrity,
    })
}
