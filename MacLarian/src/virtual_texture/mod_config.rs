//! Mod configuration file handlers for virtual textures
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! This module handles parsing mod configuration files that map `GTex` hashes
//! to GTS paths:
//! - `VTexConfig.xml` (primary) - has `TileSet` name, paths, and texture definitions
//! - `VirtualTextures.json` (Script Extender, fallback) - has `GTex` hash → GTS path mapping

use quick_xml::de::from_str as xml_from_str;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::pak::PakOperations;

use super::utils::{ExtractResult, extract_all};

// ============================================================================
// VirtualTextures.json types
// ============================================================================

/// `VirtualTextures.json` structure (Script Extender)
#[derive(Debug, Deserialize)]
pub struct VirtualTexturesJson {
    #[serde(rename = "Mappings")]
    pub mappings: Vec<VTexMapping>,
}

/// A single `GTex` → GTS mapping from `VirtualTextures.json`
#[derive(Debug, Clone, Deserialize)]
pub struct VTexMapping {
    #[serde(rename = "GTexName")]
    pub gtex_name: String,
    #[serde(rename = "GTS")]
    pub gts_path: String,
}

// ============================================================================
// VTexConfig.xml types
// ============================================================================

/// `VTexConfig.xml` structure
#[derive(Debug, Deserialize)]
#[serde(rename = "TileSet")]
pub struct VTexConfigXml {
    #[serde(rename = "@Version")]
    pub version: Option<String>,
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "Paths")]
    pub paths: Option<VTexConfigPaths>,
    #[serde(rename = "Textures")]
    pub textures: Option<VTexConfigTextures>,
}

/// Paths section of `VTexConfig.xml`
#[derive(Debug, Deserialize)]
pub struct VTexConfigPaths {
    #[serde(rename = "SourceTextures")]
    pub source_textures: Option<String>,
    #[serde(rename = "VirtualTextures")]
    pub virtual_textures: Option<String>,
}

/// Textures section of `VTexConfig.xml`
#[derive(Debug, Deserialize)]
pub struct VTexConfigTextures {
    #[serde(rename = "Texture", default)]
    pub textures: Vec<VTexConfigTexture>,
}

/// A single texture entry in `VTexConfig.xml`
#[derive(Debug, Deserialize)]
pub struct VTexConfigTexture {
    #[serde(rename = "@Name")]
    pub name: String,
}

// ============================================================================
// Discovery types
// ============================================================================

/// Source of a virtual texture mapping discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverySource {
    /// Discovered from `VTexConfig.xml` (primary, richer metadata)
    VTexConfigXml,
    /// Discovered from `VirtualTextures.json` (Script Extender fallback)
    VirtualTexturesJson,
}

/// A discovered virtual texture in a mod
#[derive(Debug, Clone)]
pub struct DiscoveredVirtualTexture {
    /// Mod name (directory name)
    pub mod_name: String,
    /// Mod root directory path
    pub mod_root: PathBuf,
    /// `TileSet` name from `VTexConfig.xml` (if available)
    pub tileset_name: Option<String>,
    /// `GTex` hash
    pub gtex_hash: String,
    /// Resolved GTS file path
    pub gts_path: PathBuf,
    /// Source of this mapping
    pub source: DiscoverySource,
}

// ============================================================================
// Legacy ModConfig (for internal extractor use)
// ============================================================================

/// Parsed mod config information (used internally by extractor)
#[derive(Debug)]
pub struct ModConfig {
    /// Path to the mod root directory
    pub mod_root: PathBuf,
    /// Mod name (from directory structure, used to locate config files)
    pub mod_name: String,
    /// `TileSet` name from `VTexConfig.xml`
    pub tileset_name: Option<String>,
    /// `GTex` hashes from `VTexConfig.xml` textures
    pub gtex_hashes: Vec<String>,
    /// `GTex` → GTS mappings from `VirtualTextures.json`
    pub mappings: Vec<VTexMapping>,
}

// ============================================================================
// Directory/file discovery helpers
// ============================================================================

/// Find the mod root directory by traversing up from the GTP path
///
/// Looks for the pattern: `ModRoot/Public/<ModName>/Assets/VirtualTextures/`
/// Returns the `ModRoot` path and mod name if found.
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

