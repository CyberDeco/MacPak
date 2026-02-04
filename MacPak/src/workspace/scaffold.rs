//! Project scaffolding from recipe templates
//!
//! Creates the directory structure and generated files for a new mod project.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::workspace::project::ProjectManifest;
use crate::workspace::recipe::{FileKind, Recipe, substitute};

use maclarian::mods::meta_generator::{generate_meta_lsx, parse_version_string};

/// Create the full project on disk from a recipe + manifest.
///
/// 1. Creates all directories from recipe.structure.directories
/// 2. For each `generated` file, calls the appropriate generator
/// 3. Writes macpak.toml manifest to project root
pub fn scaffold_project(
    project_dir: &Path,
    manifest: &ProjectManifest,
    recipe: &Recipe,
) -> Result<(), String> {
    let vars = build_template_vars(manifest);

    // Create project root
    fs::create_dir_all(project_dir)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;

    // Create all directories from recipe
    for dir_template in &recipe.structure.directories {
        let dir_path = substitute(dir_template, &vars);
        fs::create_dir_all(project_dir.join(&dir_path))
            .map_err(|e| format!("Failed to create directory {}: {}", dir_path, e))?;
    }

    // Process generated files
    for file in &recipe.files {
        if file.kind != FileKind::Generated {
            continue;
        }

        let file_path = substitute(&file.path, &vars);
        let full_path = project_dir.join(&file_path);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent dir for {}: {}", file_path, e))?;
        }

        match file.generator.as_deref() {
            Some("meta_lsx") => {
                generate_meta_lsx_file(&full_path, manifest)?;
            }
            Some("localization_xml") => {
                generate_localization_xml(&full_path, manifest)?;
            }
            Some("dye_object_txt") => {
                generate_dye_object_txt(&full_path, manifest)?;
            }
            Some("dye_item_combos_txt") => {
                generate_dye_item_combos_txt(&full_path)?;
            }
            Some("dye_treasure_table_txt") => {
                generate_dye_treasure_table_txt(&full_path, manifest)?;
            }
            Some(other) => {
                return Err(format!("Unknown generator: {}", other));
            }
            None => {
                return Err(format!(
                    "Generated file {} has no generator specified",
                    file_path
                ));
            }
        }
    }

    // Write macpak.toml manifest
    let manifest_toml = toml::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    fs::write(project_dir.join("macpak.toml"), manifest_toml)
        .map_err(|e| format!("Failed to write macpak.toml: {}", e))?;

    Ok(())
}

/// Build the template variable map from a manifest
fn build_template_vars(manifest: &ProjectManifest) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("mod_name".to_string(), manifest.project.folder.clone());
    vars.insert("uuid".to_string(), manifest.project.uuid.clone());
    vars.insert("author".to_string(), manifest.project.author.clone());
    vars.insert("version".to_string(), manifest.project.version.clone());

    // Include recipe-specific variables
    for (key, value) in &manifest.variables {
        vars.insert(key.clone(), value.clone());
    }

    vars
}

/// Generate meta.lsx using the existing MacLarian generator
fn generate_meta_lsx_file(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    let (major, minor, patch, build) =
        parse_version_string(&manifest.project.version).unwrap_or((1, 0, 0, 0));

    let content = generate_meta_lsx(
        &manifest.project.name,
        &manifest.project.folder,
        &manifest.project.author,
        &manifest.project.description,
        &manifest.project.uuid,
        major,
        minor,
        patch,
        build,
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write meta.lsx: {}", e))
}

/// Generate an empty localization XML file
fn generate_localization_xml(dest: &Path, _manifest: &ProjectManifest) -> Result<(), String> {
    let content = r#"<?xml version="1.0" encoding="utf-8"?>
<contentList>
</contentList>
"#;

    fs::write(dest, content).map_err(|e| format!("Failed to write localization XML: {}", e))
}

/// Generate a placeholder Object.txt for a dye mod.
///
/// Contains an example dye entry and pouch entry showing the expected format.
/// The Dye Lab will overwrite this with real data.
fn generate_dye_object_txt(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    let mod_name = &manifest.project.folder;
    let content = format!(
        r#"new entry "ExampleDye"
type "Object"
using "_Dyes"
data "RootTemplate" "00000000-0000-0000-0000-000000000000"

new entry "{mod_name}_DyePouch"
type "Object"
using "OBJ_Pouch"
data "RootTemplate" "00000000-0000-0000-0000-000000000000"
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write Object.txt: {}", e))
}

/// Generate a placeholder ItemCombos.txt for a dye mod.
///
/// Contains an example dye combination entry showing the expected format.
/// The Dye Lab will overwrite this with real data.
fn generate_dye_item_combos_txt(dest: &Path) -> Result<(), String> {
    let content = r#"new ItemCombination "ExampleDye"
data "Type 1" "Object"
data "Object 1" "ExampleDye"
data "Transform 1" "None"
data "Type 2" "Category"
data "Object 2" "DyableArmor"
data "Transform 2" "Dye"
data "DyeColorPresetResource" "00000000-0000-0000-0000-000000000000"

new ItemCombinationResult "ExampleDye_1"
data "ResultAmount 1" "1"
"#;

    fs::write(dest, content).map_err(|e| format!("Failed to write ItemCombos.txt: {}", e))
}

/// Generate a placeholder TreasureTable.txt for a dye mod.
///
/// Contains an example treasure table and pouch contents table.
/// The Dye Lab will overwrite this with real data and vendor entries.
fn generate_dye_treasure_table_txt(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    let mod_name = &manifest.project.folder;
    let content = format!(
        r#"new treasuretable "{mod_name}_Dyes"
new subtable "1,1"
object category "I_ExampleDye",1,0,0,0,0,0,0,0

new treasuretable "{mod_name}_DyePouch_Contents"
CanMerge 1
new subtable "1,1"
object category "I_{mod_name}_DyePouch",1,0,0,0,0,0,0,0
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write TreasureTable.txt: {}", e))
}
