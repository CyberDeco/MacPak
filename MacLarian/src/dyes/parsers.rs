//! Parsing functions for dye mod files

use std::collections::HashMap;
use super::types::{ParsedDyeEntry, DyeLocalizationInfo, ImportedDyeEntry};

/// Parse ItemCombos.txt to extract dye entries
pub fn parse_item_combos(content: &str) -> Vec<ParsedDyeEntry> {
    let mut entries = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_preset_uuid: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        // Look for: new ItemCombination "DyeName"
        if line.starts_with("new ItemCombination") {
            // Save previous entry if exists
            if let Some(name) = current_name.take() {
                entries.push(ParsedDyeEntry {
                    name,
                    preset_uuid: current_preset_uuid.take(),
                    root_template_uuid: None,
                });
            }

            // Extract name from quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    current_name = Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }

        // Look for: data "DyeColorPresetResource" "uuid"
        if line.contains("DyeColorPresetResource") {
            // Find the UUID (second quoted string)
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 4 {
                current_preset_uuid = Some(parts[3].to_string());
            }
        }
    }

    // Don't forget the last entry
    if let Some(name) = current_name {
        entries.push(ParsedDyeEntry {
            name,
            preset_uuid: current_preset_uuid,
            root_template_uuid: None,
        });
    }

    entries
}

/// Parse Object.txt to extract dye entries
pub fn parse_object_txt(content: &str) -> Vec<ParsedDyeEntry> {
    let mut entries = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_root_uuid: Option<String> = None;
    let mut is_dye = false;

    for line in content.lines() {
        let line = line.trim();

        // Look for: new Object "DyeName" or new entry "DyeName"
        if line.starts_with("new Object") || line.starts_with("new entry") {
            // Save previous entry if it was a dye
            if let Some(name) = current_name.take() {
                if is_dye {
                    entries.push(ParsedDyeEntry {
                        name,
                        preset_uuid: None,
                        root_template_uuid: current_root_uuid.take(),
                    });
                }
            }
            is_dye = false;
            current_root_uuid = None;

            // Extract name from quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    current_name = Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }

        // Check if this object uses _Dyes
        if line.contains("using") && line.contains("_Dyes") {
            is_dye = true;
        }

        // Look for: data "RootTemplate" "uuid"
        if line.contains("RootTemplate") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 4 {
                current_root_uuid = Some(parts[3].to_string());
            }
        }
    }

    // Don't forget the last entry
    if let Some(name) = current_name {
        if is_dye {
            entries.push(ParsedDyeEntry {
                name,
                preset_uuid: None,
                root_template_uuid: current_root_uuid,
            });
        }
    }

    entries
}

/// Convert linear color value to sRGB (gamma correction)
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert fvec3 string (e.g., "0.5 0.25 0.75") to hex color (e.g., "804040BF")
/// Applies sRGB gamma correction since game stores colors in linear space
pub fn fvec3_to_hex(fvec3: &str) -> String {
    let parts: Vec<f32> = fvec3
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() >= 3 {
        // Apply gamma correction (linear -> sRGB) for correct display
        let r = (linear_to_srgb(parts[0].clamp(0.0, 1.0)) * 255.0).round() as u8;
        let g = (linear_to_srgb(parts[1].clamp(0.0, 1.0)) * 255.0).round() as u8;
        let b = (linear_to_srgb(parts[2].clamp(0.0, 1.0)) * 255.0).round() as u8;
        format!("{:02X}{:02X}{:02X}", r, g, b)
    } else {
        "808080".to_string() // Default gray
    }
}

/// Extract an XML attribute value from a line
pub fn extract_xml_attribute(line: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(start) = line.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = line[value_start..].find('"') {
            return Some(line[value_start..value_start + end].to_string());
        }
    }
    None
}

