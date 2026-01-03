//! Shared utilities for dye import/export functionality

pub mod colors;
pub mod constants;
pub mod generators;
pub mod helpers;
pub mod parsers;
pub mod selector;
pub mod styles;

pub use colors::{DEFAULT_COLOR, collect_all_colors, collect_colors_skip_defaults, reset_colors_to_default, load_colors_from_map};
pub use generators::{generate_color_nodes, generate_all_color_nodes};
pub use helpers::{parse_hex_color, parse_hex_to_color, normalize_hex, copy_to_clipboard, pick_color_from_screen};
pub use parsers::{ParsedDyeEntry, parse_item_combos, parse_object_txt, parse_lsx_dye_presets};
pub use selector::{nav_row, selector_container_green, selector_container_gray, empty_state_style, selector_label_style};
pub use styles::{input_style, button_style, secondary_button_style};
