//! Content generation functions for dye mod files

use std::collections::HashMap;
use super::registry::DEFAULT_HEX;

/// Convert sRGB color value to linear (inverse gamma correction)
#[must_use] 
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert hex color (e.g., "FF0000") to fvec3 string (e.g., "1 0 0")
/// Applies inverse gamma correction since game expects colors in linear space
#[must_use] 
pub fn hex_to_fvec3(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return "0.5 0.5 0.5".to_string(); // Default gray
    }

    let r = f32::from(u8::from_str_radix(&hex[0..2], 16).unwrap_or(128)) / 255.0;
    let g = f32::from(u8::from_str_radix(&hex[2..4], 16).unwrap_or(128)) / 255.0;
    let b = f32::from(u8::from_str_radix(&hex[4..6], 16).unwrap_or(128)) / 255.0;

    // Convert from sRGB to linear for game storage
    let r = srgb_to_linear(r);
    let g = srgb_to_linear(g);
    let b = srgb_to_linear(b);

    format!("{r:.6} {g:.6} {b:.6}")
}

/// Format a single color as a `Vector3Parameters` XML node
fn format_color_node(name: &str, hex: &str) -> String {
    let fvec3 = hex_to_fvec3(hex);
    format!(
        r#"								<node id="Vector3Parameters">
									<attribute id="Color" type="bool" value="True" />
									<attribute id="Custom" type="bool" value="False" />
									<attribute id="Enabled" type="bool" value="True" />
									<attribute id="Parameter" type="FixedString" value="{name}" />
									<attribute id="Value" type="fvec3" value="{fvec3}" />
								</node>"#
    )
}

/// Generate `Vector3Parameters` XML nodes from a color `HashMap`
///
/// If `include_defaults` is false, skips colors matching `DEFAULT_HEX` (#808080)
fn generate_color_nodes_impl(colors: &HashMap<String, String>, include_defaults: bool) -> String {
    colors
        .iter()
        .filter(|(_, hex)| {
            if include_defaults {
                true
            } else {
                let normalized = hex.trim_start_matches('#').to_lowercase();
                normalized != DEFAULT_HEX
            }
        })
        .map(|(name, hex)| format_color_node(name, hex))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate `Vector3Parameters` XML nodes, skipping default colors
/// Use for new mod exports where unchanged colors shouldn't be included
#[must_use] 
pub fn generate_color_nodes(colors: &HashMap<String, String>) -> String {
    generate_color_nodes_impl(colors, false)
}

/// Generate `Vector3Parameters` XML nodes, including ALL colors
/// Use for re-exporting imported mods where all colors should be preserved
#[must_use] 
pub fn generate_all_color_nodes(colors: &HashMap<String, String>) -> String {
    generate_color_nodes_impl(colors, true)
}

