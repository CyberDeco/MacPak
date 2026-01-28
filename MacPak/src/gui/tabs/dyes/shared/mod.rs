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
    COLOR_COUNT,
    COLOR_REGISTRY,
    // Registry
    ColorCategory,
    ColorDef,
    DEFAULT_HEX,
    DyeLocalizationInfo,
    ImportedDyeEntry,
    // Types
    ParsedDyeEntry,
    colors_by_category,
    extract_xml_attribute,
    find_color,
    fvec3_to_hex,
    generate_all_color_nodes,
    // Generators
    generate_color_nodes,
    hex_to_fvec3,
    // Parsers
    parse_item_combos,
    parse_localization_xml,
    parse_lsx_dye_presets,
    parse_object_txt,
    parse_root_templates_localization,
    required_colors,
    srgb_to_linear,
};

// Re-export meta parsing from maclarian formats
pub use maclarian::formats::{ModMetadata, parse_meta_lsx};

// Local exports
pub use colors::{
    collect_all_colors, collect_colors_skip_defaults, load_colors_from_map, reset_colors_to_default,
};
pub use helpers::{
    copy_to_clipboard, normalize_hex, parse_hex_color, parse_hex_to_color, pick_color_from_screen,
};
pub use selector::{
    empty_state_style, nav_row, selector_container_gray, selector_container_green,
    selector_label_style,
};
pub use styles::{button_style, input_style, secondary_button_style};
