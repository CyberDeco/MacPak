//! Scaffold generators for equipment mod recipes.
//!
//! Branches on the `item_type` variable (Armor, Weapon, Accessory) to generate
//! type-appropriate stat entries and treasure tables.

use std::fs;
use std::path::Path;

use crate::workspace::project::ProjectManifest;

/// Read the item_type variable from the manifest, defaulting to "Armor".
fn item_type(manifest: &ProjectManifest) -> &str {
    manifest
        .variables
        .get("item_type")
        .map(|s| s.as_str())
        .unwrap_or("Armor")
}

/// Generate Armor.txt for an equipment mod.
///
/// - Armor: body armor example with ArmorClass, AC boost, weight
/// - Accessory: amulet example with ability boost (no ArmorClass)
/// - Weapon: empty file (primary stats go in Weapon.txt)
pub fn generate_equipment_armor_txt(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
    let mod_name = &manifest.project.folder;

    let content = match item_type(manifest) {
        "Armor" => format!(
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
        ),
        "Accessory" => format!(
            r#"new entry "{mod_name}_ExampleAccessory"
type "Armor"
using "ARM_Amulet"
data "RootTemplate" "00000000-0000-0000-0000-000000000000"
data "Rarity" "Uncommon"
data "Boosts" "Ability(Constitution,2)"
data "ValueOverride" "300"
"#
        ),
        // Weapon — Armor.txt not primary, write empty file
        _ => String::new(),
    };

    fs::write(dest, content).map_err(|e| format!("Failed to write Armor.txt: {}", e))
}

/// Generate Weapon.txt for an equipment mod.
///
/// - Weapon: longsword example with Damage, Damage Type, DefaultBoosts
/// - Armor/Accessory: empty file (primary stats go in Armor.txt)
pub fn generate_equipment_weapon_txt(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
    let mod_name = &manifest.project.folder;

    let content = match item_type(manifest) {
        "Weapon" => format!(
            r#"new entry "{mod_name}_ExampleWeapon"
type "Weapon"
using "WPN_Longsword_1"
data "RootTemplate" "00000000-0000-0000-0000-000000000000"
data "Rarity" "Uncommon"
data "Damage" "1d8"
data "Damage Type" "Slashing"
data "DefaultBoosts" "WeaponProperty(Magical)"
data "Weight" "1.35"
data "ValueOverride" "500"
"#
        ),
        // Armor/Accessory — Weapon.txt not primary, write empty file
        _ => String::new(),
    };

    fs::write(dest, content).map_err(|e| format!("Failed to write Weapon.txt: {}", e))
}

/// Generate TreasureTable.txt for an equipment mod.
///
/// References the correct example item name based on item type.
pub fn generate_equipment_treasure_table_txt(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
    let mod_name = &manifest.project.folder;

    let item_name = match item_type(manifest) {
        "Weapon" => format!("{mod_name}_ExampleWeapon"),
        "Accessory" => format!("{mod_name}_ExampleAccessory"),
        _ => format!("{mod_name}_ExampleArmor"),
    };

    let content = format!(
        r#"new treasuretable "TUT_Chest_Potions"
CanMerge 1
new subtable "1,1"
object category "I_{item_name}",1,0,0,0,0,0,0,0
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write TreasureTable.txt: {}", e))
}
