//! Mod structure validation

use std::path::Path;

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
    let meta_paths = [
        mod_path.join("Mods"),
        mod_path.to_path_buf(),
    ];

    let mut found_meta = false;
    for base in meta_paths {
        if base.exists() && base.is_dir()
            && let Ok(entries) = std::fs::read_dir(&base) {
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
        warnings.push("No standard mod directories found (Mods/, Public/, Localization/)".to_string());
        valid = false;
    }

    ModValidationResult {
        valid,
        structure,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_empty_dir() {
        let temp = TempDir::new().unwrap();
        let result = validate_mod_structure(temp.path());
        assert!(!result.valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_validate_valid_mod() {
        let temp = TempDir::new().unwrap();

        // Create Mods/TestMod/meta.lsx
        let mod_dir = temp.path().join("Mods").join("TestMod");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("meta.lsx"), "<xml>test</xml>").unwrap();

        let result = validate_mod_structure(temp.path());
        assert!(result.valid);
        assert!(result.structure.iter().any(|s| s.contains("Mods/")));
        assert!(result.structure.iter().any(|s| s.contains("meta.lsx")));
    }
}
