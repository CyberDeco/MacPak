//! Color utilities for the Dyes tab
//!
//! Provides shared functions for collecting and resetting color picker values.

use std::collections::HashMap;
use floem::prelude::*;
use crate::gui::state::DyesState;

/// Default color value (gray)
pub const DEFAULT_COLOR: &str = "808080";

/// Collect all color values from state into a HashMap.
/// Always includes all 31 colors regardless of their value.
pub fn collect_all_colors(state: &DyesState) -> HashMap<String, String> {
    let mut colors = HashMap::new();

    // Required colors
    colors.insert("Cloth_Primary".to_string(), state.cloth_primary.hex.get());
    colors.insert("Cloth_Secondary".to_string(), state.cloth_secondary.hex.get());
    colors.insert("Cloth_Tertiary".to_string(), state.cloth_tertiary.hex.get());
    colors.insert("Leather_Primary".to_string(), state.leather_primary.hex.get());
    colors.insert("Leather_Secondary".to_string(), state.leather_secondary.hex.get());
    colors.insert("Leather_Tertiary".to_string(), state.leather_tertiary.hex.get());
    colors.insert("Metal_Primary".to_string(), state.metal_primary.hex.get());
    colors.insert("Metal_Secondary".to_string(), state.metal_secondary.hex.get());
    colors.insert("Metal_Tertiary".to_string(), state.metal_tertiary.hex.get());
    colors.insert("Color_01".to_string(), state.color_01.hex.get());
    colors.insert("Color_02".to_string(), state.color_02.hex.get());
    colors.insert("Color_03".to_string(), state.color_03.hex.get());
    colors.insert("Custom_1".to_string(), state.custom_1.hex.get());
    colors.insert("Custom_2".to_string(), state.custom_2.hex.get());

    // Recommended colors
    colors.insert("Accent_Color".to_string(), state.accent_color.hex.get());
    colors.insert("GlowColor".to_string(), state.glow_color.hex.get());
    colors.insert("GlowColour".to_string(), state.glow_colour.hex.get());

    // Common colors
    colors.insert("AddedColor".to_string(), state.added_color.hex.get());
    colors.insert("Highlight_Color".to_string(), state.highlight_color.hex.get());
    colors.insert("BaseColor".to_string(), state.base_color.hex.get());
    colors.insert("InnerColor".to_string(), state.inner_color.hex.get());
    colors.insert("OuterColor".to_string(), state.outer_color.hex.get());
    colors.insert("PrimaryColor".to_string(), state.primary_color.hex.get());
    colors.insert("SecondaryColor".to_string(), state.secondary_color.hex.get());
    colors.insert("TetriaryColor".to_string(), state.tetriary_color.hex.get());
    colors.insert("Primary".to_string(), state.primary.hex.get());
    colors.insert("Secondary".to_string(), state.secondary.hex.get());
    colors.insert("Tertiary".to_string(), state.tertiary.hex.get());
    colors.insert("Primary_Color".to_string(), state.primary_color_underscore.hex.get());
    colors.insert("Secondary_Color".to_string(), state.secondary_color_underscore.hex.get());
    colors.insert("Tertiary_Color".to_string(), state.tertiary_color_underscore.hex.get());

    colors
}

/// Collect color values, only including optional colors if they're not default.
/// Required colors (14) are always included. Optional colors (17) are only included
/// if their value differs from the default gray.
pub fn collect_colors_skip_defaults(state: &DyesState) -> HashMap<String, String> {
    let mut colors = HashMap::new();

    // Required colors (always included)
    colors.insert("Cloth_Primary".to_string(), state.cloth_primary.hex.get());
    colors.insert("Cloth_Secondary".to_string(), state.cloth_secondary.hex.get());
    colors.insert("Cloth_Tertiary".to_string(), state.cloth_tertiary.hex.get());
    colors.insert("Leather_Primary".to_string(), state.leather_primary.hex.get());
    colors.insert("Leather_Secondary".to_string(), state.leather_secondary.hex.get());
    colors.insert("Leather_Tertiary".to_string(), state.leather_tertiary.hex.get());
    colors.insert("Metal_Primary".to_string(), state.metal_primary.hex.get());
    colors.insert("Metal_Secondary".to_string(), state.metal_secondary.hex.get());
    colors.insert("Metal_Tertiary".to_string(), state.metal_tertiary.hex.get());
    colors.insert("Color_01".to_string(), state.color_01.hex.get());
    colors.insert("Color_02".to_string(), state.color_02.hex.get());
    colors.insert("Color_03".to_string(), state.color_03.hex.get());
    colors.insert("Custom_1".to_string(), state.custom_1.hex.get());
    colors.insert("Custom_2".to_string(), state.custom_2.hex.get());

    // Helper to add color only if not default
    let mut add_if_not_default = |name: &str, hex: String| {
        if hex.to_lowercase() != DEFAULT_COLOR {
            colors.insert(name.to_string(), hex);
        }
    };

    // Recommended colors (only if not default)
    add_if_not_default("Accent_Color", state.accent_color.hex.get());
    add_if_not_default("GlowColor", state.glow_color.hex.get());
    add_if_not_default("GlowColour", state.glow_colour.hex.get());

    // Common colors (only if not default)
    add_if_not_default("AddedColor", state.added_color.hex.get());
    add_if_not_default("Highlight_Color", state.highlight_color.hex.get());
    add_if_not_default("BaseColor", state.base_color.hex.get());
    add_if_not_default("InnerColor", state.inner_color.hex.get());
    add_if_not_default("OuterColor", state.outer_color.hex.get());
    add_if_not_default("PrimaryColor", state.primary_color.hex.get());
    add_if_not_default("SecondaryColor", state.secondary_color.hex.get());
    add_if_not_default("TetriaryColor", state.tetriary_color.hex.get());
    add_if_not_default("Primary", state.primary.hex.get());
    add_if_not_default("Secondary", state.secondary.hex.get());
    add_if_not_default("Tertiary", state.tertiary.hex.get());
    add_if_not_default("Primary_Color", state.primary_color_underscore.hex.get());
    add_if_not_default("Secondary_Color", state.secondary_color_underscore.hex.get());
    add_if_not_default("Tertiary_Color", state.tertiary_color_underscore.hex.get());

    colors
}

