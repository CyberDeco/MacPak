//! Dyes module - color definitions, parsing, and generation for BG3 dye mods
//!
//! This module provides tools for working with dye color presets:
//! - Color registry with standard BG3 color parameter definitions
//! - Parsers for ItemCombos.txt, Object.txt, and LSX dye preset files
//! - Generators for creating LSX color nodes

pub mod generators;
pub mod parsers;
pub mod registry;
pub mod types;

pub use generators::{
    generate_all_color_nodes, generate_color_nodes, hex_to_fvec3, srgb_to_linear,
};
pub use parsers::{
    extract_xml_attribute, fvec3_to_hex, parse_item_combos, parse_localization_xml,
    parse_lsx_dye_presets, parse_object_txt, parse_root_templates_localization,
};
pub use registry::{COLOR_COUNT, COLOR_REGISTRY, ColorCategory, ColorDef, DEFAULT_HEX};
pub use registry::{colors_by_category, find_color, required_colors};
pub use types::{DyeLocalizationInfo, ImportedDyeEntry, ParsedDyeEntry};
