//! Import operations and data loading functions

use std::fs;
use std::collections::HashMap;
use floem::prelude::*;
use walkdir::WalkDir;

use crate::gui::state::DyesState;
use super::super::shared::{
    ParsedDyeEntry, parse_item_combos, parse_object_txt, parse_lsx_dye_presets,
    generate_all_color_nodes, collect_all_colors, reset_colors_to_default, load_colors_from_map,
};

// MacLarian imports for LSF conversion (via MacPak re-export)
use crate::MacLarian::formats::lsf;
use crate::MacLarian::converter::{to_lsx, from_lsx};

/// Import from a mod file (ItemCombos.txt or Object.txt)
pub fn import_from_file(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_preset_uuid: RwSignal<String>,
    imported_template_uuid: RwSignal<String>,
) {
    let dialog = rfd::FileDialog::new()
        .set_title("Import from Mod File")
        .add_filter("BG3 Stat Files", &["txt"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.pick_file() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Try to parse based on content
                let item_combo_entries = parse_item_combos(&content);
                let object_entries = parse_object_txt(&content);

                // Merge entries by name
                let mut merged: HashMap<String, ParsedDyeEntry> = HashMap::new();

                for entry in item_combo_entries {
                    merged.insert(entry.name.clone(), entry);
                }

                for entry in object_entries {
                    if let Some(existing) = merged.get_mut(&entry.name) {
                        if entry.root_template_uuid.is_some() {
                            existing.root_template_uuid = entry.root_template_uuid;
                        }
                    } else {
                        merged.insert(entry.name.clone(), entry);
                    }
                }

                if merged.is_empty() {
                    state.status_message.set(format!("No dye entries found in {}", filename));
                    return;
                }

                // Convert to vector and sort by name
                let mut entries: Vec<_> = merged.into_values()
                    .map(|e| (e.name, e.preset_uuid, e.root_template_uuid))
                    .collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));

                let count = entries.len();
                state.imported_entries.set(entries);
                state.selected_import_index.set(Some(0));
                load_selected_entry(state.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);
                state.status_message.set(format!("Loaded 1 of {} dyes", count));
            }
            Err(e) => {
                state.status_message.set(format!("Error reading file: {}", e));
            }
        }
    }
}

