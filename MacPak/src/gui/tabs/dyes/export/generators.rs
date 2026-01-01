//! Content generation functions for dye mod files

/// Convert hex color (e.g., "FF0000") to fvec3 string (e.g., "1 0 0")
pub fn hex_to_fvec3(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return "0.5 0.5 0.5".to_string(); // Default gray
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;

    format!("{:.6} {:.6} {:.6}", r, g, b)
}