/// Parse LSX content (converted from LSF) to extract dye entries with colors
pub fn parse_lsx_dye_presets(lsx_content: &str) -> Vec<ImportedDyeEntry> {
    let mut entries = Vec::new();

    // Track nesting depth within Resource nodes
    let mut current_entry: Option<ImportedDyeEntry> = None;
    let mut resource_depth: i32 = 0;
    let mut current_param_name: Option<String> = None;

    for line in lsx_content.lines() {
        let line = line.trim();

        // Start of a Resource node
        if line.contains("<node id=\"Resource\">") {
            resource_depth = 1;
            current_entry = Some(ImportedDyeEntry::default());
            continue;
        }

        // Track nesting within Resource
        if resource_depth > 0 {
            // Count opening and closing tags
            if line.starts_with("<node ") && !line.ends_with("/>") {
                resource_depth += 1;
            }
            if line == "</node>" {
                resource_depth -= 1;

                // If we've closed the Resource node
                if resource_depth == 0 {
                    if let Some(entry) = current_entry.take() {
                        if !entry.name.is_empty() {
                            entries.push(entry);
                        }
                    }
                    continue;
                }
            }

            // Parse attributes within Resource
            if let Some(entry) = current_entry.as_mut() {
                // ID attribute (Preset UUID for ItemCombos.txt) - only at Resource level
                if line.contains("attribute id=\"ID\"") && line.contains("type=\"FixedString\"") {
                    if let Some(value) = extract_xml_attribute(line, "value") {
                        entry.preset_uuid = Some(value);
                    }
                }

                // Name attribute
                if line.contains("attribute id=\"Name\"") {
                    if let Some(value) = extract_xml_attribute(line, "value") {
                        entry.name = value;
                    }
                }

                // Color parameter name - store for pairing with value
                if line.contains("attribute id=\"Parameter\"") {
                    if let Some(param_name) = extract_xml_attribute(line, "value") {
                        current_param_name = Some(param_name);
                    }
                }

                // Color value (fvec3) - pair with stored parameter name
                if line.contains("attribute id=\"Value\"") && line.contains("type=\"fvec3\"") {
                    if let Some(fvec3) = extract_xml_attribute(line, "value") {
                        if let Some(param_name) = current_param_name.take() {
                            let hex = fvec3_to_hex(&fvec3);
                            entry.colors.insert(param_name, hex);
                        }
                    }
                }
            }
        }
    }

    entries
}

/// Parse RootTemplates LSX to extract localization handles for dyes
pub fn parse_root_templates_localization(lsx_content: &str) -> Vec<DyeLocalizationInfo> {
    let mut entries = Vec::new();
    let mut current_entry: Option<DyeLocalizationInfo> = None;
    let mut in_game_objects = false;

    for line in lsx_content.lines() {
        let line = line.trim();

        // Start of a GameObjects node
        if line.contains("<node id=\"GameObjects\">") {
            in_game_objects = true;
            current_entry = Some(DyeLocalizationInfo::default());
            continue;
        }

        // End of GameObjects node
        if in_game_objects && line == "</node>" {
            if let Some(entry) = current_entry.take() {
                if !entry.name.is_empty() {
                    entries.push(entry);
                }
            }
            in_game_objects = false;
            continue;
        }

        if in_game_objects {
            if let Some(entry) = current_entry.as_mut() {
                // Name attribute
                if line.contains("attribute id=\"Name\"") {
                    if let Some(value) = extract_xml_attribute(line, "value") {
                        entry.name = value;
                    }
                }

                // DisplayName - TranslatedString with handle
                if line.contains("attribute id=\"DisplayName\"") {
                    if let Some(handle) = extract_xml_attribute(line, "handle") {
                        entry.display_name_handle = Some(handle);
                    }
                }

                // Description - TranslatedString with handle
                if line.contains("attribute id=\"Description\"") {
                    if let Some(handle) = extract_xml_attribute(line, "handle") {
                        entry.description_handle = Some(handle);
                    }
                }
            }
        }
    }

    entries
}

/// Parse localization XML to build a map of contentuid -> text
pub fn parse_localization_xml(xml_content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in xml_content.lines() {
        let line = line.trim();

        // Look for: <content contentuid="handle" version="1">Text</content>
        if line.starts_with("<content ") && line.contains("contentuid=") {
            if let Some(handle) = extract_xml_attribute(line, "contentuid") {
                // Extract text between > and </content>
                if let Some(start) = line.find('>') {
                    if let Some(end) = line.rfind("</content>") {
                        let text = &line[start + 1..end];
                        map.insert(handle, text.to_string());
                    }
                }
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fvec3_to_hex() {
        // Pure white in linear should be white in sRGB
        assert_eq!(fvec3_to_hex("1 1 1"), "FFFFFF");
        // Black
        assert_eq!(fvec3_to_hex("0 0 0"), "000000");
        // Mid gray (linear 0.5 -> sRGB ~0.735)
        let hex = fvec3_to_hex("0.5 0.5 0.5");
        assert!(hex.starts_with("BC")); // Approximately 188
    }

    #[test]
    fn test_extract_xml_attribute() {
        let line = r#"<attribute id="Name" type="LSString" value="TestDye" />"#;
        assert_eq!(extract_xml_attribute(line, "value"), Some("TestDye".to_string()));
        assert_eq!(extract_xml_attribute(line, "id"), Some("Name".to_string()));
        assert_eq!(extract_xml_attribute(line, "missing"), None);
    }
}