/// Import from an LSF file (DyeColorPresets)
pub fn import_from_lsf(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_preset_uuid: RwSignal<String>,
    imported_template_uuid: RwSignal<String>,
) {
    let dialog = rfd::FileDialog::new()
        .set_title("Import DyeColorPresets LSF")
        .add_filter("LSF Files", &["lsf"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.pick_file() {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        // Read and convert LSF to LSX
        match lsf::read_lsf(&path) {
            Ok(lsf_doc) => {
                match to_lsx(&lsf_doc) {
                    Ok(lsx_content) => {
                        // Parse the LSX content
                        let entries = parse_lsx_dye_presets(&lsx_content);

                        if entries.is_empty() {
                            state.status_message.set(format!("No dye presets found in {}", filename));
                            return;
                        }

                        let count = entries.len();
                        state.imported_lsf_entries.set(entries);
                        state.selected_lsf_index.set(Some(0));
                        // Store the path for re-export
                        state.imported_lsf_path.set(Some(path.to_string_lossy().to_string()));

                        // Load first entry
                        load_lsf_entry(state.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);

                        state.status_message.set(format!("Loaded 1 of {} dyes from {}", count, filename));
                    }
                    Err(e) => {
                        state.status_message.set(format!("Error converting LSF: {}", e));
                    }
                }
            }
            Err(e) => {
                state.status_message.set(format!("Error reading LSF: {}", e));
            }
        }
    }
}

/// Import from an extracted mod folder
/// Automatically discovers _merged.lsf/lsx (colors) and Object.txt (metadata)
pub fn import_from_mod_folder(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_preset_uuid: RwSignal<String>,
    imported_template_uuid: RwSignal<String>,
) {
    let dialog = rfd::FileDialog::new()
        .set_title("Select Extracted Mod Folder");

    if let Some(folder_path) = dialog.pick_folder() {
        // Clear previous imports
        state.imported_entries.set(Vec::new());
        state.selected_import_index.set(None);
        state.imported_lsf_entries.set(Vec::new());
        state.selected_lsf_index.set(None);
        state.imported_lsf_path.set(None);
        imported_dye_name.set(String::new());
        imported_preset_uuid.set(String::new());
        imported_template_uuid.set(String::new());

        let folder_name = folder_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("mod")
            .to_string();

        // Search for dye color files
        let mut color_file: Option<std::path::PathBuf> = None;  // Primary: _merged in [PAK]_DYE_Colors
        let mut color_files: Vec<std::path::PathBuf> = Vec::new();  // Fallback: individual LSF files
        let mut object_file: Option<std::path::PathBuf> = None;

        for entry in WalkDir::new(&folder_path)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_lowercase();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Look for color preset files
            // Pattern 1: _merged.lsf in [PAK]_DYE_Colors (ConsortDyes, EngelsDyes, etc.)
            // Pattern 2: Named preset file like GlowDye_Presets.lsf in [PAK]_* folders
            // Pattern 3: Individual UUID.lsf files in [PAK]_* folders (FaerunColors, FearTaylor)
            if file_name.ends_with(".lsf") || file_name.ends_with(".lsx") {
                let parent_name = path.parent()
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

        }

        // Parse color files - prefer _merged from [PAK]_DYE_Colors, fallback to individual files
        let mut lsf_entries = Vec::new();

        // Helper closure to parse a single LSF/LSX file
        let parse_color_file = |path: &std::path::PathBuf| -> Vec<crate::gui::state::ImportedDyeEntry> {
            let ext = path.extension()
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

        // Report what we found
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
            state.imported_lsf_path.set(Some(color_path.to_string_lossy().to_string()));
        }

        // Populate the LSF entries
        let count = lsf_entries.len();
        state.imported_lsf_entries.set(lsf_entries);
        state.selected_lsf_index.set(Some(0));

        // Load the first entry
        load_lsf_entry(state.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);

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
    imported_preset_uuid: RwSignal<String>,
    imported_template_uuid: RwSignal<String>,
) {
    let entries = state.imported_entries.get();
    if let Some(index) = state.selected_import_index.get() {
        if let Some((name, preset_uuid, root_template_uuid)) = entries.get(index) {
            imported_dye_name.set(name.clone());
            imported_preset_uuid.set(preset_uuid.clone().unwrap_or_default());
            imported_template_uuid.set(root_template_uuid.clone().unwrap_or_default());
            state.status_message.set(format!("Loaded: {}", name));
        }
    }
}

/// Load the selected LSF entry into local display fields and color pickers
pub fn load_lsf_entry(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_preset_uuid: RwSignal<String>,
    imported_template_uuid: RwSignal<String>,
) {
    let entries = state.imported_lsf_entries.get();
    if let Some(index) = state.selected_lsf_index.get() {
        if let Some(entry) = entries.get(index) {
            // Set local display fields
            imported_dye_name.set(entry.name.clone());
            imported_preset_uuid.set(entry.preset_uuid.clone().unwrap_or_default());
            imported_template_uuid.set(entry.root_template_uuid.clone().unwrap_or_default());

            // Reset all color pickers to default gray before applying imported colors
            reset_colors_to_default(&state);

            // Load colors from the imported entry
            load_colors_from_map(&state, &entry.colors);

            state.status_message.set(format!("Loaded: {}", entry.name));
        }
    }
}

/// Update the selected LSF entry with current color picker values
pub fn update_lsf_entry(state: DyesState) {
    let idx = state.selected_lsf_index.get().unwrap_or(0);
    let mut entries = state.imported_lsf_entries.get();
    if idx < entries.len() {
        let name = entries[idx].name.clone();
        entries[idx].colors = collect_all_colors(&state);
        state.imported_lsf_entries.set(entries);
        state.status_message.set(format!("Updated '{}'", name));
    }
}

/// Re-export the imported LSF entries back to the original file
pub fn reexport_lsf(state: DyesState) {
    let entries = state.imported_lsf_entries.get();
    if entries.is_empty() {
        state.status_message.set("No LSF entries to export".to_string());
        return;
    }

    let path = match state.imported_lsf_path.get() {
        Some(p) => p,
        None => {
            state.status_message.set("No import path stored - import an LSF file first".to_string());
            return;
        }
    };

    // Generate LSX content from entries
    let lsx_content = generate_color_presets_lsx(&entries);

    // Convert to LSF and write
    match from_lsx(&lsx_content) {
        Ok(lsf_doc) => {
            match lsf::write_lsf(&lsf_doc, std::path::Path::new(&path)) {
                Ok(_) => {
                    let filename = std::path::Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file");
                    state.status_message.set(format!("Re-exported {} dyes to {}", entries.len(), filename));
                }
                Err(e) => {
                    state.status_message.set(format!("Failed to write LSF: {}", e));
                }
            }
        }
        Err(e) => {
            state.status_message.set(format!("Failed to generate LSF: {}", e));
        }
    }
}

/// Generate MaterialPresetBank LSX content from imported dye entries
fn generate_color_presets_lsx(entries: &[crate::gui::state::ImportedDyeEntry]) -> String {
    let dye_nodes: Vec<String> = entries
        .iter()
        .map(|entry| {
            let color_nodes = generate_all_color_nodes(&entry.colors);
            let preset_uuid = entry.preset_uuid.clone().unwrap_or_default();
            format!(
                r#"				<node id="Resource">
					<attribute id="ID" type="FixedString" value="{preset_uuid}" />
					<attribute id="Name" type="LSString" value="{name}" />
					<children>
						<node id="Presets">
							<attribute id="MaterialResource" type="FixedString" value="" />
							<children>
								<node id="ColorPreset">
									<attribute id="ForcePresetValues" type="bool" value="False" />
									<attribute id="GroupName" type="FixedString" value="" />
									<attribute id="MaterialPresetResource" type="FixedString" value="" />
								</node>
								<node id="MaterialPresets" />
{color_nodes}
							</children>
						</node>
					</children>
				</node>"#,
                preset_uuid = preset_uuid,
                name = entry.name,
                color_nodes = color_nodes,
            )
        })
        .collect();

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<save>
	<version major="4" minor="7" revision="1" build="3" lslib_meta="v1,bswap_guids" />
	<region id="MaterialPresetBank">
		<node id="MaterialPresetBank">
			<children>
{}
			</children>
		</node>
	</region>
</save>
"#,
        dye_nodes.join("\n")
    )
}

