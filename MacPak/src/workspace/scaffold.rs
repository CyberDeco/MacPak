//! Project scaffolding from recipe templates
//!
//! Creates the directory structure and generated files for a new mod project.

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
            Some("meta_lsx") => {
                generate_meta_lsx_file(&full_path, manifest)?;
            }
            Some("localization_xml") => {
                generate_localization_xml(&full_path, manifest)?;
            }
            Some("dye_object_txt") => {
                generate_dye_object_txt(&full_path, manifest)?;
            }
            Some("dye_item_combos_txt") => {
                generate_dye_item_combos_txt(&full_path)?;
            }
            Some("dye_treasure_table_txt") => {
                generate_dye_treasure_table_txt(&full_path, manifest)?;
            }
            Some("spell_data_txt") => {
                generate_spell_data_txt(&full_path, manifest)?;
            }
            Some("equipment_armor_txt") => {
                generate_equipment_armor_txt(&full_path, manifest)?;
            }
            Some("equipment_treasure_table_txt") => {
                generate_equipment_treasure_table_txt(&full_path, manifest)?;
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

/// Generate a placeholder Object.txt for a dye mod.
///
/// Contains an example dye entry and pouch entry showing the expected format.
/// The Dye Lab will overwrite this with real data.
fn generate_dye_object_txt(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    let mod_name = &manifest.project.folder;
    let content = format!(
        r#"new entry "ExampleDye"
type "Object"
using "_Dyes"
data "RootTemplate" "00000000-0000-0000-0000-000000000000"

new entry "{mod_name}_DyePouch"
type "Object"
using "OBJ_Pouch"
data "RootTemplate" "00000000-0000-0000-0000-000000000000"
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write Object.txt: {}", e))
}

/// Generate a placeholder ItemCombos.txt for a dye mod.
///
/// Contains an example dye combination entry showing the expected format.
/// The Dye Lab will overwrite this with real data.
fn generate_dye_item_combos_txt(dest: &Path) -> Result<(), String> {
    let content = r#"new ItemCombination "ExampleDye"
data "Type 1" "Object"
data "Object 1" "ExampleDye"
data "Transform 1" "None"
data "Type 2" "Category"
data "Object 2" "DyableArmor"
data "Transform 2" "Dye"
data "DyeColorPresetResource" "00000000-0000-0000-0000-000000000000"

new ItemCombinationResult "ExampleDye_1"
data "ResultAmount 1" "1"
"#;

    fs::write(dest, content).map_err(|e| format!("Failed to write ItemCombos.txt: {}", e))
}

/// Generate a placeholder TreasureTable.txt for a dye mod.
///
/// Contains an example treasure table and pouch contents table.
/// The Dye Lab will overwrite this with real data and vendor entries.
fn generate_dye_treasure_table_txt(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    let mod_name = &manifest.project.folder;
    let content = format!(
        r#"new treasuretable "{mod_name}_Dyes"
new subtable "1,1"
object category "I_ExampleDye",1,0,0,0,0,0,0,0

new treasuretable "{mod_name}_DyePouch_Contents"
CanMerge 1
new subtable "1,1"
object category "I_{mod_name}_DyePouch",1,0,0,0,0,0,0,0
"#
    );

    fs::write(dest, content).map_err(|e| format!("Failed to write TreasureTable.txt: {}", e))
}

/// Generate a placeholder spell data file based on the selected spell type.
///
/// Produces a type-appropriate example entry (Projectile, Target, Zone, etc.)
/// with common fields for that spell type pre-filled.
fn generate_spell_data_txt(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    let mod_name = &manifest.project.folder;
    let spell_type = manifest
        .variables
        .get("spell_type")
        .map(|s| s.as_str())
        .unwrap_or("Projectile");
    let spell_school = manifest
        .variables
        .get("spell_school")
        .map(|s| s.as_str())
        .unwrap_or("Evocation");

    let content = match spell_type {
        "Target" => format!(
            r#"new entry "Target_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "Target"
using "Target_MainHandAttack"
data "SpellSchool" "{spell_school}"
data "Level" "1"
data "Cooldown" "OncePerTurn"
data "UseCosts" "ActionPoint:1;SpellSlot:1:1:1"
data "SpellRoll" "not SavingThrow(Ability.Wisdom, SourceSpellDC())"
data "SpellSuccess" "ApplyStatus(CHARMED,100,10)"
data "TargetConditions" "not Self() and not Dead() and Character()"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
data "SpellFlags" "HasSomaticComponent;HasVerbalComponent"
"#
        ),
        "Zone" => format!(
            r#"new entry "Zone_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "Zone"
using "Zone_Fear"
data "SpellSchool" "{spell_school}"
data "Level" "1"
data "UseCosts" "ActionPoint:1;SpellSlot:1:1:1"
data "SpellRoll" "not SavingThrow(Ability.Constitution, SourceSpellDC())"
data "SpellSuccess" "DealDamage(2d8,Thunder);Force(15,OriginToTarget)"
data "TargetConditions" "not Self() and not Dead()"
data "Shape" "Cone"
data "Range" "18"
data "Base" "5"
data "Angle" "100"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
data "SpellFlags" "HasSomaticComponent;HasVerbalComponent"
"#
        ),
        "Shout" => format!(
            r#"new entry "Shout_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "Shout"
using "Shout_Disengage"
data "SpellSchool" "{spell_school}"
data "Level" "1"
data "Cooldown" "OncePerShortRest"
data "UseCosts" "BonusActionPoint:1"
data "SpellProperties" "ApplyStatus(SELF,YOURMOD_BUFF,100,10)"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
data "SpellFlags" "HasSomaticComponent;HasVerbalComponent"
"#
        ),
        "Throw" => format!(
            r#"new entry "Throw_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "Throw"
using "Throw_FrenziedThrow"
data "SpellSchool" "{spell_school}"
data "Level" "0"
data "UseCosts" "ActionPoint:1"
data "TargetRadius" "ThrownObjectRange"
data "AreaRadius" "1"
data "SpellSuccess" "DealDamage(1d4,Bludgeoning)"
data "ThrowableTargetConditions" "CanThrowWeight() and not Grounded()"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
"#
        ),
        "Rush" => format!(
            r#"new entry "Rush_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "Rush"
using "Rush_SpringAttack"
data "SpellSchool" "{spell_school}"
data "Level" "1"
data "UseCosts" "ActionPoint:1;SpellSlot:1:1:1"
data "MovementSpeed" "60000"
data "SpellRoll" "not SavingThrow(Ability.Dexterity, SourceSpellDC())"
data "SpellSuccess" "DealDamage(3d8,Thunder,Magical)"
data "SpellFail" "DealDamage(1d8,Thunder,Magical)"
data "DamageType" "Thunder"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
"#
        ),
        "Wall" => format!(
            r#"new entry "Wall_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "Wall"
using "Wall_WallOfFire_5"
data "SpellSchool" "{spell_school}"
data "Level" "4"
data "UseCosts" "ActionPoint:1;SpellSlot:1:4:4"
data "MaxDistance" "18"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
data "SpellFlags" "HasSomaticComponent;HasVerbalComponent;IsConcentration"
"#
        ),
        "ProjectileStrike" => format!(
            r#"new entry "ProjectileStrike_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "ProjectileStrike"
using "ProjectileStrike_TUT_UpperDeck_Bombardment"
data "SpellSchool" "{spell_school}"
data "Level" "3"
data "UseCosts" "ActionPoint:1;SpellSlot:1:3:3"
data "AreaRadius" "5"
data "ProjectileCount" "3"
data "SpellRoll" "not SavingThrow(Ability.Dexterity, SourceSpellDC())"
data "SpellSuccess" "DealDamage(2d6,Fire)"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
"#
        ),
        "Teleportation" => format!(
            r#"new entry "Teleportation_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "Teleportation"
using "Teleportation_ArcaneGate"
data "SpellSchool" "{spell_school}"
data "Level" "4"
data "Cooldown" "OncePerShortRest"
data "UseCosts" "ActionPoint:1;SpellSlot:1:4:4"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
data "SpellFlags" "HasSomaticComponent;HasVerbalComponent"
"#
        ),
        // Projectile (default)
        _ => format!(
            r#"new entry "Projectile_{mod_name}_ExampleSpell"
type "SpellData"
data "SpellType" "Projectile"
using "Projectile_FireBolt"
data "SpellSchool" "{spell_school}"
data "Level" "0"
data "UseCosts" "ActionPoint:1"
data "SpellRoll" "Attack(AttackType.RangedSpellAttack)"
data "SpellSuccess" "DealDamage(1d10,Fire)"
data "TooltipDamageList" "DealDamage(1d10,Fire)"
data "TargetRadius" "18"
data "Icon" "Spell_{spell_school}_Placeholder"
data "DisplayName" "<name-handle>;1"
data "Description" "<desc-handle>;1"
data "SpellFlags" "HasSomaticComponent;HasVerbalComponent"
"#
        ),
    };

    fs::write(dest, content).map_err(|e| format!("Failed to write spell data: {}", e))
}

/// Generate a placeholder Armor.txt for an equipment mod.
///
/// Contains an example armor entry showing the stat format with common fields.
fn generate_equipment_armor_txt(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
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
fn generate_equipment_treasure_table_txt(
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
