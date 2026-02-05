//! Scaffold generators for equipment mod recipes.

use std::fs;
use std::path::Path;

use crate::workspace::project::ProjectManifest;

/// Generate a placeholder Armor.txt for an equipment mod.
///
/// Contains an example armor entry showing the stat format with common fields.
pub fn generate_equipment_armor_txt(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
    let mod_name = &manifest.project.folder;
    let content = format!(
        r#"new entry "{mod_name}_ExampleArmor"
type "Armor"
using "ARM_ScaleMail_Body"
data "RootTemplate" "00000000-0000-0000-0000-000000000000"
data "Rarity" "Uncommon"
data "ArmorClass" "14"
data "Boosts" "AC(1)"
data "Weight" "20.0"
data "ValueOverride" "500"
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write Armor.txt: {}", e))
}

/// Generate a placeholder TreasureTable.txt for an equipment mod.
///
/// Contains an example entry adding the item to the tutorial chest.
pub fn generate_equipment_treasure_table_txt(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
    let mod_name = &manifest.project.folder;
    let content = format!(
        r#"new treasuretable "TUT_Chest_Potions"
CanMerge 1
new subtable "1,1"
object category "I_{mod_name}_ExampleArmor",1,0,0,0,0,0,0,0
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write TreasureTable.txt: {}", e))
}
