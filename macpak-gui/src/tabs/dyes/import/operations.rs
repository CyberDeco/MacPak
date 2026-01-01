//! Import operations and data loading functions

use std::fs;
use std::collections::HashMap;
use floem::prelude::*;

use crate::state::DyesState;
use super::super::shared::{ParsedDyeEntry, parse_item_combos, parse_object_txt, parse_lsx_dye_presets};

// MacLarian imports for LSF conversion (via MacPak re-export)
use MacPak::MacLarian::formats::lsf;
use MacPak::MacLarian::converter::{to_lsx, from_lsx};

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
            imported_template_uuid.set(String::new()); // LSF doesn't contain template UUID

            // Set colors in the shared color pickers (this is the main purpose of LSF import)
            for (param_name, hex_color) in &entry.colors {
                match param_name.as_str() {
                    "Cloth_Primary" => state.cloth_primary.hex.set(hex_color.clone()),
                    "Cloth_Secondary" => state.cloth_secondary.hex.set(hex_color.clone()),
                    "Cloth_Tertiary" => state.cloth_tertiary.hex.set(hex_color.clone()),
                    "Leather_Primary" => state.leather_primary.hex.set(hex_color.clone()),
                    "Leather_Secondary" => state.leather_secondary.hex.set(hex_color.clone()),
                    "Leather_Tertiary" => state.leather_tertiary.hex.set(hex_color.clone()),
                    "Metal_Primary" => state.metal_primary.hex.set(hex_color.clone()),
                    "Metal_Secondary" => state.metal_secondary.hex.set(hex_color.clone()),
                    "Metal_Tertiary" => state.metal_tertiary.hex.set(hex_color.clone()),
                    "Accent_Color" => state.accent_color.hex.set(hex_color.clone()),
                    "Color_01" => state.color_01.hex.set(hex_color.clone()),
                    "Color_02" => state.color_02.hex.set(hex_color.clone()),
                    "Color_03" => state.color_03.hex.set(hex_color.clone()),
                    "Custom_1" => state.custom_1.hex.set(hex_color.clone()),
                    "Custom_2" => state.custom_2.hex.set(hex_color.clone()),
                    // Recommended colors
                    "GlowColor" => state.glow_color.hex.set(hex_color.clone()),
                    "GlowColour" => state.glow_colour.hex.set(hex_color.clone()),
                    // Common colors
                    "AddedColor" => state.added_color.hex.set(hex_color.clone()),
                    "Highlight_Color" => state.highlight_color.hex.set(hex_color.clone()),
                    "BaseColor" => state.base_color.hex.set(hex_color.clone()),
                    "InnerColor" => state.inner_color.hex.set(hex_color.clone()),
                    "OuterColor" => state.outer_color.hex.set(hex_color.clone()),
                    "PrimaryColor" => state.primary_color.hex.set(hex_color.clone()),
                    "SecondaryColor" => state.secondary_color.hex.set(hex_color.clone()),
                    "TetriaryColor" => state.tetriary_color.hex.set(hex_color.clone()),
                    "Primary" => state.primary.hex.set(hex_color.clone()),
                    "Secondary" => state.secondary.hex.set(hex_color.clone()),
                    "Tertiary" => state.tertiary.hex.set(hex_color.clone()),
                    "Primary_Color" => state.primary_color_underscore.hex.set(hex_color.clone()),
                    "Secondary_Color" => state.secondary_color_underscore.hex.set(hex_color.clone()),
                    "Tertiary_Color" => state.tertiary_color_underscore.hex.set(hex_color.clone()),
                    _ => {} // Unknown parameter, ignore
                }
            }

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
        entries[idx].colors = collect_current_colors(&state);
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
fn generate_color_presets_lsx(entries: &[crate::state::ImportedDyeEntry]) -> String {
    let dye_nodes: Vec<String> = entries
        .iter()
        .map(|entry| {
            let color_nodes = generate_color_nodes(&entry.colors);
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

/// Generate Vector3Parameters nodes from a color HashMap
fn generate_color_nodes(colors: &HashMap<String, String>) -> String {
    colors
        .iter()
        .map(|(name, hex)| {
            let fvec3 = hex_to_fvec3(hex);
            format!(
                r#"								<node id="Vector3Parameters">
									<attribute id="Color" type="bool" value="True" />
									<attribute id="Custom" type="bool" value="False" />
									<attribute id="Enabled" type="bool" value="True" />
									<attribute id="Parameter" type="FixedString" value="{name}" />
									<attribute id="Value" type="fvec3" value="{fvec3}" />
								</node>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert hex color (e.g., "FF0000") to fvec3 string (e.g., "1 0 0")
fn hex_to_fvec3(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return "0.5 0.5 0.5".to_string();
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;

    format!("{:.6} {:.6} {:.6}", r, g, b)
}

/// Collect current colors from the color pickers into a HashMap
fn collect_current_colors(state: &DyesState) -> HashMap<String, String> {
    let mut colors = HashMap::new();

    // Required colors
    colors.insert("Cloth_Primary".to_string(), state.cloth_primary.hex.get());
    colors.insert("Cloth_Secondary".to_string(), state.cloth_secondary.hex.get());
    colors.insert("Cloth_Tertiary".to_string(), state.cloth_tertiary.hex.get());
    colors.insert("Leather_Primary".to_string(), state.leather_primary.hex.get());
    colors.insert("Leather_Secondary".to_string(), state.leather_secondary.hex.get());
    colors.insert("Leather_Tertiary".to_string(), state.leather_tertiary.hex.get());
    colors.insert("Metal_Primary".to_string(), state.metal_primary.hex.get());
    colors.insert("Metal_Secondary".to_string(), state.metal_secondary.hex.get());
    colors.insert("Metal_Tertiary".to_string(), state.metal_tertiary.hex.get());
    colors.insert("Accent_Color".to_string(), state.accent_color.hex.get());
    colors.insert("Color_01".to_string(), state.color_01.hex.get());
    colors.insert("Color_02".to_string(), state.color_02.hex.get());
    colors.insert("Color_03".to_string(), state.color_03.hex.get());
    colors.insert("Custom_1".to_string(), state.custom_1.hex.get());
    colors.insert("Custom_2".to_string(), state.custom_2.hex.get());

    // Recommended colors
    colors.insert("GlowColor".to_string(), state.glow_color.hex.get());
    colors.insert("GlowColour".to_string(), state.glow_colour.hex.get());

    // Common colors
    colors.insert("AddedColor".to_string(), state.added_color.hex.get());
    colors.insert("Highlight_Color".to_string(), state.highlight_color.hex.get());
    colors.insert("BaseColor".to_string(), state.base_color.hex.get());
    colors.insert("InnerColor".to_string(), state.inner_color.hex.get());
    colors.insert("OuterColor".to_string(), state.outer_color.hex.get());
    colors.insert("PrimaryColor".to_string(), state.primary_color.hex.get());
    colors.insert("SecondaryColor".to_string(), state.secondary_color.hex.get());
    colors.insert("TetriaryColor".to_string(), state.tetriary_color.hex.get());
    colors.insert("Primary".to_string(), state.primary.hex.get());
    colors.insert("Secondary".to_string(), state.secondary.hex.get());
    colors.insert("Tertiary".to_string(), state.tertiary.hex.get());
    colors.insert("Primary_Color".to_string(), state.primary_color_underscore.hex.get());
    colors.insert("Secondary_Color".to_string(), state.secondary_color_underscore.hex.get());
    colors.insert("Tertiary_Color".to_string(), state.tertiary_color_underscore.hex.get());

    colors
}
