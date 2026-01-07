//! Type definitions for dye mod data structures

use std::collections::HashMap;

/// Parsed dye entry from TXT mod files (ItemCombos.txt, Object.txt)
#[derive(Clone, Debug, Default)]
pub struct ParsedDyeEntry {
    pub name: String,
    pub preset_uuid: Option<String>,
    pub root_template_uuid: Option<String>,
}

/// Localization handle info parsed from RootTemplates
#[derive(Clone, Debug, Default)]
pub struct DyeLocalizationInfo {
    pub name: String,
    pub display_name_handle: Option<String>,
    pub description_handle: Option<String>,
}

/// A fully parsed dye entry from LSF/LSX files with all color data
#[derive(Clone, Debug, Default)]
pub struct ImportedDyeEntry {
    pub name: String,
    /// Display name from localization
    pub display_name: String,
    /// Description from localization
    pub description: String,
    /// The Resource ID from the LSF - this is the Preset UUID used in ItemCombos.txt
    pub preset_uuid: Option<String>,
    /// The RootTemplate UUID from Object.txt (used for the dye item)
    pub root_template_uuid: Option<String>,
    /// Color parameters: parameter name -> hex color (e.g., "Cloth_Primary" -> "FF0000")
    pub colors: HashMap<String, String>,
}
