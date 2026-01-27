//! Mod configuration file handlers for virtual textures
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! This module handles parsing mod configuration files that map GTex hashes
//! to GTS paths:
//! - `VirtualTextures.json` (Script Extender) - has GTex hash → GTS path mapping
//! - `VTexConfig.xml` - has TileSet name and texture definitions

use std::path::{Path, PathBuf};
use serde::Deserialize;
use quick_xml::de::from_str as xml_from_str;

/// VirtualTextures.json structure (Script Extender)
#[derive(Debug, Deserialize)]
pub struct VirtualTexturesJson {
    #[serde(rename = "Mappings")]
    pub mappings: Vec<VTexMapping>,
}

/// A single GTex → GTS mapping
#[derive(Debug, Deserialize)]
pub struct VTexMapping {
    #[serde(rename = "GTexName")]
    pub gtex_name: String,
    #[serde(rename = "GTS")]
    pub gts_path: String,
}

/// VTexConfig.xml structure
#[derive(Debug, Deserialize)]
#[serde(rename = "TileSet")]
pub struct VTexConfigXml {
    #[serde(rename = "@Version")]
    #[allow(dead_code)]
    pub version: Option<String>,
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "Textures")]
    pub textures: Option<VTexConfigTextures>,
}

/// Textures section of VTexConfig.xml
#[derive(Debug, Deserialize)]
pub struct VTexConfigTextures {
    #[serde(rename = "Texture", default)]
    pub textures: Vec<VTexConfigTexture>,
}

/// A single texture entry in VTexConfig.xml
#[derive(Debug, Deserialize)]
pub struct VTexConfigTexture {
    #[serde(rename = "@Name")]
    pub name: String,
}

/// Parsed mod config information
#[derive(Debug)]
pub struct ModConfig {
    /// Path to the mod root directory
    #[allow(dead_code)]
    pub mod_root: PathBuf,
    /// Mod name (from directory structure, used to locate config files)
    pub mod_name: String,
    /// TileSet name from VTexConfig.xml
    pub tileset_name: Option<String>,
    /// GTex hashes from VTexConfig.xml textures
    pub gtex_hashes: Vec<String>,
    /// GTex → GTS mappings from VirtualTextures.json
    pub mappings: Vec<VTexMapping>,
}

/// Find the mod root directory by traversing up from the GTP path
///
/// Looks for the pattern: `ModRoot/Public/<ModName>/Assets/VirtualTextures/`
/// Returns the ModRoot path and mod name if found.
#[must_use]
pub fn find_mod_root(gtp_path: &Path) -> Option<(PathBuf, String)> {
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

/// Load VirtualTextures.json from a mod
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

/// Load VTexConfig.xml from a mod
#[must_use]
pub fn load_vtex_config_xml(mod_root: &Path, mod_name: &str) -> Option<VTexConfigXml> {
    let xml_path = mod_root
        .join("Mods")
        .join(mod_name)
        .join("VTexConfig.xml");

    if !xml_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&xml_path).ok()?;
    xml_from_str(&content).ok()
}

/// Load mod configuration from a GTP file path
///
/// This function:
/// 1. Finds the mod root directory
/// 2. Loads VTexConfig.xml for TileSet name and GTex hashes
/// 3. Loads VirtualTextures.json for GTex → GTS mappings
/// 4. Returns the parsed configuration
#[must_use]
pub fn load_mod_config(gtp_path: &Path) -> Option<ModConfig> {
    let (mod_root, mod_name) = find_mod_root(gtp_path)?;

    let vtex_json = load_virtual_textures_json(&mod_root, &mod_name);
    let vtex_xml = load_vtex_config_xml(&mod_root, &mod_name);

    // Extract tileset name and GTex hashes from VTexConfig.xml
    let (tileset_name, gtex_hashes) = if let Some(ref xml) = vtex_xml {
        let hashes = xml.textures.as_ref()
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

/// Find GTS path for a GTex hash using mod config (for future use)
#[must_use]
#[allow(dead_code)]
pub fn find_gts_for_gtex(config: &ModConfig, gtex_hash: &str) -> Option<PathBuf> {
    for mapping in &config.mappings {
        if mapping.gtex_name == gtex_hash {
            return Some(config.mod_root.join(&mapping.gts_path));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_mod_root() {
        let path = Path::new("/mods/TestMod/Public/TestMod/Assets/VirtualTextures/Test/Test.gtp");
        let result = find_mod_root(path);
        assert!(result.is_some());
        let (root, name) = result.unwrap();
        assert_eq!(root, Path::new("/mods/TestMod"));
        assert_eq!(name, "TestMod");
    }
}
