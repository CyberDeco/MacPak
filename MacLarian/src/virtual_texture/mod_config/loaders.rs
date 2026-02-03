//! Config file loaders for virtual textures
//!
//!

use quick_xml::de::from_str as xml_from_str;
use std::path::Path;

use crate::formats::lsf::LsfDocument;

use super::types::{ModConfig, VTexConfigXml, VirtualTexturesJson};

// ============================================================================
// Directory/file discovery helpers
// ============================================================================

/// Find the mod root directory by traversing up from the GTP path
///
/// Looks for the pattern: `ModRoot/Public/<ModName>/Assets/VirtualTextures/`
/// Returns the `ModRoot` path and mod name if found.
#[must_use]
pub fn find_mod_root(gtp_path: &Path) -> Option<(std::path::PathBuf, String)> {
    let mut current = gtp_path.parent()?;

    // Traverse up looking for "Public" directory
    loop {
        if let Some(name) = current.file_name().and_then(|n| n.to_str()) {
            if name == "Public" {
                // Found Public, parent is mod root
                let mod_root = current.parent()?.to_path_buf();

                // Get mod name from the next directory after Public
                let remaining = gtp_path.strip_prefix(current).ok()?;
                let mod_name = remaining.iter().next()?.to_str()?.to_string();

                return Some((mod_root, mod_name));
            }
        }
        current = current.parent()?;
    }
}

/// Find mod name by looking for `Mods/<name>/` subdirectory
pub fn find_mod_name_from_mods_dir(mod_root: &Path) -> Option<String> {
    let mods_dir = mod_root.join("Mods");
    if !mods_dir.is_dir() {
        return None;
    }

    // Look for first subdirectory that contains VTexConfig.xml or ScriptExtender/
    for entry in std::fs::read_dir(&mods_dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let has_vtex_config = path.join("VTexConfig.xml").exists();
            let has_script_extender = path.join("ScriptExtender").is_dir();
            if has_vtex_config || has_script_extender {
                return path.file_name()?.to_str().map(String::from);
            }
        }
    }
    None
}

// ============================================================================
// Config file loaders
// ============================================================================

/// Load `VirtualTextures.json` from a mod
#[must_use]
pub fn load_virtual_textures_json(mod_root: &Path, mod_name: &str) -> Option<VirtualTexturesJson> {
    let json_path = mod_root
        .join("Mods")
        .join(mod_name)
        .join("ScriptExtender")
        .join("VirtualTextures.json");

    if !json_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&json_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Load `VTexConfig.xml` from a mod
#[must_use]
pub fn load_vtex_config_xml(mod_root: &Path, mod_name: &str) -> Option<VTexConfigXml> {
    let xml_path = mod_root.join("Mods").join(mod_name).join("VTexConfig.xml");

    if !xml_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&xml_path).ok()?;
    xml_from_str(&content).ok()
}

// ============================================================================
// LSF VTexConfig parsing (for PAK files)
// ============================================================================

/// Parsed `VTexConfig` data from LSF format
#[derive(Debug)]
pub struct LsfVTexConfig {
    /// The tile set name.
    pub tileset_name: String,
    /// Path to virtual textures output directory.
    pub virtual_textures_path: Option<String>,
    /// Names of textures in the tile set.
    pub texture_names: Vec<String>,
}

/// Parse `VTexConfig` from `LsfDocument`
pub fn parse_vtex_config_from_lsf(doc: &LsfDocument) -> Option<LsfVTexConfig> {
    // Find root TileSet node
    let root_nodes = doc.root_nodes();
    let tileset_idx = root_nodes
        .iter()
        .find(|&&idx| doc.node_name(idx) == Some("TileSet"))?;

    // Get TileSet Name attribute
    let tileset_name = doc.get_fixed_string_attr(*tileset_idx, "Name")?;

    // Find Paths child node
    let paths_nodes = doc.find_children_by_name(*tileset_idx, "Paths");
    let virtual_textures_path = if let Some(&paths_idx) = paths_nodes.first() {
        // Find VirtualTextures child - its text content is stored as an attribute
        let vt_nodes = doc.find_children_by_name(paths_idx, "VirtualTextures");
        vt_nodes.first().and_then(|&vt_idx| {
            // Text content is typically stored as a FixedString attribute with empty name
            // or as the first string-type attribute
            get_text_content(doc, vt_idx)
        })
    } else {
        None
    };

    // Find Textures child node
    let textures_nodes = doc.find_children_by_name(*tileset_idx, "Textures");
    let texture_names = if let Some(&textures_idx) = textures_nodes.first() {
        // Find all Texture children and get their Name attributes
        doc.find_children_by_name(textures_idx, "Texture")
            .iter()
            .filter_map(|&tex_idx| doc.get_fixed_string_attr(tex_idx, "Name"))
            .collect()
    } else {
        Vec::new()
    };

    Some(LsfVTexConfig {
        tileset_name,
        virtual_textures_path,
        texture_names,
    })
}

/// Get text content from an LSF node (stored as empty-name or value attribute)
fn get_text_content(doc: &LsfDocument, node_idx: usize) -> Option<String> {
    for (_, name, type_id, offset, length) in doc.attributes_of(node_idx) {
        // Text content is often stored with empty name or as first string attribute
        // FixedString = 22, LSString = 19
        if (name.is_empty() || name == "value") && (type_id == 22 || type_id == 19) {
            if offset + length > doc.values.len() {
                continue;
            }
            let bytes = &doc.values[offset..offset + length];

            if type_id == 22 {
                // FixedString: null-terminated
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                return String::from_utf8(bytes[..end].to_vec()).ok();
            } else if type_id == 19 && length >= 4 {
                // LSString: 4-byte length prefix followed by null-terminated string
                let str_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
                if str_len > 0 && 4 + str_len <= bytes.len() {
                    let str_bytes = &bytes[4..4 + str_len - 1]; // -1 for null terminator
                    return String::from_utf8(str_bytes.to_vec()).ok();
                }
            }
        }
    }
    None
}

/// Load mod configuration from a GTP file path (used by extractor)
///
/// This function:
/// 1. Finds the mod root directory
/// 2. Loads `VTexConfig.xml` for `TileSet` name and `GTex` hashes
/// 3. Loads `VirtualTextures.json` for `GTex` → GTS mappings
/// 4. Returns the parsed configuration
#[must_use]
pub fn load_mod_config(gtp_path: &Path) -> Option<ModConfig> {
    let (mod_root, mod_name) = find_mod_root(gtp_path)?;

    let vtex_json = load_virtual_textures_json(&mod_root, &mod_name);
    let vtex_xml = load_vtex_config_xml(&mod_root, &mod_name);

    // Extract tileset name and GTex hashes from VTexConfig.xml
    let (tileset_name, gtex_hashes) = if let Some(ref xml) = vtex_xml {
        let hashes = xml
            .textures
            .as_ref()
            .map(|t| t.textures.iter().map(|tex| tex.name.clone()).collect())
            .unwrap_or_default();
        (Some(xml.name.clone()), hashes)
    } else {
        (None, Vec::new())
    };

    Some(ModConfig {
        mod_root,
        mod_name,
        tileset_name,
        gtex_hashes,
        mappings: vtex_json.map(|v| v.mappings).unwrap_or_default(),
    })
}
