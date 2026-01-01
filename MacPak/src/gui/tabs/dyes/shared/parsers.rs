//! Parsing functions for dye mod files

use crate::gui::state::ImportedDyeEntry;

/// Parsed dye entry from TXT mod files
#[derive(Clone, Debug)]
pub struct ParsedDyeEntry {
    pub name: String,
    pub preset_uuid: Option<String>,
    pub root_template_uuid: Option<String>,
}

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

/// Convert fvec3 string (e.g., "0.5 0.25 0.75") to hex color (e.g., "804040BF")
pub fn fvec3_to_hex(fvec3: &str) -> String {
    let parts: Vec<f32> = fvec3
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() >= 3 {
        let r = (parts[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (parts[1].clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (parts[2].clamp(0.0, 1.0) * 255.0).round() as u8;
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
