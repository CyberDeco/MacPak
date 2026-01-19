//! Full mod export functionality - exports a complete ConsortDyes-like mod structure

use std::fs;
use std::path::Path;

use floem::prelude::*;

use crate::gui::state::{DyesState, GeneratedDyeEntry};
use crate::gui::utils::{generate_uuid, generate_meta_lsx, UuidFormat};
use super::super::shared::{generate_color_nodes, parse_hex_color, required_colors};

/// Base game dye item template that all custom dyes inherit from
/// This is the "LOOT_Dye_Generic" template from Shared.pak
const DYE_PARENT_TEMPLATE_ID: &str = "1a750a66-e5c2-40be-9f62-0a4bf3ddb403";

// maclarian imports for LSF and LOCA conversion
use crate::maclarian::converter::{from_lsx, loca_from_xml};
use crate::maclarian::formats::lsf::write_lsf;
use crate::maclarian::formats::loca::write_loca;

/// Write LSX content as LSF binary file
fn write_lsx_as_lsf<P: AsRef<Path>>(lsx_content: &str, dest: P) -> std::io::Result<()> {
    let lsf_doc = from_lsx(lsx_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    write_lsf(&lsf_doc, dest)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

/// Write XML content as .loca binary file
fn write_xml_as_loca<P: AsRef<Path>>(xml_content: &str, dest: P) -> std::io::Result<()> {
    let loca_resource = loca_from_xml(xml_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    write_loca(dest, &loca_resource)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

/// Check which required colors are still at default value
pub fn check_required_colors_at_default(state: &DyesState) -> Vec<&'static str> {
    required_colors()
        .filter(|def| state.is_color_default(def.name))
        .map(|def| def.name)
        .collect()
}

/// Export a complete dye mod to the specified directory
pub fn export_dye_mod(state: &DyesState, output_dir: &Path, mod_name: &str) -> String {
    // Validate inputs
    if mod_name.is_empty() {
        return "Mod name is required".to_string();
    }

    // Append mod_name to output directory
    let output_dir = output_dir.join(mod_name);
    let output_dir = output_dir.as_path();

    let dyes = state.generated_dyes.get();
    if dyes.is_empty() {
        return "No dyes generated. Use 'Generate Dye' first.".to_string();
    }

    // Get mod metadata from state (or generate if empty)
    let mod_uuid = {
        let uuid = state.mod_uuid.get();
        if uuid.is_empty() {
            generate_uuid(UuidFormat::Standard)
        } else {
            uuid
        }
    };
    let author = state.mod_author.get();
    let description = state.mod_description.get();
    let version_major = state.mod_version_major.get();
    let version_minor = state.mod_version_minor.get();
    let version_patch = state.mod_version_patch.get();
    let version_build = state.mod_version_build.get();

    // Generate container (pouch) UUIDs and handles
    let container_template_uuid = generate_uuid(UuidFormat::Standard);
    let container_name_handle = generate_uuid(UuidFormat::Larian);
    let container_desc_handle = generate_uuid(UuidFormat::Larian);

    // Create directory structure
    if let Err(e) = create_mod_structure(output_dir, mod_name) {
        return format!("Failed to create directories: {}", e);
    }

    // Generate and write all files
    let results = vec![
        // Localization (all dyes combined - XML + .loca binary)
        write_localization_files(output_dir, mod_name, &dyes,
            &container_name_handle, &container_desc_handle),

        // Meta
        write_meta_lsx(output_dir, mod_name, &mod_uuid, &author, &description,
            version_major, version_minor, version_patch, version_build),

        // Stats (all dyes combined)
        write_object_txt(output_dir, mod_name, &dyes, &container_template_uuid),
        write_item_combos_txt(output_dir, mod_name, &dyes),
        write_treasure_table_txt(output_dir, mod_name, &dyes, &state.selected_vendors.get()),

        // RootTemplates (all dyes combined)
        write_root_templates_lsx(output_dir, mod_name, &dyes,
            &container_template_uuid, &container_name_handle, &container_desc_handle),

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
            return format!("Failed to write file: {}", e);
        }
    }

    let count = dyes.len();
    format!("Exported {} dye{} to {}", count, if count == 1 { "" } else { "s" }, output_dir.display())
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

/// Write localization files (both XML and .loca binary)
fn write_localization_files(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
    container_name_handle: &str,
    container_desc_handle: &str,
) -> std::io::Result<()> {
    // Dye entries
    let dye_entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            format!(
                r#"	<content contentuid="{}" version="1">{}</content>
	<content contentuid="{}" version="1">{}</content>"#,
                dye.name_handle, dye.display_name, dye.desc_handle, dye.description
            )
        })
        .collect();

    // Container entries
    let container_entries = format!(
        r#"	<content contentuid="{}" version="1">{} Dye Pouch</content>
	<content contentuid="{}" version="1">A pouch containing all {} dyes. Open it to add them to your inventory.</content>"#,
        container_name_handle, mod_name,
        container_desc_handle, mod_name
    );

    let mut all_entries = dye_entries;
    all_entries.push(container_entries);

    let content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<contentList>
{}
</contentList>
"#,
        all_entries.join("\n")
    );

    // Write XML (for reference/editing)
    let xml_path = output_dir.join(format!("Localization/English/{}.xml", mod_name));
    fs::write(xml_path, &content)?;

    // Write .loca binary (what the game uses)
    let loca_path = output_dir.join(format!("Localization/English/{}.loca", mod_name));
    write_xml_as_loca(&content, loca_path)
}

