//! Dyes tab state

use floem::prelude::*;
use std::collections::HashMap;

// Import from maclarian dyes module
use maclarian::dyes::{ColorCategory, COLOR_REGISTRY, DEFAULT_HEX};
// Re-export ImportedDyeEntry from maclarian for public use
pub use maclarian::dyes::ImportedDyeEntry;

/// Vendor table definition for dye distribution
#[derive(Clone, Debug)]
pub struct VendorDef {
    pub id: &'static str,
    pub display_name: &'static str,
    pub location: &'static str,
    pub always_enabled: bool,
}

/// All available vendor tables for dye distribution
pub const VENDOR_DEFS: &[VendorDef] = &[
    // Always enabled
    VendorDef { id: "TUT_Chest_Potions", display_name: "Tutorial Chest", location: "Nautiloid", always_enabled: true }, // 0

    // Act 1 - Wilderness / Underdark [0, 1, 2, 7, 10, 14, 15, 16]
    VendorDef { id: "DEN_Entrance_Trade", display_name: "Arron", location: "Emerald Grove", always_enabled: false }, // 1
    VendorDef { id: "DEN_Weaponsmith_Trade", display_name: "Dammon", location: "Emerald Grove", always_enabled: false }, // 2
    VendorDef { id: "DEN_Trader_Food", display_name: "Okta", location: "Emerald Grove", always_enabled: false }, // 3, Commented out of dialog
    VendorDef { id: "DEN_Ethel", display_name: "Auntie Ethel", location: "Emerald Grove", always_enabled: false }, // 4, Commented out of dialog
    VendorDef { id: "DEN_Thieflings_Pickpocket_Trade", display_name: "Mattis", location: "Emerald Grove", always_enabled: false }, // 5, Commented out of dialog
    VendorDef { id: "GOB_Festivities_Trader", display_name: "Grat the Trader", location: "Goblin Camp", always_enabled: false }, // 6
    VendorDef { id: "GOB_Quartermaster", display_name: "Roah Moonglow", location: "Goblin Camp", always_enabled: false }, // 7
    VendorDef { id: "PLA_ZhentarimTrader_Common", display_name: "Brem", location: "Zhentarim Hideout", always_enabled: false }, // 8, Commented out of dialog
    VendorDef { id: "PLA_Tollhouse_SuppliesTrader", display_name: "Cyrel", location: "Tollhouse", always_enabled: false }, // 9, Commented out of dialog
    VendorDef { id: "UND_MycoVillage_AlchemistDwarf_Trade", display_name: "Derryth Bonecloak", location: "Myconid Colony", always_enabled: false }, // 10
    VendorDef { id: "UND_MushroomHunter", display_name: "Baelen Bonecloak", location: "Underdark", always_enabled: false }, // 11, Commented out of dialog
    VendorDef { id: "UND_SocietyOfBrilliance_Hobgoblin", display_name: "Blurg", location: "Myconid Colony", always_enabled: false }, // 12, Commented out of dialog
    VendorDef { id: "UND_Duergar_Generic", display_name: "Elder Brithvar", location: "Grymforge", always_enabled: false }, // 13, Commented out of dialog
    VendorDef { id: "UND_KC_Trader_Weapons", display_name: "Corsair Greymon", location: "Grymforge", always_enabled: false }, // 14
    VendorDef { id: "UND_KC_Trader_Utility", display_name: "Stonemason Kith", location: "Grymforge", always_enabled: false }, // 15
    VendorDef { id: "DEN_Volo_Trade", display_name: "Volo", location: "", always_enabled: false }, // 16

    // Act 2 - Creche / Shadow-Cursed Lands [17, 18, 19, 20, 21]
    VendorDef { id: "CRE_GithQuartermistress_Trade", display_name: "A'jak'nir Jeera", location: "Creche Y'llek", always_enabled: false }, // 17
    VendorDef { id: "CRE_Expeditioner_Trade", display_name: "Lady Esther", location: "Rosymorn Monastery", always_enabled: false }, // 18
    VendorDef { id: "HAV_HarperQuarterMaster_Magic_Trade", display_name: "Quartermaster Talli", location: "Last Light Inn", always_enabled: false }, // 19
    VendorDef { id: "MOO_BugBearvendor_Trade", display_name: "Lann Tarv", location: "Moonrise Towers", always_enabled: false }, // 20
    VendorDef { id: "MOO_InfernalTrader_Trade", display_name: "Araj Oblodra", location: "Moonrise Towers", always_enabled: false }, // 21
    VendorDef { id: "TWN_Hospital_CorpseTender", display_name: "Sister Lidwin", location: "House of Healing", always_enabled: false }, // 22, Commented out of dialog
    VendorDef { id: "SHA_MerregonTrader", display_name: "Hoarding Merregon", location: "Gauntlet of Shar", always_enabled: false }, // 23, Commented out of dialog

    // Act 3 - Wyrm's Crossing / Rivington [24, 25, 26, 29, 33, 34, 37, 38, 40]
    VendorDef { id: "WYR_OrinsImpersonation_Smith", display_name: "The Rivington General", location: "Rivington", always_enabled: false }, // 24
    VendorDef { id: "WYR_Danthelon_Trader", display_name: "Danthelon's Dancing Axe", location: "Rivington", always_enabled: false }, // 25
    VendorDef { id: "WYR_AlchemyTrader", display_name: "Stylin' Horst", location: "Wyrm's Crossing", always_enabled: false }, // 26
    VendorDef { id: "WYR_Bridge_Trader_Art", display_name: "Roberon Silt", location: "Wyrm's Crossing", always_enabled: false }, // 27, Commented out of dialog
    VendorDef { id: "WYR_Bridge_Trader_Supplies", display_name: "Velson Oakes", location: "Wyrm's Crossing", always_enabled: false }, // 28, Commented out of dialog
    VendorDef { id: "WYR_Bridge_Trader_Tools", display_name: "Glynda Oltower", location: "Wyrm's Crossing", always_enabled: false }, // 29
    VendorDef { id: "WYR_Flophouse_Cook", display_name: "Queelia Arvis", location: "Wyrm's Crossing", always_enabled: false }, // 30, Commented out of dialog
    VendorDef { id: "WYR_Ironhand_Merchant", display_name: "Bumpnagel", location: "Ironhand Gnomes", always_enabled: false }, // 31, Commented out of dialog
    VendorDef { id: "WYR_SharessCaress_Bartender_Trade", display_name: "Hoots Hooligan", location: "Sharess' Caress", always_enabled: false }, // 32, Commented out of dialog
    
    // Act 3 - Baldur's Gate / Lower City
    VendorDef { id: "LOW_Figaro_Trade", display_name: "Facemaker's Boutique", location: "Lower City", always_enabled: false }, // 33
    VendorDef { id: "LOW_DevilsFee_Diabolist_Trade", display_name: "Devil's Fee", location: "Lower City", always_enabled: false }, // 34
    VendorDef { id: "LOW_MysticCarrion_Trade", display_name: "Mystic Carrion", location: "Philgrave's Mansion", always_enabled: false }, // 35, Commented out of dialog
    VendorDef { id: "LOW_Guildhall_FetchersBrat_Trade", display_name: "Sticky Dondo", location: "Guildhall", always_enabled: false }, // 36, Commented out of dialog
    VendorDef { id: "LOW_SteepsTrader_Weapons", display_name: "Fytz the Firecracker", location: "Lower City", always_enabled: false }, // 37
    VendorDef { id: "LOW_SteepsTrader_Armor", display_name: "Gloomy Fentonson", location: "Lower City", always_enabled: false }, // 38
    VendorDef { id: "LOW_SteepsTrader_BooksAndScrolls", display_name: "Nansi Gretta", location: "Lower City", always_enabled: false }, // 39, Commented out of dialog
    VendorDef { id: "LOW_SteepsTrader_Consumables", display_name: "Beehive General", location: "Lower City", always_enabled: false }, // 40
    VendorDef { id: "LOW_MusicTrader", display_name: "Thomas C. Quirkilious", location: "Lower City", always_enabled: false }, // 41, Commented out of dialog
    VendorDef { id: "LOW_JewelTrader", display_name: "Omotola", location: "Lower City", always_enabled: false }, // 42, Commented out of dialog
    VendorDef { id: "LOW_MurderTribunal_Merchant", display_name: "Echo of Abazigal", location: "Murder Tribunal", always_enabled: false }, // 43, Commented out of dialog
    VendorDef { id: "LOW_GortashParent_Trade", display_name: "Flymm Family", location: "Lower City", always_enabled: false }, // 44, Commented out of dialog
];

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

