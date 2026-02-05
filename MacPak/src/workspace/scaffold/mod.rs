//! Project scaffolding from recipe templates.
//!
//! Creates the directory structure and generated files for a new mod project.
//! Generator functions are organized by recipe type in submodules.

mod class;
mod dyes;
mod equipment;
mod hair;
mod spell;

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
            // Shared generators
            Some("meta_lsx") => {
                generate_meta_lsx_file(&full_path, manifest)?;
            }
            Some("localization_xml") => {
                generate_localization_xml(&full_path, manifest)?;
            }
            // Dye generators
            Some("dye_object_txt") => {
                dyes::generate_dye_object_txt(&full_path, manifest)?;
            }
            Some("dye_item_combos_txt") => {
                dyes::generate_dye_item_combos_txt(&full_path)?;
            }
            Some("dye_treasure_table_txt") => {
                dyes::generate_dye_treasure_table_txt(&full_path, manifest)?;
            }
            // Spell generators
            Some("spell_data_txt") => {
                spell::generate_spell_data_txt(&full_path, manifest)?;
            }
            // Hair generators
            Some("hair_cc_appearance_lsx") => {
                hair::generate_hair_cc_appearance_lsx(&full_path, manifest)?;
            }
            // Class generators
            Some("class_descriptions_lsx") => {
                class::generate_class_descriptions_lsx(&full_path, manifest)?;
            }
            Some("class_progressions_lsx") => {
                class::generate_class_progressions_lsx(&full_path, manifest)?;
            }
            // Equipment generators
            Some("equipment_armor_txt") => {
                equipment::generate_equipment_armor_txt(&full_path, manifest)?;
            }
            Some("equipment_weapon_txt") => {
                equipment::generate_equipment_weapon_txt(&full_path, manifest)?;
            }
            Some("equipment_treasure_table_txt") => {
                equipment::generate_equipment_treasure_table_txt(&full_path, manifest)?;
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
