//! Scaffold generators for class/subclass mod recipes.

use std::fs;
use std::path::Path;

use crate::workbench::project::ProjectManifest;

/// Generate a placeholder ClassDescriptions LSX for a class mod.
///
/// Produces a template with the required attributes for defining a new class
/// (base HP, hit die, primary ability, spellcasting, progression table reference).
pub fn generate_class_descriptions_lsx(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
    let class_name = manifest
        .variables
        .get("class_name")
        .cloned()
        .unwrap_or_else(|| "MyClass".to_string());

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<save>
	<version major="4" minor="0" revision="9" build="330" />
	<region id="ClassDescriptions">
		<node id="root">
			<children>
				<!--
					{class_name} — Class Description

					Replace placeholder UUIDs and adjust values:
					  UUID — unique identifier for this class
					  ProgressionTableUUID — must match TableUUID in your Progressions file
					  BaseHp / HpPerLevel — hit points (e.g., Fighter: 10/6, Wizard: 6/4)
					  PrimaryAbility — 0=None, 1=Str, 2=Dex, 3=Con, 4=Int, 5=Wis, 6=Cha
					  SpellCastingAbility — same numbering (set to 0 if non-caster)
					  LearningStrategy — 1=AddKnown, 2=Prepared
					  ClassEquipment — references an Equipment.txt entry for starting gear
				-->
				<node id="ClassDescription">
					<attribute id="BaseHp" type="int32" value="10" />
					<attribute id="CanLearnSpells" type="bool" value="false" />
					<attribute id="ClassEquipment" type="FixedString" value="EQP_CC_{class_name}" />
					<attribute id="Description" type="TranslatedString" handle="h00000000g0000g0000g0000g000000000001" version="1" />
					<attribute id="DisplayName" type="TranslatedString" handle="h00000000g0000g0000g0000g000000000000" version="1" />
					<attribute id="HpPerLevel" type="int32" value="6" />
					<attribute id="LearningStrategy" type="uint8" value="1" />
					<attribute id="MustPrepareSpells" type="bool" value="false" />
					<attribute id="Name" type="FixedString" value="{class_name}" />
					<attribute id="PrimaryAbility" type="uint8" value="1" />
					<attribute id="ProgressionTableUUID" type="guid" value="00000000-0000-0000-0000-000000000000" />
					<attribute id="SoundClassType" type="FixedString" value="Fighter" />
					<attribute id="SpellCastingAbility" type="uint8" value="0" />
					<attribute id="UUID" type="guid" value="00000000-0000-0000-0000-000000000000" />
					<children>
						<node id="Tags">
							<attribute id="Object" type="guid" value="00000000-0000-0000-0000-000000000000" />
						</node>
					</children>
				</node>
			</children>
		</node>
	</region>
</save>
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write ClassDescriptions.lsx: {}", e))
}

/// Generate a placeholder Progressions LSX for a class mod.
///
/// Produces a template with progression entries for levels 1-12.
/// Each entry defines what the character gains at that level.
pub fn generate_class_progressions_lsx(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
    let class_name = manifest
        .variables
        .get("class_name")
        .cloned()
        .unwrap_or_else(|| "MyClass".to_string());

    let mut level_entries = String::new();
    for level in 1..=12u8 {
        // ProgressionType: 0 = class base, 1 = subclass
        level_entries.push_str(&format!(
            r#"
				<!-- Level {level} -->
				<node id="Progression">
					<attribute id="Level" type="uint8" value="{level}" />
					<attribute id="Name" type="LSString" value="{class_name}" />
					<attribute id="ProgressionType" type="uint8" value="0" />
					<attribute id="TableUUID" type="guid" value="00000000-0000-0000-0000-000000000000" />
					<attribute id="UUID" type="guid" value="00000000-0000-0000-0000-000000000000" />
				</node>"#
        ));
    }

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<save>
	<version major="4" minor="0" revision="0" build="49" />
	<region id="Progressions">
		<node id="root">
			<children>
				<!--
					{class_name} — Progression Table

					One Progression node per level (1-12). Replace placeholder UUIDs:
					  TableUUID — must match ProgressionTableUUID in ClassDescriptions
					             (use the SAME TableUUID for all levels of this class)
					  UUID — unique per level entry

					ProgressionType: 0 = base class, 1 = subclass

					Add children to grant features at each level:
					  <attribute id="PassivesAdded" type="LSString" value="Passive1,Passive2" />
					  <attribute id="Boosts" type="LSString" value="ActionResource(SpellSlot,2,1)" />
					  <attribute id="Selectors" type="LSString" value="SelectPassives(uuid,1,ClassName)" />
				-->
{level_entries}
			</children>
		</node>
	</region>
</save>
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write Progressions.lsx: {}", e))
}