// ImportedDyeEntry is now imported from maclarian::dyes

/// A generated dye entry created in the current session
#[derive(Clone, Debug)]
pub struct GeneratedDyeEntry {
    pub name: String,
    /// Display name shown in-game (for localization)
    pub display_name: String,
    /// Description shown in-game (for localization)
    pub description: String,
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
    /// All color entries, indexed by position in COLOR_REGISTRY
    colors: Vec<DyeColorEntry>,

    // Status message
    pub status_message: RwSignal<String>,

    // Generate Dye settings
    pub individual_dye_name: RwSignal<String>,
    pub individual_display_name: RwSignal<String>,
    pub individual_description: RwSignal<String>,

    // Generated dyes (created in current session)
    pub generated_dyes: RwSignal<Vec<GeneratedDyeEntry>>,
    pub selected_generated_index: RwSignal<Option<usize>>,

    // Export settings
    pub mod_name: RwSignal<String>,
    pub mod_author: RwSignal<String>,
    pub mod_description: RwSignal<String>,
    pub mod_uuid: RwSignal<String>,
    pub mod_version_major: RwSignal<u32>,
    pub mod_version_minor: RwSignal<u32>,
    pub mod_version_patch: RwSignal<u32>,
    pub mod_version_build: RwSignal<u32>,

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