/// Write meta.lsx using the shared meta generator
/// Note: meta.lsx is always XML, never binary LSF
fn write_meta_lsx(
    output_dir: &Path,
    mod_name: &str,
    mod_uuid: &str,
    author: &str,
    description: &str,
    version_major: u32,
    version_minor: u32,
    version_patch: u32,
    version_build: u32,
) -> std::io::Result<()> {
    let content = generate_meta_lsx(
        mod_name,
        mod_name, // folder = mod_name
        author,
        description,
        mod_uuid,
        version_major,
        version_minor,
        version_patch,
        version_build,
    );

    let path = output_dir.join(format!("Mods/{}/meta.lsx", mod_name));
    fs::write(path, content)
}

/// Write Object.txt (stats) for all dyes + container item
fn write_object_txt(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
    container_template_uuid: &str,
) -> std::io::Result<()> {
    // Individual dye entries
    let dye_entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            format!(
                r#"new entry "{}"
type "Object"
using "_Dyes"
data "RootTemplate" "{}""#,
                dye.name, dye.template_uuid
            )
        })
        .collect();

    // Container item entry (a pouch/bag that contains all dyes)
    let container_entry = format!(
        r#"new entry "{mod_name}_DyePouch"
type "Object"
using "OBJ_Pouch"
data "RootTemplate" "{container_template_uuid}""#
    );

    let mut content = dye_entries.join("\n\n");
    content.push_str("\n\n");
    content.push_str(&container_entry);

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

/// Write TreasureTable.txt for all dyes with vendor integration
fn write_treasure_table_txt(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
    selected_vendors: &[bool],
) -> std::io::Result<()> {
    use crate::gui::state::VENDOR_DEFS;

    let mut content = String::new();

    // Main treasure table containing all individual dyes
    let subtables: Vec<String> = dyes
        .iter()
        .map(|dye| {
            format!(
                r#"new subtable "1,1"
object category "I_{}",1,0,0,0,0,0,0,0"#,
                dye.name
            )
        })
        .collect();

    content.push_str(&format!(
        r#"new treasuretable "{mod_name}_Dyes"
{subtables}

"#,
        subtables = subtables.join("\n")
    ));

    // Container treasure table (what the pouch spawns)
    content.push_str(&format!(
        r#"new treasuretable "{mod_name}_DyePouch_Contents"
CanMerge 1
new subtable "1,1"
object category "I_{mod_name}_DyePouch",1,0,0,0,0,0,0,0

"#
    ));

    // Add container to selected vendor tables
    for (idx, vendor) in VENDOR_DEFS.iter().enumerate() {
        // Include if always enabled OR if selected
        let is_selected = vendor.always_enabled || selected_vendors.get(idx).copied().unwrap_or(false);
        if is_selected {
            content.push_str(&format!(
                r#"new treasuretable "{}"
CanMerge 1
new subtable "1,1"
object category "I_{mod_name}_DyePouch",1,0,0,0,0,0,0,0

"#,
                vendor.id
            ));
        }
    }

    let path = output_dir.join(format!(
        "Public/{}/Stats/Generated/TreasureTable.txt",
        mod_name
    ));
    fs::write(path, content.trim_end())
}

/// Base game pouch template that the dye container inherits from
const POUCH_PARENT_TEMPLATE_ID: &str = "3e6aac21-333b-4812-a554-376c2d157ba9";

