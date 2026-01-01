//! Full mod export functionality - exports a complete ConsortDyes-like mod structure

use std::fs;
use std::path::Path;

use floem::prelude::*;

use crate::state::{DyesState, GeneratedDyeEntry};
use crate::utils::{generate_uuid, UuidFormat};
use super::generators::hex_to_fvec3;

/// Export result with status message
pub struct ExportResult {
    pub success: bool,
    pub message: String,
}

/// Default color value - colors matching this are skipped in export
const DEFAULT_COLOR: &str = "808080";

/// Check which required colors are still at default value
pub fn check_required_colors_at_default(state: &DyesState) -> Vec<&'static str> {
    let required = [
        ("Cloth_Primary", &state.cloth_primary),
        ("Cloth_Secondary", &state.cloth_secondary),
        ("Cloth_Tertiary", &state.cloth_tertiary),
        ("Leather_Primary", &state.leather_primary),
        ("Leather_Secondary", &state.leather_secondary),
        ("Leather_Tertiary", &state.leather_tertiary),
        ("Metal_Primary", &state.metal_primary),
        ("Metal_Secondary", &state.metal_secondary),
        ("Metal_Tertiary", &state.metal_tertiary),
        ("Color_01", &state.color_01),
        ("Color_02", &state.color_02),
        ("Color_03", &state.color_03),
        ("Custom_1", &state.custom_1),
        ("Custom_2", &state.custom_2),
    ];

    required
        .iter()
        .filter(|(_, entry)| {
            let hex = entry.hex.get();
            let normalized = hex.trim_start_matches('#').to_lowercase();
            normalized == DEFAULT_COLOR
        })
        .map(|(name, _)| *name)
        .collect()
}

/// Export a complete dye mod to the specified directory
pub fn export_dye_mod(state: &DyesState, output_dir: &Path, mod_name: &str) -> ExportResult {
    // Validate inputs
    if mod_name.is_empty() {
        return ExportResult {
            success: false,
            message: "Mod name is required".to_string(),
        };
    }

    let dyes = state.generated_dyes.get();
    if dyes.is_empty() {
        return ExportResult {
            success: false,
            message: "No dyes generated. Use 'Generate Dye' first.".to_string(),
        };
    }

    // Generate mod UUID
    let mod_uuid = generate_uuid(UuidFormat::Larian);

    // Create directory structure
    if let Err(e) = create_mod_structure(output_dir, mod_name) {
        return ExportResult {
            success: false,
            message: format!("Failed to create directories: {}", e),
        };
    }

    // Generate and write all files
    let results = vec![
        // Localization (all dyes combined)
        write_localization_xml(output_dir, mod_name, &dyes),
        write_placeholder_loca(output_dir, mod_name),

        // Meta
        write_meta_lsx(output_dir, mod_name, &mod_uuid),

        // Stats (all dyes combined)
        write_object_txt(output_dir, mod_name, &dyes),
        write_item_combos_txt(output_dir, mod_name, &dyes),
        write_treasure_table_txt(output_dir, mod_name, &dyes),

        // RootTemplates (all dyes combined)
        write_root_templates_lsx(output_dir, mod_name, &dyes),

        // Color Presets (all dyes combined)
        write_color_presets_lsx(output_dir, mod_name, &dyes),

        // GUI / Icons
        write_texture_atlas_info_lsx(output_dir, mod_name, &dyes),
        write_texture_bank_lsx(output_dir, mod_name),
        write_placeholder_dds(output_dir, mod_name, &dyes),
    ];

    // Check for errors
    for result in &results {
        if let Err(e) = result {
            return ExportResult {
                success: false,
                message: format!("Failed to write file: {}", e),
            };
        }
    }

    let count = dyes.len();
    ExportResult {
        success: true,
        message: format!("Exported {} dye{} to {}", count, if count == 1 { "" } else { "s" }, output_dir.display()),
    }
}

/// Create the mod directory structure
fn create_mod_structure(output_dir: &Path, mod_name: &str) -> std::io::Result<()> {
    let dirs = [
        format!("Localization/English"),
        format!("Mods/{}", mod_name),
        format!("Public/{}/Assets/Textures/Icons", mod_name),
        format!("Public/{}/Content/Assets/Characters/[PAK]_DYE_Colors", mod_name),
        format!("Public/{}/Content/UI/[PAK]_UI", mod_name),
        format!("Public/{}/GUI", mod_name),
        format!("Public/{}/RootTemplates", mod_name),
        format!("Public/{}/Stats/Generated/Data", mod_name),
        "Public/Game/GUI/Assets/Tooltips/ItemIcons".to_string(),
        "Public/Game/GUI/Assets/ControllerUIIcons/items_png".to_string(),
    ];

    for dir in &dirs {
        fs::create_dir_all(output_dir.join(dir))?;
    }

    Ok(())
}

