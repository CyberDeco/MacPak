//! Content generation functions for dye mod files

use std::collections::HashMap;
use super::colors::DEFAULT_COLOR;

/// Convert sRGB color value to linear (inverse gamma correction)
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert hex color (e.g., "FF0000") to fvec3 string (e.g., "1 0 0")
/// Applies inverse gamma correction since game expects colors in linear space
pub fn hex_to_fvec3(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return "0.5 0.5 0.5".to_string(); // Default gray
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;

    // Convert from sRGB to linear for game storage
    let r = srgb_to_linear(r);
    let g = srgb_to_linear(g);
    let b = srgb_to_linear(b);

    format!("{:.6} {:.6} {:.6}", r, g, b)
}

/// Generate Vector3Parameters XML nodes from a color HashMap
/// Skips colors matching DEFAULT_COLOR (#808080) - use for new mod exports
pub fn generate_color_nodes(colors: &HashMap<String, String>) -> String {
    colors
        .iter()
        .filter(|(_, hex)| {
            let normalized = hex.trim_start_matches('#').to_lowercase();
            normalized != DEFAULT_COLOR
        })
        .map(|(name, hex)| {
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
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate Vector3Parameters XML nodes from a color HashMap
/// Includes ALL colors - use for re-exporting imported mods
pub fn generate_all_color_nodes(colors: &HashMap<String, String>) -> String {
    colors
        .iter()
        .map(|(name, hex)| {
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
        })
        .collect::<Vec<_>>()
        .join("\n")
}
