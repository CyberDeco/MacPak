//! Scaffold generators for dye mod recipes.

use std::fs;
use std::path::Path;

use crate::workbench::project::ProjectManifest;

/// Generate a placeholder Object.txt for a dye mod.
///
/// Contains an example dye entry and pouch entry showing the expected format.
/// The Dye Lab will overwrite this with real data.
pub fn generate_dye_object_txt(dest: &Path, manifest: &ProjectManifest) -> Result<(), String> {
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
pub fn generate_dye_item_combos_txt(dest: &Path) -> Result<(), String> {
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
pub fn generate_dye_treasure_table_txt(
    dest: &Path,
    manifest: &ProjectManifest,
) -> Result<(), String> {
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