/// Write localization XML for all dyes
fn write_localization_xml(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            let display_name = dye.name.replace('_', " ");
            let description = format!("A custom dye: {}", display_name);
            format!(
                r#"	<content contentuid="{}" version="1">{}</content>
	<content contentuid="{}" version="1">{}</content>"#,
                dye.name_handle, display_name, dye.desc_handle, description
            )
        })
        .collect();

    let content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<contentList>
{}
</contentList>
"#,
        entries.join("\n")
    );

    let path = output_dir.join(format!("Localization/English/{}.xml", mod_name));
    fs::write(path, content)
}

/// Write placeholder .loca file (minimal binary format)
fn write_placeholder_loca(output_dir: &Path, mod_name: &str) -> std::io::Result<()> {
    // Minimal .loca header - version + empty string count
    // This is a placeholder; real .loca files should be compiled from XML
    let placeholder: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let path = output_dir.join(format!("Localization/English/{}.loca", mod_name));
    fs::write(path, placeholder)
}

/// Write meta.lsx
fn write_meta_lsx(output_dir: &Path, mod_name: &str, mod_uuid: &str) -> std::io::Result<()> {
    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<save>
	<version major="4" minor="7" revision="1" build="3" lslib_meta="v1,bswap_guids" />
	<region id="Config">
		<node id="root">
			<children>
				<node id="Dependencies" />
				<node id="ModuleInfo">
					<attribute id="Author" type="LSString" value="MacPak" />
					<attribute id="CharacterCreationLevelName" type="FixedString" value="" />
					<attribute id="Description" type="LSString" value="Custom dye mod created with MacPak" />
					<attribute id="Folder" type="LSString" value="{mod_name}" />
					<attribute id="LobbyLevelName" type="FixedString" value="" />
					<attribute id="MD5" type="LSString" value="" />
					<attribute id="MainMenuBackgroundVideo" type="FixedString" value="" />
					<attribute id="MenuLevelName" type="FixedString" value="" />
					<attribute id="Name" type="LSString" value="{mod_name}" />
					<attribute id="NumPlayers" type="uint8" value="4" />
					<attribute id="PhotoBooth" type="FixedString" value="" />
					<attribute id="StartupLevelName" type="FixedString" value="" />
					<attribute id="Tags" type="LSString" value="" />
					<attribute id="Type" type="FixedString" value="Add-on" />
					<attribute id="UUID" type="FixedString" value="{mod_uuid}" />
					<attribute id="Version64" type="int64" value="36028797018963968" />
					<children>
						<node id="PublishVersion">
							<attribute id="Version64" type="int64" value="36028797018963968" />
						</node>
						<node id="TargetModes">
							<children>
								<node id="Target">
									<attribute id="Object" type="FixedString" value="Story" />
								</node>
							</children>
						</node>
					</children>
				</node>
			</children>
		</node>
	</region>
</save>
"#
    );

    let path = output_dir.join(format!("Mods/{}/meta.lsx", mod_name));
    fs::write(path, content)
}

/// Write Object.txt (stats) for all dyes
fn write_object_txt(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            format!(
                r#"new entry "{}"
using "_Dyes"
data "RootTemplate" "{}""#,
                dye.name, dye.template_uuid
            )
        })
        .collect();

    let content = entries.join("\n\n");

    let path = output_dir.join(format!(
        "Public/{}/Stats/Generated/Data/Object.txt",
        mod_name
    ));
    fs::write(path, content)
}

/// Write ItemCombos.txt for all dyes
fn write_item_combos_txt(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            format!(
                r#"new ItemCombination "{name}"
data "Type 1" "Object"
data "Object 1" "{name}"
data "Transform 1" "None"
data "Type 2" "Category"
data "Object 2" "DyableArmor"
data "Transform 2" "Dye"
data "DyeColorPresetResource" "{preset_uuid}"

new ItemCombinationResult "{name}_1"
data "ResultAmount 1" "1""#,
                name = dye.name,
                preset_uuid = dye.preset_uuid
            )
        })
        .collect();

    let content = entries.join("\n\n");

    let path = output_dir.join(format!(
        "Public/{}/Stats/Generated/ItemCombos.txt",
        mod_name
    ));
    fs::write(path, content)
}

