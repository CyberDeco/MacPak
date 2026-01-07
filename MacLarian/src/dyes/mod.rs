//! Dyes module - color definitions, parsing, and generation for BG3 dye mods
//!
//! This module provides tools for working with dye color presets:
//! - Color registry with standard BG3 color parameter definitions
//! - Parsers for ItemCombos.txt, Object.txt, and LSX dye preset files
//! - Generators for creating LSX color nodes

pub mod registry;
pub mod types;
pub mod parsers;
pub mod generators;

pub use registry::{ColorCategory, ColorDef, COLOR_REGISTRY, DEFAULT_HEX, COLOR_COUNT};
pub use registry::{colors_by_category, required_colors, find_color};
pub use types::{ParsedDyeEntry, DyeLocalizationInfo, ImportedDyeEntry};
pub use parsers::{
    parse_item_combos, parse_object_txt, parse_lsx_dye_presets,
    parse_root_templates_localization, parse_localization_xml,
    fvec3_to_hex, extract_xml_attribute,
};
pub use generators::{
    generate_color_nodes, generate_all_color_nodes,
    hex_to_fvec3, srgb_to_linear,
};
