//! Shared utilities for dye import/export functionality
//!
//! Core dye types and parsers are now in maclarian. This module provides
//! GUI-specific helpers and re-exports the library types for convenience.

// GUI-specific modules (stay here)
pub mod colors;
pub mod constants;
pub mod helpers;
pub mod selector;
pub mod styles;

// Re-export from local dyes module (moved from maclarian)
pub use crate::dyes::{
    // Registry
    ColorCategory, ColorDef, COLOR_REGISTRY, DEFAULT_HEX, COLOR_COUNT,
    colors_by_category, required_colors, find_color,
    // Types
    ParsedDyeEntry, DyeLocalizationInfo, ImportedDyeEntry,
    // Parsers
    parse_item_combos, parse_object_txt, parse_lsx_dye_presets,
    parse_root_templates_localization, parse_localization_xml,
    fvec3_to_hex, extract_xml_attribute,
    // Generators
    generate_color_nodes, generate_all_color_nodes,
    hex_to_fvec3, srgb_to_linear,
};

// Re-export meta parsing from maclarian formats
pub use maclarian::formats::{ModMetadata, parse_meta_lsx};

// Local exports
pub use colors::{collect_all_colors, collect_colors_skip_defaults, reset_colors_to_default, load_colors_from_map};
pub use helpers::{parse_hex_color, parse_hex_to_color, normalize_hex, copy_to_clipboard, pick_color_from_screen};
pub use selector::{nav_row, selector_container_green, selector_container_gray, empty_state_style, selector_label_style};
pub use styles::{input_style, button_style, secondary_button_style};
