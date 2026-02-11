//! Helper functions for the Dyes tab

use super::constants::COLOR_DEFAULT_GRAY;
use floem::prelude::*;

/// Parse hex string to Color for display
pub fn parse_hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return Color::rgb8(r, g, b);
        }
    }
    COLOR_DEFAULT_GRAY
}

/// Parse hex string to RGB tuple
pub fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Normalize hex string (uppercase, 6 chars)
pub fn normalize_hex(hex: &str) -> String {
    let hex = hex.trim_start_matches('#').to_uppercase();
    if hex.len() == 3 {
        // Expand shorthand (e.g., "F00" -> "FF0000")
        hex.chars().flat_map(|c| [c, c]).collect()
    } else if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        hex
    } else {
        "808080".to_string() // Default to gray if invalid
    }
}

/// Copy text to clipboard
pub fn copy_to_clipboard(text: &str) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                child.wait()
            });
    }
}