    // Vendor selection for export (indices into VENDOR_DEFS that are enabled)
    pub selected_vendors: RwSignal<Vec<bool>>,
}

impl DyesState {
    pub fn new() -> Self {
        // Initialize all colors from the registry
        let colors: Vec<DyeColorEntry> = COLOR_REGISTRY
            .iter()
            .map(|def| DyeColorEntry::new(def.name, DEFAULT_HEX))
            .collect();

        Self {
            colors,

            status_message: RwSignal::new(String::new()),

            // Generate Dye settings
            individual_dye_name: RwSignal::new(String::new()),
            individual_display_name: RwSignal::new(String::new()),
            individual_description: RwSignal::new(String::new()),

            // Generated dyes
            generated_dyes: RwSignal::new(Vec::new()),
            selected_generated_index: RwSignal::new(None),

            // Export settings
            mod_name: RwSignal::new(String::new()),
            mod_author: RwSignal::new(String::new()),
            mod_description: RwSignal::new(String::new()),
            mod_uuid: RwSignal::new(String::new()),
            mod_version_major: RwSignal::new(1),
            mod_version_minor: RwSignal::new(0),
            mod_version_patch: RwSignal::new(0),
            mod_version_build: RwSignal::new(0),

            // Import state
            imported_entries: RwSignal::new(Vec::new()),
            selected_import_index: RwSignal::new(None),

            // LSF import state
            imported_lsf_entries: RwSignal::new(Vec::new()),
            selected_lsf_index: RwSignal::new(None),
            imported_lsf_path: RwSignal::new(None),

            // Meta.lsx dialog
            show_meta_dialog: RwSignal::new(false),

            // Vendor selection - default to none (except always_enabled ones)
            selected_vendors: RwSignal::new(
                VENDOR_DEFS.iter().map(|v| v.always_enabled).collect()
            ),
        }
    }

    /// Get all color entries
    pub fn all_colors(&self) -> &[DyeColorEntry] {
        &self.colors
    }

    /// Get color entries for a specific category
    pub fn colors_by_category(&self, category: ColorCategory) -> Vec<&DyeColorEntry> {
        self.colors
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| {
                if COLOR_REGISTRY[i].category == category {
                    Some(entry)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get a color entry by name
    pub fn color(&self, name: &str) -> Option<&DyeColorEntry> {
        self.colors.iter().find(|c| c.name == name)
    }

    /// Get a mutable reference to color hex signal by name
    pub fn color_hex(&self, name: &str) -> Option<RwSignal<String>> {
        self.colors.iter().find(|c| c.name == name).map(|c| c.hex)
    }

    /// Check if a color is at default value
    pub fn is_color_default(&self, name: &str) -> bool {
        self.color(name)
            .map(|c| c.hex.get().to_lowercase() == DEFAULT_HEX)
            .unwrap_or(true)
    }
}

impl Default for DyesState {
    fn default() -> Self {
        Self::new()
    }
}