/// Write RootTemplates LSX for all dyes + container
fn write_root_templates_lsx(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
    container_template_uuid: &str,
    container_name_handle: &str,
    container_desc_handle: &str,
) -> std::io::Result<()> {
    // Individual dye entries
    let dye_entries: Vec<String> = dyes
        .iter()
        .map(|dye| {
            let icon_name = format!("{}_Icon", dye.name);
            format!(
                r#"				<node id="GameObjects">
					<attribute id="MapKey" type="FixedString" value="{template_uuid}" />
					<attribute id="Name" type="LSString" value="{name}" />
					<attribute id="LevelName" type="FixedString" value="" />
					<attribute id="Type" type="FixedString" value="item" />
					<attribute id="ParentTemplateId" type="FixedString" value="{parent_template}" />
					<attribute id="DisplayName" type="TranslatedString" handle="{name_handle}" version="1" />
					<attribute id="Icon" type="FixedString" value="{icon_name}" />
					<attribute id="Stats" type="FixedString" value="{name}" />
					<attribute id="Description" type="TranslatedString" handle="{desc_handle}" version="1" />
					<attribute id="ColorPreset" type="guid" value="{preset_uuid}" />
				</node>"#,
                template_uuid = dye.template_uuid,
                name = dye.name,
                parent_template = DYE_PARENT_TEMPLATE_ID,
                name_handle = dye.name_handle,
                icon_name = icon_name,
                desc_handle = dye.desc_handle,
                preset_uuid = dye.preset_uuid,
            )
        })
        .collect();

    // Container (pouch) entry with TreasureTable to spawn all dyes
    let container_entry = format!(
        r#"				<node id="GameObjects">
					<attribute id="MapKey" type="FixedString" value="{container_template_uuid}" />
					<attribute id="Name" type="LSString" value="{mod_name}_DyePouch" />
					<attribute id="LevelName" type="FixedString" value="" />
					<attribute id="Type" type="FixedString" value="item" />
					<attribute id="ParentTemplateId" type="FixedString" value="{pouch_parent}" />
					<attribute id="DisplayName" type="TranslatedString" handle="{container_name_handle}" version="1" />
					<attribute id="Icon" type="FixedString" value="Item_LOOT_GEN_Pouch_A" />
					<attribute id="Stats" type="FixedString" value="{mod_name}_DyePouch" />
					<attribute id="Description" type="TranslatedString" handle="{container_desc_handle}" version="1" />
					<attribute id="TreasureTable" type="FixedString" value="{mod_name}_Dyes" />
				</node>"#,
        pouch_parent = POUCH_PARENT_TEMPLATE_ID,
    );

    let mut all_entries = dye_entries;
    all_entries.push(container_entry);

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
        all_entries.join("\n")
    );

    let path = output_dir.join(format!(
        "Public/{}/RootTemplates/_merged.lsf",
        mod_name
    ));
    write_lsx_as_lsf(&content, path)
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
            let color_nodes = generate_color_nodes(&dye.colors);
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
        "Public/{}/Content/Assets/Characters/[PAK]_DYE_Colors/_merged.lsf",
        mod_name
    ));
    write_lsx_as_lsf(&content, path)
}

/// Write TextureAtlasInfo LSX (icon atlas metadata) for all dyes
fn write_texture_atlas_info_lsx(
    output_dir: &Path,
    mod_name: &str,
    dyes: &[GeneratedDyeEntry],
) -> std::io::Result<()> {
    let atlas_uuid = generate_uuid(UuidFormat::Standard);
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
        "Public/{}/GUI/{}_TextureAtlasInfo.lsf",
        mod_name, mod_name
    ));
    write_lsx_as_lsf(&content, path)
}

/// Write TextureBank LSX (UI texture references)
fn write_texture_bank_lsx(output_dir: &Path, mod_name: &str) -> std::io::Result<()> {
    let texture_uuid = generate_uuid(UuidFormat::Standard);

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
        "Public/{}/Content/UI/[PAK]_UI/_merged.lsf",
        mod_name
    ));
    write_lsx_as_lsf(&content, path)
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
            // Use the Cloth_Primary color as the icon color (magenta if missing/invalid)
            dye.colors.get("Cloth_Primary")
                .and_then(|hex| parse_hex_color(hex))
                .unwrap_or((255, 0, 255))
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
