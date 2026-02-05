//! Scaffold generators for hair mod recipes.

use std::fs;
use std::path::Path;

use crate::workspace::project::ProjectManifest;

/// Generate a placeholder CharacterCreationAppearanceVisuals.lsx for a hair mod.
///
/// Produces a template with example entries for the standard body type/race
/// combinations. Each entry maps a VisualResource (from the hair VisualBank)
/// to a specific race + body shape + body type slot in character creation.
pub fn generate_hair_cc_appearance_lsx(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
    let mod_name = &manifest.project.folder;

    // Common playable race UUIDs from vanilla BG3
    let races = [
        ("Human", "0eb594cb-8820-4be6-a58d-8be7a1a98fba"),
        ("Elf", "6c038dcb-7eb5-431d-84f8-cecfaf1c0c5a"),
        ("Half-Elf", "45f4ac10-3c89-4fb2-b37d-f973bb9110c0"),
        ("Drow", "4f5d1434-5175-4fa9-b7dc-ab24fba37929"),
        ("Dwarf", "0ab2874d-cfdc-405e-8571-b54f3a97d97c"),
        ("Halfling", "78cd3bcc-1c43-4a2a-aa80-c34322c16571"),
        ("Gnome", "f1b3f884-4029-4f0f-b158-1f9c0f0571a8"),
        ("Tiefling", "b6dccbed-30f3-424b-a181-c4540cf38197"),
        ("Dragonborn", "9c61a74a-20df-4119-89c5-d996956b6c66"),
        ("Half-Orc", "5c39a726-71c8-4748-ba8d-f768b3c11a91"),
        ("Githyanki", "bdf9b779-002c-4077-b377-8ea7c1faa795"),
    ];

    let mut entries = String::new();
    for (race_name, race_uuid) in &races {
        // BodyShape 0 = Normal, 1 = Strong
        // BodyType 0 = Masculine, 1 = Feminine
        for body_shape in 0..=1u8 {
            for body_type in 0..=1u8 {
                let shape_label = if body_shape == 0 { "Normal" } else { "Strong" };
                let type_label = if body_type == 0 {
                    "Masculine"
                } else {
                    "Feminine"
                };
                entries.push_str(&format!(
                    r#"
				<!-- {race_name} / {shape_label} / {type_label} -->
				<node id="CharacterCreationAppearanceVisual">
					<attribute id="BodyShape" type="uint8" value="{body_shape}" />
					<attribute id="BodyType" type="uint8" value="{body_type}" />
					<attribute id="RaceUUID" type="guid" value="{race_uuid}" />
					<attribute id="SlotName" type="FixedString" value="Hair" />
					<attribute id="UUID" type="guid" value="00000000-0000-0000-0000-000000000000" />
					<attribute id="VisualResource" type="guid" value="00000000-0000-0000-0000-000000000000" />
				</node>"#
                ));
            }
        }
    }

    let content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<save>
	<version major="4" minor="0" revision="6" build="5" />
	<region id="CharacterCreationAppearanceVisuals">
		<node id="CharacterCreationAppearanceVisuals">
			<children>
				<!--
					{mod_name} — Hair Character Creation Entries

					Each node maps your hair visual to a specific race/body combination.
					Replace the placeholder UUIDs:
					  UUID — generate a unique UUID for each entry
					  VisualResource — the Resource ID from your _merged.lsf VisualBank

					For autosnapping hairs (NeedsSkeletonRemap=True), all entries can
					share the same VisualResource. For per-race meshes, use a different
					VisualResource for each race pointing to its specific mesh.

					Delete any race/body combos you don't want to support.
				-->
{entries}
			</children>
		</node>
	</region>
</save>
"#
    );

    fs::write(dest, content).map_err(|e| {
        format!(
            "Failed to write CharacterCreationAppearanceVisuals.lsx: {}",
            e
        )
    })
}