/// Reset all color pickers to default gray
pub fn reset_colors_to_default(state: &DyesState) {
    let default = DEFAULT_COLOR.to_string();

    // Required colors
    state.cloth_primary.hex.set(default.clone());
    state.cloth_secondary.hex.set(default.clone());
    state.cloth_tertiary.hex.set(default.clone());
    state.leather_primary.hex.set(default.clone());
    state.leather_secondary.hex.set(default.clone());
    state.leather_tertiary.hex.set(default.clone());
    state.metal_primary.hex.set(default.clone());
    state.metal_secondary.hex.set(default.clone());
    state.metal_tertiary.hex.set(default.clone());
    state.color_01.hex.set(default.clone());
    state.color_02.hex.set(default.clone());
    state.color_03.hex.set(default.clone());
    state.custom_1.hex.set(default.clone());
    state.custom_2.hex.set(default.clone());

    // Recommended colors
    state.accent_color.hex.set(default.clone());
    state.glow_color.hex.set(default.clone());
    state.glow_colour.hex.set(default.clone());

    // Common colors
    state.added_color.hex.set(default.clone());
    state.highlight_color.hex.set(default.clone());
    state.base_color.hex.set(default.clone());
    state.inner_color.hex.set(default.clone());
    state.outer_color.hex.set(default.clone());
    state.primary_color.hex.set(default.clone());
    state.secondary_color.hex.set(default.clone());
    state.tetriary_color.hex.set(default.clone());
    state.primary.hex.set(default.clone());
    state.secondary.hex.set(default.clone());
    state.tertiary.hex.set(default.clone());
    state.primary_color_underscore.hex.set(default.clone());
    state.secondary_color_underscore.hex.set(default.clone());
    state.tertiary_color_underscore.hex.set(default);
}

/// Load colors from a HashMap into the state color pickers.
/// Only sets colors that exist in the provided HashMap.
pub fn load_colors_from_map(state: &DyesState, colors: &HashMap<String, String>) {
    for (param_name, hex_color) in colors {
        match param_name.as_str() {
            // Required colors
            "Cloth_Primary" => state.cloth_primary.hex.set(hex_color.clone()),
            "Cloth_Secondary" => state.cloth_secondary.hex.set(hex_color.clone()),
            "Cloth_Tertiary" => state.cloth_tertiary.hex.set(hex_color.clone()),
            "Leather_Primary" => state.leather_primary.hex.set(hex_color.clone()),
            "Leather_Secondary" => state.leather_secondary.hex.set(hex_color.clone()),
            "Leather_Tertiary" => state.leather_tertiary.hex.set(hex_color.clone()),
            "Metal_Primary" => state.metal_primary.hex.set(hex_color.clone()),
            "Metal_Secondary" => state.metal_secondary.hex.set(hex_color.clone()),
            "Metal_Tertiary" => state.metal_tertiary.hex.set(hex_color.clone()),
            "Color_01" => state.color_01.hex.set(hex_color.clone()),
            "Color_02" => state.color_02.hex.set(hex_color.clone()),
            "Color_03" => state.color_03.hex.set(hex_color.clone()),
            "Custom_1" => state.custom_1.hex.set(hex_color.clone()),
            "Custom_2" => state.custom_2.hex.set(hex_color.clone()),
            // Recommended colors
            "Accent_Color" => state.accent_color.hex.set(hex_color.clone()),
            "GlowColor" => state.glow_color.hex.set(hex_color.clone()),
            "GlowColour" => state.glow_colour.hex.set(hex_color.clone()),
            // Common colors
            "AddedColor" => state.added_color.hex.set(hex_color.clone()),
            "Highlight_Color" => state.highlight_color.hex.set(hex_color.clone()),
            "BaseColor" => state.base_color.hex.set(hex_color.clone()),
            "InnerColor" => state.inner_color.hex.set(hex_color.clone()),
            "OuterColor" => state.outer_color.hex.set(hex_color.clone()),
            "PrimaryColor" => state.primary_color.hex.set(hex_color.clone()),
            "SecondaryColor" => state.secondary_color.hex.set(hex_color.clone()),
            "TetriaryColor" => state.tetriary_color.hex.set(hex_color.clone()),
            "Primary" => state.primary.hex.set(hex_color.clone()),
            "Secondary" => state.secondary.hex.set(hex_color.clone()),
            "Tertiary" => state.tertiary.hex.set(hex_color.clone()),
            "Primary_Color" => state.primary_color_underscore.hex.set(hex_color.clone()),
            "Secondary_Color" => state.secondary_color_underscore.hex.set(hex_color.clone()),
            "Tertiary_Color" => state.tertiary_color_underscore.hex.set(hex_color.clone()),
            _ => {} // Unknown parameter, ignore
        }
    }
}