/// Write TreasureTable.txt for all dyes
fn write_treasure_table_txt(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            format!(
                r#"new treasuretable "{name}_TT"
CanMerge 1
new subtable "1,1"
object category "I_{name}",1,0,0,0,0,0,0,0"#,
                name = dye.name
            )
        })
        .collect();

    let content = entries.join("\n\n");

    let path = output_dir.join(format!(
        "Public/{}/Stats/Generated/TreasureTable.txt",
        mod_name
    ));
    fs::write(path, content)
}

/// Write RootTemplates LSX for all dyes
fn write_root_templates_lsx(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            let icon_name = format!("{}_Icon", dye.name);
            format!(
                r#"				<node id="GameObjects">
					<attribute id="MapKey" type="FixedString" value="{template_uuid}" />
					<attribute id="Name" type="LSString" value="{name}" />
					<attribute id="LevelName" type="FixedString" value="" />
					<attribute id="Type" type="FixedString" value="item" />
					<attribute id="ParentTemplateId" type="FixedString" value="1a750a66-e5c2-40be-9f62-0a4bf3ddb403" />
					<attribute id="DisplayName" type="TranslatedString" handle="{name_handle}" version="1" />
					<attribute id="Icon" type="FixedString" value="{icon_name}" />
					<attribute id="Stats" type="FixedString" value="{name}" />
					<attribute id="Description" type="TranslatedString" handle="{desc_handle}" version="1" />
					<attribute id="ColorPreset" type="guid" value="{preset_uuid}" />
				</node>"#,
                template_uuid = dye.template_uuid,
                name = dye.name,
                name_handle = dye.name_handle,
                icon_name = icon_name,
                desc_handle = dye.desc_handle,
                preset_uuid = dye.preset_uuid,
            )
        })
        .collect();

    let content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<save>
	<version major="4" minor="7" revision="1" build="3" lslib_meta="v1,bswap_guids" />
	<region id="Templates">
		<node id="Templates">
			<children>
{}
			</children>
		</node>
	</region>
</save>
"#,
        entries.join("\n")
    );

    let path = output_dir.join(format!(
        "Public/{}/RootTemplates/{}_Dyes.lsx",
        mod_name, mod_name
    ));
    fs::write(path, content)
}

/// Write color presets LSX (MaterialPresetBank) for all dyes
fn write_color_presets_lsx(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            let color_nodes = generate_color_nodes_from_hashmap(&dye.colors);
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
                preset_uuid = dye.preset_uuid,
                name = dye.name,
                color_nodes = color_nodes,
            )
        })
        .collect();

    let content = format!(
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
        entries.join("\n")
    );

    let path = output_dir.join(format!(
        "Public/{}/Content/Assets/Characters/[PAK]_DYE_Colors/_merged.lsx",
        mod_name
    ));
    fs::write(path, content)
}

/// Generate color nodes from a HashMap of colors
/// Skips colors that are unchanged from the default (#808080)
fn generate_color_nodes_from_hashmap(colors: &std::collections::HashMap<String, String>) -> String {
    colors
        .iter()
        .filter(|(_, hex)| {
            // Skip colors that match the default
            let normalized = hex.trim_start_matches('#').to_lowercase();
            normalized != DEFAULT_COLOR
        })
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

/// Write TextureAtlasInfo LSX (icon atlas metadata) for all dyes
fn write_texture_atlas_info_lsx(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let atlas_uuid = generate_uuid(UuidFormat::Larian);
    let dye_count = dyes.len();

    // Calculate atlas dimensions (arrange icons in a grid)
    let icons_per_row = (dye_count as f32).sqrt().ceil() as u32;
    let num_rows = ((dye_count as f32) / (icons_per_row as f32)).ceil() as u32;
    let atlas_width = icons_per_row * 64;
    let atlas_height = num_rows * 64;

    // Generate UV entries for each dye icon
    let uv_entries: Vec<String> = dyes
        .iter()
        .enumerate()
        .map(|(i, dye)| {
            let icon_name = format!("{}_Icon", dye.name);
            let col = (i as u32) % icons_per_row;
            let row = (i as u32) / icons_per_row;
            let u1 = (col * 64) as f32 / atlas_width as f32;
            let u2 = ((col + 1) * 64) as f32 / atlas_width as f32;
            let v1 = (row * 64) as f32 / atlas_height as f32;
            let v2 = ((row + 1) * 64) as f32 / atlas_height as f32;
            format!(
                r#"				<node id="IconUV">
					<attribute id="MapKey" type="FixedString" value="{}"/>
					<attribute id="U1" type="float" value="{}"/>
					<attribute id="U2" type="float" value="{}"/>
					<attribute id="V1" type="float" value="{}"/>
					<attribute id="V2" type="float" value="{}"/>
				</node>"#,
                icon_name, u1, u2, v1, v2
            )
        })
        .collect();

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<save>
	<version major="4" minor="0" revision="6" build="5" />
	<region id="TextureAtlasInfo">
		<node id="root">
			<children>
				<node id="TextureAtlasIconSize">
					<attribute id="Height" type="int64" value="64"/>
					<attribute id="Width" type="int64" value="64"/>
				</node>
				<node id="TextureAtlasPath">
					<attribute id="Path" type="LSString" value="Assets/Textures/Icons/{mod_name}_Icons.dds"/>
					<attribute id="UUID" type="FixedString" value="{atlas_uuid}"/>
				</node>
				<node id="TextureAtlasTextureSize">
					<attribute id="Height" type="int64" value="{atlas_height}"/>
					<attribute id="Width" type="int64" value="{atlas_width}"/>
				</node>
			</children>
		</node>
	</region>
	<region id="IconUVList">
		<node id="root">
			<children>
{}
			</children>
		</node>
	</region>
</save>
"#,
        uv_entries.join("\n")
    );

    let path = output_dir.join(format!(
        "Public/{}/GUI/{}_TextureAtlasInfo.lsx",
        mod_name, mod_name
    ));
    fs::write(path, content)
}

