//! Import operations and data loading functions

use floem::prelude::*;
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

use super::super::shared::{
    ParsedDyeEntry, load_colors_from_map, parse_localization_xml, parse_lsx_dye_presets,
    parse_meta_lsx, parse_object_txt, parse_root_templates_localization, reset_colors_to_default,
};
use crate::gui::state::DyesState;

// maclarian imports for LSF conversion (via MacPak re-export)
use crate::maclarian::converter::to_lsx;
use crate::maclarian::formats::lsf;

/// Import from an extracted mod folder
/// Automatically discovers _merged.lsf/lsx (colors), Object.txt (metadata), and meta.lsx (mod info)
pub fn import_from_mod_folder(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_display_name: RwSignal<String>,
    imported_mod_name: RwSignal<String>,
    imported_mod_author: RwSignal<String>,
) {
    let dialog = rfd::FileDialog::new().set_title("Select Extracted Mod Folder");

    if let Some(folder_path) = dialog.pick_folder() {
        // Clear previous imports
        state.imported_entries.set(Vec::new());
        state.selected_import_index.set(None);
        state.imported_lsf_entries.set(Vec::new());
        state.selected_lsf_index.set(None);
        state.imported_lsf_path.set(None);
        imported_dye_name.set(String::new());
        imported_display_name.set(String::new());
        imported_mod_name.set(String::new());
        imported_mod_author.set(String::new());

        let folder_name = folder_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("mod")
            .to_string();

        // Search for dye color files
        let mut color_file: Option<std::path::PathBuf> = None; // Primary: _merged in [PAK]_DYE_Colors
        let mut color_files: Vec<std::path::PathBuf> = Vec::new(); // Fallback: individual LSF files
        let mut object_file: Option<std::path::PathBuf> = None;
        let mut root_templates_files: Vec<std::path::PathBuf> = Vec::new();
        let mut localization_file: Option<std::path::PathBuf> = None;
        let mut meta_file: Option<std::path::PathBuf> = None;

        for entry in WalkDir::new(&folder_path)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_lowercase();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Look for color preset files
            // Pattern 1: _merged.lsf in [PAK]_DYE_Colors (ConsortDyes, EngelsDyes, etc.)
            // Pattern 2: Named preset file like GlowDye_Presets.lsf in [PAK]_* folders
            // Pattern 3: Individual UUID.lsf files in [PAK]_* folders (FaerunColors, FearTaylor)
            if file_name.ends_with(".lsf") || file_name.ends_with(".lsx") {
                let parent_name = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Pattern 1: _merged in [PAK]_DYE_Colors - preferred, single file with all dyes
                if color_file.is_none() && parent_name == "[PAK]_DYE_Colors" {
                    if file_name == "_merged.lsf" || file_name == "_merged.lsx" {
                        color_file = Some(path.to_path_buf());
                    }
                }

                // Pattern 2 & 3: LSF files in [PAK]_* folders under Content/
                // These could be preset files or individual dye files
                if parent_name.starts_with("[PAK]_")
                    && parent_name != "[PAK]_DYE_Colors"
                    && path_str.contains("content")
                    && !path_str.contains("roottemplates")
                    && !path_str.contains("/ui/")
                {
                    color_files.push(path.to_path_buf());
                }
            }

            // Look for Object.txt
            if object_file.is_none() {
                if file_name == "object.txt"
                    && path_str.contains("stats")
                    && path_str.contains("generated")
                {
                    object_file = Some(path.to_path_buf());
                }
            }

            // Look for RootTemplates LSX/LSF files
            if (file_name.ends_with(".lsx") || file_name.ends_with(".lsf"))
                && path_str.contains("roottemplates")
            {
                root_templates_files.push(path.to_path_buf());
            }

            // Look for localization XML file (in Localization/English/)
            if localization_file.is_none() {
                if file_name.ends_with(".xml")
                    && path_str.contains("localization")
                    && path_str.contains("english")
                {
                    localization_file = Some(path.to_path_buf());
                }
            }

            // Look for meta.lsx (anywhere in the mod folder)
            if meta_file.is_none() {
                if file_name == "meta.lsx" || file_name == "meta.lsf" {
                    meta_file = Some(path.to_path_buf());
                }
            }
        }

        // Parse color files - prefer _merged from [PAK]_DYE_Colors, fallback to individual files
        let mut lsf_entries = Vec::new();

        // Helper closure to parse a single LSF/LSX file
        let parse_color_file =
            |path: &std::path::PathBuf| -> Vec<crate::gui::state::ImportedDyeEntry> {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if ext == "lsf" {
                    if let Ok(lsf_doc) = lsf::read_lsf(path) {
                        if let Ok(lsx_content) = to_lsx(&lsf_doc) {
                            return parse_lsx_dye_presets(&lsx_content);
                        }
                    }
                } else if ext == "lsx" {
                    if let Ok(lsx_content) = fs::read_to_string(path) {
                        return parse_lsx_dye_presets(&lsx_content);
                    }
                }
                Vec::new()
            };

        if let Some(ref color_path) = color_file {
            // Primary: single merged file
            lsf_entries = parse_color_file(color_path);
        } else if !color_files.is_empty() {
            // Fallback: parse all individual color files
            for color_path in &color_files {
                let entries = parse_color_file(color_path);
                lsf_entries.extend(entries);
            }
        }

        // Parse the Object.txt file for metadata
        let mut object_entries: HashMap<String, ParsedDyeEntry> = HashMap::new();
        if let Some(ref obj_path) = object_file {
            if let Ok(content) = fs::read_to_string(obj_path) {
                for entry in parse_object_txt(&content) {
                    object_entries.insert(entry.name.clone(), entry);
                }
            }
        }

        // Correlate entries: add RootTemplate UUIDs from Object.txt to LSF entries
        for lsf_entry in &mut lsf_entries {
            if let Some(obj_entry) = object_entries.get(&lsf_entry.name) {
                if obj_entry.root_template_uuid.is_some() {
                    lsf_entry.root_template_uuid = obj_entry.root_template_uuid.clone();
                }
            }
        }

        // Parse RootTemplates to get localization handles
        let mut localization_handles: HashMap<String, (Option<String>, Option<String>)> =
            HashMap::new();
        for rt_path in &root_templates_files {
            let ext = rt_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let lsx_content = if ext == "lsf" {
                lsf::read_lsf(rt_path)
                    .ok()
                    .and_then(|doc| to_lsx(&doc).ok())
            } else {
                fs::read_to_string(rt_path).ok()
            };

            if let Some(content) = lsx_content {
                for info in parse_root_templates_localization(&content) {
                    localization_handles.insert(
                        info.name,
                        (info.display_name_handle, info.description_handle),
                    );
                }
            }
        }

        // Parse localization XML to get actual text
        let localization_map: HashMap<String, String> =
            if let Some(ref loc_path) = localization_file {
                fs::read_to_string(loc_path)
                    .ok()
                    .map(|content| parse_localization_xml(&content))
                    .unwrap_or_default()
            } else {
                HashMap::new()
            };

        // Correlate localization data with dye entries
        for lsf_entry in &mut lsf_entries {
            if let Some((display_handle, desc_handle)) = localization_handles.get(&lsf_entry.name) {
                // Look up display name
                if let Some(handle) = display_handle {
                    if let Some(text) = localization_map.get(handle) {
                        lsf_entry.display_name = text.clone();
                    }
                }
                // Look up description
                if let Some(handle) = desc_handle {
                    if let Some(text) = localization_map.get(handle) {
                        lsf_entry.description = text.clone();
                    }
                }
            }
        }

        // Parse meta.lsx to get mod name and author
        if let Some(ref meta_path) = meta_file {
            let ext = meta_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let lsx_content = if ext == "lsf" {
                lsf::read_lsf(meta_path)
                    .ok()
                    .and_then(|doc| to_lsx(&doc).ok())
            } else {
                fs::read_to_string(meta_path).ok()
            };

            if let Some(content) = lsx_content {
                let metadata = parse_meta_lsx(&content);
                imported_mod_name.set(metadata.name);
                imported_mod_author.set(metadata.author);
            }
        }

        // Returns findings
        let color_found = color_file.is_some() || !color_files.is_empty();
        let object_found = object_file.is_some();

        if lsf_entries.is_empty() {
            state.status_message.set(format!(
                "No dyes found in '{}' (colors: {}, metadata: {})",
                folder_name,
                if color_found { "found" } else { "not found" },
                if object_found { "found" } else { "not found" }
            ));
            return;
        }

        // Store the path for re-export (use the color file path)
        if let Some(ref color_path) = color_file {
            state
                .imported_lsf_path
                .set(Some(color_path.to_string_lossy().to_string()));
        }

        // Populate the LSF entries
        let count = lsf_entries.len();
        state.imported_lsf_entries.set(lsf_entries);
        state.selected_lsf_index.set(Some(0));

        // Load the first entry
        load_lsf_entry(state.clone(), imported_dye_name, imported_display_name);

        // Build status message
        let mut status_parts = vec![format!("Loaded {} dyes from '{}'", count, folder_name)];
        if !object_found {
            status_parts.push("(no Object.txt found)".to_string());
        }
        state.status_message.set(status_parts.join(" "));
    }
}

/// Load the selected TXT import entry into local display fields
pub fn load_selected_entry(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_display_name: RwSignal<String>,
) {
    let entries = state.imported_entries.get();
    if let Some(index) = state.selected_import_index.get() {
        if let Some((name, _preset_uuid, _root_template_uuid)) = entries.get(index) {
            imported_dye_name.set(name.clone());
            // TXT imports don't have display name
            imported_display_name.set(String::new());
            state.status_message.set(format!("Loaded: {}", name));
        }
    }
}

/// Load the selected LSF entry into local display fields and color pickers
pub fn load_lsf_entry(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_display_name: RwSignal<String>,
) {
    let entries = state.imported_lsf_entries.get();
    if let Some(index) = state.selected_lsf_index.get() {
        if let Some(entry) = entries.get(index) {
            // Set local display fields
            imported_dye_name.set(entry.name.clone());
            imported_display_name.set(entry.display_name.clone());

            // Reset all color pickers to default gray before applying imported colors
            reset_colors_to_default(&state);

            // Load colors from the imported entry
            load_colors_from_map(&state, &entry.colors);

            state.status_message.set(format!("Loaded: {}", entry.name));
        }
    }
}
