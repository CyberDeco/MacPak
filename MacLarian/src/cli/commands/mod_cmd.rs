//! CLI commands for mod utilities

use std::path::Path;

/// Validate mod directory structure
pub fn validate(source: &Path) -> anyhow::Result<()> {
    let result = crate::mods::validate_mod_structure(source);

    // Print structure elements
    if !result.structure.is_empty() {
        println!("Structure:");
        for item in &result.structure {
            println!("  {item}");
        }
    }

    // Print warnings
    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  - {warning}");
        }
    }

    // Print result
    if result.valid {
        println!("\nValidation: PASSED");
        Ok(())
    } else {
        println!("\nValidation: FAILED");
        std::process::exit(1);
    }
}

/// Generate info.json for BaldursModManager
pub fn info_json(pak: &Path, extracted: &Path, output: Option<&Path>) -> anyhow::Result<()> {
    let pak_str = pak.to_string_lossy();
    let extracted_str = extracted.to_string_lossy();

    let result = crate::mods::generate_info_json(&extracted_str, &pak_str);

    if !result.success {
        eprintln!("Error: {}", result.message);
        std::process::exit(1);
    }

    let json = result.content.expect("success should have content");

    match output {
        Some(path) => {
            std::fs::write(path, &json)?;
            println!("Wrote info.json to {}", path.display());
        }
        None => {
            println!("{json}");
        }
    }

    Ok(())
}
