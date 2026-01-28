//! Mod structure validation

use std::path::Path;

use crate::pak::PakOperations;

use super::types::{ModPhase, ModProgress, ModProgressCallback};

/// Result of mod structure validation
#[derive(Clone, Debug)]
pub struct ModValidationResult {
    /// Whether the mod structure is valid
    pub valid: bool,
    /// Found structure elements (e.g., "+ Mods/", "+ Public/")
    pub structure: Vec<String>,
    /// Warning messages about potential issues
    pub warnings: Vec<String>,
}

/// Validate mod directory structure
///
/// Checks for:
/// - Standard mod directories (Mods, Public, Localization)
/// - Presence of meta.lsx file
///
/// # Arguments
/// * `mod_path` - Path to the mod directory to validate
///
/// # Returns
/// `ModValidationResult` with validation status and details
#[must_use]
pub fn validate_mod_structure(mod_path: &Path) -> ModValidationResult {
    validate_mod_structure_with_progress(mod_path, &|_| {})
}

/// Validate mod directory structure with progress callback
///
/// Checks for:
/// - Standard mod directories (Mods, Public, Localization)
/// - Presence of meta.lsx file
///
/// # Arguments
/// * `mod_path` - Path to the mod directory to validate
/// * `progress` - Progress callback
///
/// # Returns
/// `ModValidationResult` with validation status and details
#[must_use]
pub fn validate_mod_structure_with_progress(
    mod_path: &Path,
    progress: ModProgressCallback,
) -> ModValidationResult {
    progress(&ModProgress::with_file(
        ModPhase::Validating,
        0,
        1,
        "Checking directory structure",
    ));

    let mut valid = true;
    let mut structure = Vec::new();
    let mut warnings = Vec::new();

    // Check for common mod directories
    let expected_dirs = ["Mods", "Public", "Localization"];
    for dir_name in expected_dirs {
        let dir_path = mod_path.join(dir_name);
        if dir_path.exists() {
            structure.push(format!("+ {dir_name}/"));
        }
    }

    // Check for meta.lsx
    let meta_paths = [mod_path.join("Mods"), mod_path.to_path_buf()];

    let mut found_meta = false;
    for base in meta_paths {
        if base.exists()
            && base.is_dir()
            && let Ok(entries) = std::fs::read_dir(&base)
        {
            for entry in entries.flatten() {
                let meta_path = entry.path().join("meta.lsx");
                if meta_path.exists() {
                    found_meta = true;
                    structure.push(format!(
                        "+ {}/meta.lsx",
                        entry.file_name().to_string_lossy()
                    ));
                }
            }
        }
    }

    if !found_meta {
        warnings.push("No meta.lsx found - mod may not load properly".to_string());
        valid = false;
    }

    if structure.is_empty() {
        warnings
            .push("No standard mod directories found (Mods/, Public/, Localization/)".to_string());
        valid = false;
    }

    progress(&ModProgress::new(ModPhase::Complete, 1, 1));

    ModValidationResult {
        valid,
        structure,
        warnings,
    }
}

/// Validate mod structure within a PAK file
///
/// Checks for:
/// - Standard mod directories (Mods, Public, Localization)
/// - Presence of meta.lsx file
///
/// # Arguments
/// * `pak_path` - Path to the PAK file to validate
///
/// # Returns
/// `ModValidationResult` with validation status and details
///
/// # Errors
/// Returns an error if the PAK file cannot be read
pub fn validate_pak_mod_structure(pak_path: &Path) -> crate::error::Result<ModValidationResult> {
    validate_pak_mod_structure_with_progress(pak_path, &|_| {})
}

/// Validate mod structure within a PAK file with progress callback
///
/// Checks for:
/// - Standard mod directories (Mods, Public, Localization)
/// - Presence of meta.lsx file
///
/// # Arguments
/// * `pak_path` - Path to the PAK file to validate
/// * `progress` - Progress callback
///
/// # Returns
/// `ModValidationResult` with validation status and details
///
/// # Errors
/// Returns an error if the PAK file cannot be read
pub fn validate_pak_mod_structure_with_progress(
    pak_path: &Path,
    progress: ModProgressCallback,
) -> crate::error::Result<ModValidationResult> {
    progress(&ModProgress::with_file(
        ModPhase::Validating,
        0,
        1,
        "Reading PAK file list",
    ));

    let files = PakOperations::list(pak_path)?;

    let mut valid = true;
    let mut structure = Vec::new();
    let mut warnings = Vec::new();

    // Check for common mod directories
    let expected_dirs = ["Mods/", "Public/", "Localization/"];
    for dir_name in expected_dirs {
        if files.iter().any(|f| f.starts_with(dir_name)) {
            structure.push(format!("+ {dir_name}"));
        }
    }

    // Check for meta.lsx
    let meta_files: Vec<_> = files.iter().filter(|f| f.ends_with("meta.lsx")).collect();

    if meta_files.is_empty() {
        warnings.push("No meta.lsx found - mod may not load properly".to_string());
        valid = false;
    } else {
        for meta in meta_files {
            structure.push(format!("+ {meta}"));
        }
    }

    if structure.is_empty() {
        warnings
            .push("No standard mod directories found (Mods/, Public/, Localization/)".to_string());
        valid = false;
    }

    progress(&ModProgress::new(ModPhase::Complete, 1, 1));

    Ok(ModValidationResult {
        valid,
        structure,
        warnings,
    })
}
