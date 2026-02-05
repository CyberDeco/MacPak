//! Scaffold generators for spell mod recipes.

use std::fs;
use std::path::Path;

use crate::workspace::project::ProjectManifest;

/// Generate a placeholder spell data file based on the selected spell type.
///
/// Produces a type-appropriate example entry (Projectile, Target, Zone, etc.)
/// with common fields for that spell type pre-filled.
pub fn generate_spell_data_txt(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
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
