//! Dyes tab state

use floem::prelude::*;
use std::collections::HashMap;

/// A single dye color entry with its category name and color value
#[derive(Clone)]
pub struct DyeColorEntry {
    pub name: &'static str,
    pub hex: RwSignal<String>,
}

impl DyeColorEntry {
    pub fn new(name: &'static str, default_hex: &str) -> Self {
        Self {
            name,
            hex: RwSignal::new(default_hex.to_string()),
        }
    }
}

/// A fully parsed dye entry from LSF/LSX files with all color data
#[derive(Clone, Debug, Default)]
pub struct ImportedDyeEntry {
    pub name: String,
    /// The Resource ID from the LSF - this is the Preset UUID used in ItemCombos.txt
    pub preset_uuid: Option<String>,
    /// Color parameters: parameter name -> hex color (e.g., "Cloth_Primary" -> "FF0000")
    pub colors: HashMap<String, String>,
}

/// A generated dye entry created in the current session
#[derive(Clone, Debug)]
pub struct GeneratedDyeEntry {
    pub name: String,
    /// The preset UUID for this dye (used in ItemCombos.txt and ColorPresets)
    pub preset_uuid: String,
    /// The root template UUID for this dye (used in Object.txt and RootTemplates)
    pub template_uuid: String,
    /// Localization handle for display name
    pub name_handle: String,
    /// Localization handle for description
    pub desc_handle: String,
    /// Color parameters: parameter name -> hex color
    pub colors: HashMap<String, String>,
}

/// Dyes tab state for custom dye color creation
#[derive(Clone)]
pub struct DyesState {
    // Required colors
    pub cloth_primary: DyeColorEntry,
    pub cloth_secondary: DyeColorEntry,
    pub cloth_tertiary: DyeColorEntry,
    pub color_01: DyeColorEntry,
    pub color_02: DyeColorEntry,
    pub color_03: DyeColorEntry,
    pub custom_1: DyeColorEntry,
    pub custom_2: DyeColorEntry,
    pub leather_primary: DyeColorEntry,
    pub leather_secondary: DyeColorEntry,
    pub leather_tertiary: DyeColorEntry,
    pub metal_primary: DyeColorEntry,
    pub metal_secondary: DyeColorEntry,
    pub metal_tertiary: DyeColorEntry,

    // Recommended colors
    pub accent_color: DyeColorEntry,
    pub glow_color: DyeColorEntry,
    pub glow_colour: DyeColorEntry,

    // Commonly used in mods
    pub added_color: DyeColorEntry,
    pub highlight_color: DyeColorEntry,
    pub base_color: DyeColorEntry,
    pub inner_color: DyeColorEntry,
    pub outer_color: DyeColorEntry,
    pub primary_color: DyeColorEntry,
    pub secondary_color: DyeColorEntry,
    pub tetriary_color: DyeColorEntry,
    pub primary: DyeColorEntry,
    pub secondary: DyeColorEntry,
    pub tertiary: DyeColorEntry,
    pub primary_color_underscore: DyeColorEntry,
    pub secondary_color_underscore: DyeColorEntry,
    pub tertiary_color_underscore: DyeColorEntry,

    // Status message
    pub status_message: RwSignal<String>,

    // Generate Dye settings
    pub individual_dye_name: RwSignal<String>,

    // Generated dyes (created in current session)
    pub generated_dyes: RwSignal<Vec<GeneratedDyeEntry>>,
    pub selected_generated_index: RwSignal<Option<usize>>,

    // Export settings
    pub mod_name: RwSignal<String>,

    // Import state (from txt files)
    pub imported_entries: RwSignal<Vec<(String, Option<String>, Option<String>)>>, // (name, preset_uuid, root_template_uuid)
    pub selected_import_index: RwSignal<Option<usize>>,

    // Import state (from LSF/LSX files with full color data)
    pub imported_lsf_entries: RwSignal<Vec<ImportedDyeEntry>>,
    pub selected_lsf_index: RwSignal<Option<usize>>,
    /// Path to the imported LSF file (for re-export in place)
    pub imported_lsf_path: RwSignal<Option<String>>,

    // Meta.lsx dialog visibility
    pub show_meta_dialog: RwSignal<bool>,
}

impl DyesState {
    pub fn new() -> Self {
        // Default to a neutral gray
        let default = "808080";

        Self {
            // Required
            cloth_primary: DyeColorEntry::new("Cloth_Primary", default),
            cloth_secondary: DyeColorEntry::new("Cloth_Secondary", default),
            cloth_tertiary: DyeColorEntry::new("Cloth_Tertiary", default),
            color_01: DyeColorEntry::new("Color_01", default),
            color_02: DyeColorEntry::new("Color_02", default),
            color_03: DyeColorEntry::new("Color_03", default),
            custom_1: DyeColorEntry::new("Custom_1", default),
            custom_2: DyeColorEntry::new("Custom_2", default),
            leather_primary: DyeColorEntry::new("Leather_Primary", default),
            leather_secondary: DyeColorEntry::new("Leather_Secondary", default),
            leather_tertiary: DyeColorEntry::new("Leather_Tertiary", default),
            metal_primary: DyeColorEntry::new("Metal_Primary", default),
            metal_secondary: DyeColorEntry::new("Metal_Secondary", default),
            metal_tertiary: DyeColorEntry::new("Metal_Tertiary", default),

            // Recommended
            accent_color: DyeColorEntry::new("Accent_Color", default),
            glow_color: DyeColorEntry::new("GlowColor", default),
            glow_colour: DyeColorEntry::new("GlowColour", default),

            // Commonly used in mods
            added_color: DyeColorEntry::new("AddedColor", default),
            highlight_color: DyeColorEntry::new("Highlight_Color", default),
            base_color: DyeColorEntry::new("BaseColor", default),
            inner_color: DyeColorEntry::new("InnerColor", default),
            outer_color: DyeColorEntry::new("OuterColor", default),
            primary_color: DyeColorEntry::new("PrimaryColor", default),
            secondary_color: DyeColorEntry::new("SecondaryColor", default),
            tetriary_color: DyeColorEntry::new("TetriaryColor", default),
            primary: DyeColorEntry::new("Primary", default),
            secondary: DyeColorEntry::new("Secondary", default),
            tertiary: DyeColorEntry::new("Tertiary", default),
            primary_color_underscore: DyeColorEntry::new("Primary_Color", default),
            secondary_color_underscore: DyeColorEntry::new("Secondary_Color", default),
            tertiary_color_underscore: DyeColorEntry::new("Tertiary_Color", default),

            status_message: RwSignal::new(String::new()),

            // Generate Dye settings
            individual_dye_name: RwSignal::new(String::new()),

            // Generated dyes
            generated_dyes: RwSignal::new(Vec::new()),
            selected_generated_index: RwSignal::new(None),

            // Export settings
            mod_name: RwSignal::new("MyDyeMod".to_string()),

            // Import state
            imported_entries: RwSignal::new(Vec::new()),
            selected_import_index: RwSignal::new(None),

            // LSF import state
            imported_lsf_entries: RwSignal::new(Vec::new()),
            selected_lsf_index: RwSignal::new(None),
            imported_lsf_path: RwSignal::new(None),

            // Meta.lsx dialog
            show_meta_dialog: RwSignal::new(false),
        }
    }
}

impl Default for DyesState {
    fn default() -> Self {
        Self::new()
    }
}