/// Find mod name by looking for `Mods/<name>/` subdirectory
fn find_mod_name_from_mods_dir(mod_root: &Path) -> Option<String> {
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

// ============================================================================
// Discovery functions
// ============================================================================

/// Discover all virtual textures in a mod directory
///
/// Scans for `VTexConfig.xml` (primary) and `VirtualTextures.json` (fallback).
/// Returns all `GTex` → GTS mappings found.
///
/// # Arguments
/// * `mod_root` - Path to the mod root directory (containing `Mods/` and `Public/`)
///
/// # Returns
/// A list of discovered virtual textures, or an empty vec if none found
pub fn discover_mod_virtual_textures(mod_root: &Path) -> Result<Vec<DiscoveredVirtualTexture>> {
    let mut discovered = Vec::new();
    let mut seen_hashes = std::collections::HashSet::new();

    // Find mod name
    let mod_name = match find_mod_name_from_mods_dir(mod_root) {
        Some(name) => name,
        None => return Ok(discovered), // No mod config found
    };

    // Primary: Try VTexConfig.xml
    if let Some(xml) = load_vtex_config_xml(mod_root, &mod_name) {
        let tileset_name = xml.name.clone();

        // Derive GTS path from Paths/VirtualTextures + TileSet name
        if let Some(ref paths) = xml.paths {
            if let Some(ref vt_path) = paths.virtual_textures {
                // Normalize path separators (Windows uses backslash in XML)
                let vt_path_normalized = vt_path.replace('\\', "/");
                let gts_filename = format!("{tileset_name}.gts");
                let gts_path = mod_root.join(&vt_path_normalized).join(&gts_filename);

                // Add each texture from the XML
                if let Some(ref textures) = xml.textures {
                    for texture in &textures.textures {
                        seen_hashes.insert(texture.name.clone());
                        discovered.push(DiscoveredVirtualTexture {
                            mod_name: mod_name.clone(),
                            mod_root: mod_root.to_path_buf(),
                            tileset_name: Some(tileset_name.clone()),
                            gtex_hash: texture.name.clone(),
                            gts_path: gts_path.clone(),
                            source: DiscoverySource::VTexConfigXml,
                        });
                    }
                }
            }
        }
    }

    // Fallback: Try VirtualTextures.json for any hashes not in XML
    if let Some(json) = load_virtual_textures_json(mod_root, &mod_name) {
        for mapping in json.mappings {
            if !seen_hashes.contains(&mapping.gtex_name) {
                // Normalize path separators
                let gts_path_normalized = mapping.gts_path.replace('\\', "/");
                let gts_path = mod_root.join(&gts_path_normalized);

                discovered.push(DiscoveredVirtualTexture {
                    mod_name: mod_name.clone(),
                    mod_root: mod_root.to_path_buf(),
                    tileset_name: None, // Not available from JSON
                    gtex_hash: mapping.gtex_name,
                    gts_path,
                    source: DiscoverySource::VirtualTexturesJson,
                });
            }
        }
    }

    Ok(discovered)
}

/// Discover virtual textures from within a PAK file
///
/// Scans the PAK for `VTexConfig.xml` and `VirtualTextures.json` files.
///
/// # Arguments
/// * `pak_path` - Path to the PAK file
///
/// # Returns
/// A list of discovered virtual textures, or an empty vec if none found
pub fn discover_pak_virtual_textures(pak_path: &Path) -> Result<Vec<DiscoveredVirtualTexture>> {
    let mut discovered = Vec::new();
    let mut seen_hashes = std::collections::HashSet::new();

    // List files in PAK
    let files = PakOperations::list(pak_path)?;

    // Find VTexConfig.xml files (pattern: Mods/*/VTexConfig.xml)
    let vtex_configs: Vec<_> = files
        .iter()
        .filter(|f| f.ends_with("VTexConfig.xml"))
        .collect();

    // Find VirtualTextures.json files (pattern: Mods/*/ScriptExtender/VirtualTextures.json)
    let vtex_jsons: Vec<_> = files
        .iter()
        .filter(|f| f.ends_with("VirtualTextures.json"))
        .collect();

    // Process VTexConfig.xml files (primary)
    for config_path in &vtex_configs {
        // Extract mod name from path (e.g., "Mods/ModName/VTexConfig.xml" -> "ModName")
        let mod_name = extract_mod_name_from_pak_path(config_path);

        if let Ok(content) = PakOperations::read_file_bytes(pak_path, config_path) {
            if let Ok(content_str) = String::from_utf8(content) {
                if let Ok(xml) = xml_from_str::<VTexConfigXml>(&content_str) {
                    let tileset_name = xml.name.clone();

                    // Derive GTS path from Paths/VirtualTextures + TileSet name
                    if let Some(ref paths) = xml.paths {
                        if let Some(ref vt_path) = paths.virtual_textures {
                            let vt_path_normalized = vt_path.replace('\\', "/");
                            let gts_filename = format!("{tileset_name}.gts");
                            // GTS path is relative to PAK root
                            let gts_path = PathBuf::from(&vt_path_normalized).join(&gts_filename);

                            if let Some(ref textures) = xml.textures {
                                for texture in &textures.textures {
                                    seen_hashes.insert(texture.name.clone());
                                    discovered.push(DiscoveredVirtualTexture {
                                        mod_name: mod_name.clone(),
                                        mod_root: pak_path.to_path_buf(), // PAK file is the "root"
                                        tileset_name: Some(tileset_name.clone()),
                                        gtex_hash: texture.name.clone(),
                                        gts_path: gts_path.clone(),
                                        source: DiscoverySource::VTexConfigXml,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Process VirtualTextures.json files (fallback)
    for json_path in &vtex_jsons {
        let mod_name = extract_mod_name_from_pak_path(json_path);

        if let Ok(content) = PakOperations::read_file_bytes(pak_path, json_path) {
            if let Ok(content_str) = String::from_utf8(content) {
                if let Ok(json) = serde_json::from_str::<VirtualTexturesJson>(&content_str) {
                    for mapping in json.mappings {
                        if !seen_hashes.contains(&mapping.gtex_name) {
                            let gts_path_normalized = mapping.gts_path.replace('\\', "/");
                            let gts_path = PathBuf::from(&gts_path_normalized);

                            discovered.push(DiscoveredVirtualTexture {
                                mod_name: mod_name.clone(),
                                mod_root: pak_path.to_path_buf(),
                                tileset_name: None,
                                gtex_hash: mapping.gtex_name,
                                gts_path,
                                source: DiscoverySource::VirtualTexturesJson,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(discovered)
}

/// Extract mod name from a path inside a PAK file
fn extract_mod_name_from_pak_path(path: &str) -> String {
    // Pattern: "Mods/ModName/..." -> "ModName"
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 && parts[0] == "Mods" {
        parts[1].to_string()
    } else {
        // Fallback to filename without extension
        Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }
}

/// Discover virtual textures across multiple search paths
///
/// Scans each path for mod directories or PAK files containing virtual texture configs.
/// If a path is a .pak file, it will be scanned for VTexConfig.xml/VirtualTextures.json.
/// If a path is itself a mod root, it will be scanned directly.
/// If a path contains multiple mod subdirectories, each will be scanned.
///
/// # Arguments
/// * `search_paths` - Paths to scan (can be mod roots, directories containing mods, or .pak files)
///
/// # Returns
/// All discovered virtual textures across all search paths
pub fn discover_virtual_textures(
    search_paths: &[PathBuf],
) -> Result<Vec<DiscoveredVirtualTexture>> {
    let mut all_discovered = Vec::new();

    for search_path in search_paths {
        if !search_path.exists() {
            continue;
        }

        // Check if this is a PAK file
        let is_pak = search_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"));

        if is_pak {
            if let Ok(discovered) = discover_pak_virtual_textures(search_path) {
                all_discovered.extend(discovered);
            }
            continue;
        }

        // Check if this path is itself a mod root (has Mods/ subdirectory)
        if search_path.join("Mods").is_dir() {
            if let Ok(discovered) = discover_mod_virtual_textures(search_path) {
                all_discovered.extend(discovered);
            }
            continue;
        }

        // Otherwise, scan subdirectories as potential mod roots or PAK files
        if let Ok(entries) = std::fs::read_dir(search_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Check for PAK files
                let is_pak = path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"));

                if is_pak {
                    if let Ok(discovered) = discover_pak_virtual_textures(&path) {
                        all_discovered.extend(discovered);
                    }
                } else if path.is_dir() && path.join("Mods").is_dir() {
                    if let Ok(discovered) = discover_mod_virtual_textures(&path) {
                        all_discovered.extend(discovered);
                    }
                }
            }
        }
    }

    Ok(all_discovered)
}

// ============================================================================
// Lookup functions
// ============================================================================

/// Find GTS path for a `GTex` hash across search paths
///
/// Searches `VTexConfig.xml` first (primary), then falls back to `VirtualTextures.json`.
/// Returns the first match found.
///
/// # Arguments
/// * `gtex_hash` - The `GTex` hash to look up
/// * `search_paths` - Paths to search (mod roots or directories containing mods)
///
/// # Returns
/// The resolved GTS file path if found, or `None` if not found
pub fn find_gts_for_gtex(gtex_hash: &str, search_paths: &[PathBuf]) -> Result<Option<PathBuf>> {
    let discovered = discover_virtual_textures(search_paths)?;

    for vt in discovered {
        if vt.gtex_hash == gtex_hash {
            return Ok(Some(vt.gts_path));
        }
    }

    Ok(None)
}

/// Find a discovered virtual texture by `GTex` hash
///
/// Like `find_gts_for_gtex` but returns full discovery information.
pub fn find_virtual_texture(
    gtex_hash: &str,
    search_paths: &[PathBuf],
) -> Result<Option<DiscoveredVirtualTexture>> {
    let discovered = discover_virtual_textures(search_paths)?;

    for vt in discovered {
        if vt.gtex_hash == gtex_hash {
            return Ok(Some(vt));
        }
    }

    Ok(None)
}

// ============================================================================
// High-level extraction
// ============================================================================

/// Extract a virtual texture by `GTex` hash
///
/// Discovers the GTS location from mod configs and extracts to the output directory.
///
/// # Arguments
/// * `gtex_hash` - The `GTex` hash to extract
/// * `search_paths` - Paths to search for the mod containing this texture
/// * `output_dir` - Directory to write extracted DDS files
///
/// # Errors
/// Returns an error if the `GTex` hash is not found or extraction fails.
pub fn extract_by_gtex(
    gtex_hash: &str,
    search_paths: &[PathBuf],
    output_dir: &Path,
) -> Result<ExtractResult> {
    let vt = find_virtual_texture(gtex_hash, search_paths)?.ok_or_else(|| {
        Error::ConversionError(format!("GTex hash '{gtex_hash}' not found in search paths"))
    })?;

    if !vt.gts_path.exists() {
        return Err(Error::ConversionError(format!(
            "GTS file not found at derived path: {}",
            vt.gts_path.display()
        )));
    }

    // Extract all GTPs referenced by this GTS
    extract_all(&vt.gts_path, output_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Integration test with real mod directory (skipped if not present)
    #[test]
    fn test_discover_medusae_outfits() {
        let mod_root =
            PathBuf::from("/Users/corrine/Desktop/extracted_paks/all_mods/Medusae_Outfits");
        if !mod_root.exists() {
            eprintln!("Skipping test: Medusae_Outfits mod not found");
            return;
        }

        let discovered =
            discover_mod_virtual_textures(&mod_root).expect("Discovery should not fail");

        assert!(
            !discovered.is_empty(),
            "Should discover at least one texture"
        );

        let first = &discovered[0];
        assert_eq!(first.mod_name, "Medusae_Outfits");
        assert_eq!(first.gtex_hash, "2d1fe47f16484210a68d71256662a676");
        assert_eq!(
            first.tileset_name,
            Some("Medusae_Outfits_Textures".to_string())
        );
        assert_eq!(first.source, DiscoverySource::VTexConfigXml);
        assert!(
            first.gts_path.ends_with("Medusae_Outfits_Textures.gts"),
            "GTS path should end with tileset name"
        );

        println!("Discovered: {:?}", first);
    }

    #[test]
    fn test_parse_vtex_config_xml() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<TileSet Version="2" Name="Medusae_Outfits_Textures">
    <Paths>
        <SourceTextures>Public\Medusae_Outfits\Assets\Textures</SourceTextures>
        <VirtualTextures>Public\Medusae_Outfits\Assets\VirtualTextures</VirtualTextures>
    </Paths>
    <Textures>
        <Texture Name="2d1fe47f16484210a68d71256662a676">
            <Layer Name="Albedo" Source="2d1fe47f16484210a68d71256662a676_0.dds" />
        </Texture>
    </Textures>
</TileSet>"#;

        let config: VTexConfigXml = xml_from_str(xml).expect("Failed to parse XML");

        assert_eq!(config.name, "Medusae_Outfits_Textures");
        assert_eq!(config.version, Some("2".to_string()));

        let paths = config.paths.expect("Should have paths");
        assert_eq!(
            paths.virtual_textures,
            Some("Public\\Medusae_Outfits\\Assets\\VirtualTextures".to_string())
        );

        let textures = config.textures.expect("Should have textures");
        assert_eq!(textures.textures.len(), 1);
        assert_eq!(
            textures.textures[0].name,
            "2d1fe47f16484210a68d71256662a676"
        );
    }

    #[test]
    fn test_parse_virtual_textures_json() {
        let json = r#"{
    "Mappings": [
        {
            "GTexName": "2d1fe47f16484210a68d71256662a676",
            "GTS": "Public/Medusae_Outfits/Assets/VirtualTextures/Medusae_Outfits_Textures.gts"
        }
    ]
}"#;

        let config: VirtualTexturesJson = serde_json::from_str(json).expect("Failed to parse JSON");

        assert_eq!(config.mappings.len(), 1);
        assert_eq!(
            config.mappings[0].gtex_name,
            "2d1fe47f16484210a68d71256662a676"
        );
        assert_eq!(
            config.mappings[0].gts_path,
            "Public/Medusae_Outfits/Assets/VirtualTextures/Medusae_Outfits_Textures.gts"
        );
    }
}
