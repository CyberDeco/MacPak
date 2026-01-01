//! Helper functions for the Dyes tab

use floem::prelude::*;
use std::process::Command;

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
    Color::rgb8(128, 128, 128) // Default gray
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
        hex.chars()
            .flat_map(|c| [c, c])
            .collect()
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

/// Opens the native macOS color picker and returns the selected color as (R, G, B).
pub fn pick_color_from_screen() -> Option<(u8, u8, u8)> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"choose color"#;
        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split(", ").collect();
        if parts.len() >= 3 {
            let r16: u32 = parts[0].parse().ok()?;
            let g16: u32 = parts[1].parse().ok()?;
            let b16: u32 = parts[2].parse().ok()?;

            // Convert 16-bit to 8-bit
            let r = (r16 / 257) as u8;
            let g = (g16 / 257) as u8;
            let b = (b16 / 257) as u8;

            return Some((r, g, b));
        }
        None
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}