/// Write TextureBank LSX (UI texture references)
fn write_texture_bank_lsx(output_dir: &Path, mod_name: &str) -> std::io::Result<()> {
    let texture_uuid = generate_uuid(UuidFormat::Larian);

    let content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<save>
	<version major="4" minor="7" revision="1" build="3" lslib_meta="v1,bswap_guids" />
	<region id="TextureBank">
		<node id="TextureBank">
			<children>
				<node id="Resource">
					<attribute id="ID" type="FixedString" value="{texture_uuid}" />
					<attribute id="Localized" type="bool" value="False" />
					<attribute id="Name" type="LSString" value="{mod_name}_Icons" />
					<attribute id="SRGB" type="bool" value="True" />
					<attribute id="SourceFile" type="LSString" value="Public/{mod_name}/Assets/Textures/Icons/{mod_name}_Icons.dds" />
					<attribute id="Streaming" type="bool" value="True" />
					<attribute id="Template" type="FixedString" value="{mod_name}_Icons" />
					<attribute id="Type" type="int32" value="0" />
				</node>
			</children>
		</node>
	</region>
</save>
"#
    );

    let path = output_dir.join(format!(
        "Public/{}/Content/UI/[PAK]_UI/{}_TextureBank.lsx",
        mod_name, mod_name
    ));
    fs::write(path, content)
}

/// Write placeholder DDS files (64x64 magenta for visibility)
fn write_placeholder_dds(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let dye_count = dyes.len();

    // Calculate atlas dimensions
    let icons_per_row = (dye_count as f32).sqrt().ceil() as u32;
    let num_rows = ((dye_count as f32) / (icons_per_row as f32)).ceil() as u32;
    let atlas_width = icons_per_row * 64;
    let atlas_height = num_rows * 64;

    // Create atlas DDS
    let atlas_dds = create_dds_data(atlas_width, atlas_height, dyes);

    // Write atlas
    let atlas_path = output_dir.join(format!(
        "Public/{}/Assets/Textures/Icons/{}_Icons.dds",
        mod_name, mod_name
    ));
    fs::write(atlas_path, &atlas_dds)?;

    // Write individual icon DDS files for each dye
    let single_icon_dds = create_single_icon_dds();
    for dye in dyes {
        let icon_name = format!("{}_Icon", dye.name);
        let paths = [
            format!("Public/Game/GUI/Assets/Tooltips/ItemIcons/{}.DDS", icon_name),
            format!("Public/Game/GUI/Assets/ControllerUIIcons/items_png/{}.DDS", icon_name),
        ];
        for path in &paths {
            fs::write(output_dir.join(path), &single_icon_dds)?;
        }
    }

    Ok(())
}

/// Create DDS data for an atlas with multiple icons
fn create_dds_data(width: u32, height: u32, dyes: &[GeneratedDyeEntry]) -> Vec<u8> {
    let icons_per_row = (dyes.len() as f32).sqrt().ceil() as u32;

    let mut dds_data = Vec::with_capacity(128 + (width * height * 4) as usize);

    // DDS header
    dds_data.extend_from_slice(b"DDS ");
    dds_data.extend_from_slice(&124u32.to_le_bytes());
    dds_data.extend_from_slice(&0x1007u32.to_le_bytes());
    dds_data.extend_from_slice(&height.to_le_bytes());
    dds_data.extend_from_slice(&width.to_le_bytes());
    dds_data.extend_from_slice(&(width * 4).to_le_bytes());
    dds_data.extend_from_slice(&0u32.to_le_bytes());
    dds_data.extend_from_slice(&1u32.to_le_bytes());
    for _ in 0..11 {
        dds_data.extend_from_slice(&0u32.to_le_bytes());
    }
    dds_data.extend_from_slice(&32u32.to_le_bytes());
    dds_data.extend_from_slice(&0x41u32.to_le_bytes());
    dds_data.extend_from_slice(&0u32.to_le_bytes());
    dds_data.extend_from_slice(&32u32.to_le_bytes());
    dds_data.extend_from_slice(&0x00FF0000u32.to_le_bytes());
    dds_data.extend_from_slice(&0x0000FF00u32.to_le_bytes());
    dds_data.extend_from_slice(&0x000000FFu32.to_le_bytes());
    dds_data.extend_from_slice(&0xFF000000u32.to_le_bytes());
    dds_data.extend_from_slice(&0x1000u32.to_le_bytes());
    for _ in 0..4 {
        dds_data.extend_from_slice(&0u32.to_le_bytes());
    }

    // Generate pixel data with different colors for each dye
    let colors: Vec<(u8, u8, u8)> = dyes
        .iter()
        .map(|dye| {
            // Use the Cloth_Primary color as the icon color
            if let Some(hex) = dye.colors.get("Cloth_Primary") {
                parse_hex_color(hex)
            } else {
                (255, 0, 255) // Default magenta
            }
        })
        .collect();

    for y in 0..height {
        for x in 0..width {
            let icon_col = x / 64;
            let icon_row = y / 64;
            let icon_idx = (icon_row * icons_per_row + icon_col) as usize;

            let (r, g, b) = if icon_idx < colors.len() {
                colors[icon_idx]
            } else {
                (128, 128, 128) // Gray for empty slots
            };

            dds_data.push(b);   // Blue
            dds_data.push(g);   // Green
            dds_data.push(r);   // Red
            dds_data.push(255); // Alpha
        }
    }

    dds_data
}

/// Create a single 64x64 icon DDS
fn create_single_icon_dds() -> Vec<u8> {
    let mut dds_data = Vec::with_capacity(128 + 64 * 64 * 4);

    // DDS header
    dds_data.extend_from_slice(b"DDS ");
    dds_data.extend_from_slice(&124u32.to_le_bytes());
    dds_data.extend_from_slice(&0x1007u32.to_le_bytes());
    dds_data.extend_from_slice(&64u32.to_le_bytes());
    dds_data.extend_from_slice(&64u32.to_le_bytes());
    dds_data.extend_from_slice(&(64 * 4u32).to_le_bytes());
    dds_data.extend_from_slice(&0u32.to_le_bytes());
    dds_data.extend_from_slice(&1u32.to_le_bytes());
    for _ in 0..11 {
        dds_data.extend_from_slice(&0u32.to_le_bytes());
    }
    dds_data.extend_from_slice(&32u32.to_le_bytes());
    dds_data.extend_from_slice(&0x41u32.to_le_bytes());
    dds_data.extend_from_slice(&0u32.to_le_bytes());
    dds_data.extend_from_slice(&32u32.to_le_bytes());
    dds_data.extend_from_slice(&0x00FF0000u32.to_le_bytes());
    dds_data.extend_from_slice(&0x0000FF00u32.to_le_bytes());
    dds_data.extend_from_slice(&0x000000FFu32.to_le_bytes());
    dds_data.extend_from_slice(&0xFF000000u32.to_le_bytes());
    dds_data.extend_from_slice(&0x1000u32.to_le_bytes());
    for _ in 0..4 {
        dds_data.extend_from_slice(&0u32.to_le_bytes());
    }

    // Magenta pixels
    for _ in 0..(64 * 64) {
        dds_data.push(0xFF); // Blue
        dds_data.push(0x00); // Green
        dds_data.push(0xFF); // Red
        dds_data.push(0xFF); // Alpha
    }

    dds_data
}

/// Parse hex color string to RGB tuple
fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        (r, g, b)
    } else {
        (255, 0, 255) // Default magenta
    }
}
