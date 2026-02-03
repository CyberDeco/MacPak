//! Virtual texture discovery functions
//!
//!

use quick_xml::de::from_str as xml_from_str;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::formats::lsf::parse_lsf_bytes;
use crate::pak::PakOperations;

use super::loaders::{
    find_mod_name_from_mods_dir, load_virtual_textures_json, load_vtex_config_xml,
    parse_vtex_config_from_lsf,
};
use super::types::{DiscoveredVirtualTexture, DiscoverySource, VTexConfigXml, VirtualTexturesJson};

// ============================================================================
// Filesystem discovery
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
    let mut seen_hashes = HashSet::new();

    // Find mod name (optional - GTS fallback works without it)
    let mod_name = find_mod_name_from_mods_dir(mod_root);

    // Primary: Try VTexConfig.xml (requires mod_name)
    if let Some(ref mod_name) = mod_name {
        if let Some(xml) = load_vtex_config_xml(mod_root, mod_name) {
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

        // Secondary fallback: Try VirtualTextures.json for any hashes not in XML
        if let Some(json) = load_virtual_textures_json(mod_root, mod_name) {
            for mapping in json.mappings {
                if !seen_hashes.contains(&mapping.gtex_name) {
                    seen_hashes.insert(mapping.gtex_name.clone());
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
    }

    // Tertiary fallback: Scan for GTS files directly (for raw editor output)
    // Only run if this looks like a mod root (has Public/ or Generated/ directory)
    // This prevents recursive scanning when called on a parent directory containing multiple mods
    let has_public = mod_root.join("Public").is_dir();
    let has_generated = mod_root.join("Generated").is_dir();
    let has_mods = mod_root.join("Mods").is_dir();

    if has_public || has_generated || has_mods {
        // Track seen GTS paths to avoid duplicates
        let seen_gts_paths: HashSet<_> = discovered.iter().map(|d| d.gts_path.clone()).collect();
        let gts_discovered = discover_gts_files(mod_root, &seen_hashes, &seen_gts_paths);
        discovered.extend(gts_discovered);
    }

    Ok(discovered)
}

/// Discover virtual textures by scanning for GTS files (fallback)
///
/// This is a tertiary fallback for mods that don't have VTexConfig.xml or VirtualTextures.json
/// but do have GTS/GTP files (e.g., raw editor output).
///
/// # Arguments
/// * `mod_root` - Path to the mod root directory
/// * `seen_hashes` - Set of already-discovered GTex hashes to skip
/// * `seen_gts_paths` - Set of already-discovered GTS file paths to skip
///
/// # Returns
/// A list of discovered virtual textures
fn discover_gts_files(
    mod_root: &Path,
    seen_hashes: &HashSet<String>,
    seen_gts_paths: &HashSet<PathBuf>,
) -> Vec<DiscoveredVirtualTexture> {
    let mut discovered = Vec::new();

    // Find all .gts files recursively
    let gts_files = find_gts_files_recursive(mod_root);

    for gts_path in gts_files {
        // Skip if this GTS file was already discovered
        if seen_gts_paths.contains(&gts_path) {
            continue;
        }
        // Also check if a GTS with the same stem was already discovered
        let gts_stem = gts_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let is_duplicate = seen_gts_paths
            .iter()
            .any(|p| p.file_stem().and_then(|s| s.to_str()) == Some(gts_stem));
        if is_duplicate {
            continue;
        }

        // Extract mod name from path
        let mod_name = extract_mod_name_from_path(&gts_path, mod_root);
        let gts_dir = gts_path.parent().unwrap_or(mod_root);

        // Find .gtp files in the same directory
        if let Ok(entries) = std::fs::read_dir(gts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("gtp"))
                {
                    let gtp_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                    // GTP naming: either "<tileset>_<gtex_hash>.gtp" or "<tileset>.gtp"
                    let gtex_hash =
                        if let Some(hash) = extract_gtex_hash_from_gtp_name(gtp_name, gts_stem) {
                            hash
                        } else {
                            // Use GTS stem as a pseudo-hash if we can't extract one
                            gts_stem.to_string()
                        };

                    // Skip if already discovered
                    if seen_hashes.contains(&gtex_hash) {
                        continue;
                    }

                    // Try to find a companion StackedTexture XML for more metadata
                    let tileset_name = find_stacked_texture_xml(mod_root, &gtex_hash)
                        .or_else(|| Some(gts_stem.to_string()));

                    discovered.push(DiscoveredVirtualTexture {
                        mod_name: mod_name.clone(),
                        mod_root: mod_root.to_path_buf(),
                        tileset_name,
                        gtex_hash,
                        gts_path: gts_path.clone(),
                        source: DiscoverySource::GtsFileScan,
                    });
                }
            }
        }

        // If no GTP files found, still register the GTS with its stem as identifier
        if discovered.is_empty() && !seen_hashes.contains(gts_stem) {
            let mod_name = extract_mod_name_from_path(&gts_path, mod_root);
            discovered.push(DiscoveredVirtualTexture {
                mod_name,
                mod_root: mod_root.to_path_buf(),
                tileset_name: Some(gts_stem.to_string()),
                gtex_hash: gts_stem.to_string(),
                gts_path: gts_path.clone(),
                source: DiscoverySource::GtsFileScan,
            });
        }
    }

    discovered
}

/// Recursively find all .gts files in a directory
fn find_gts_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut gts_files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                gts_files.extend(find_gts_files_recursive(&path));
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("gts"))
            {
                gts_files.push(path);
            }
        }
    }

    gts_files
}

/// Extract GTex hash from GTP filename
///
/// GTP files are named: `<tileset>_<gtex_hash>.gtp` or `<tileset>.gtp`
/// Returns the GTex hash if found, or None if the GTP has no hash suffix.
fn extract_gtex_hash_from_gtp_name(gtp_name: &str, gts_stem: &str) -> Option<String> {
    // Check if GTP name starts with GTS stem followed by underscore
    if gtp_name.starts_with(gts_stem) && gtp_name.len() > gts_stem.len() + 1 {
        let suffix = &gtp_name[gts_stem.len()..];
        if suffix.starts_with('_') {
            let hash = &suffix[1..];
            // Validate it looks like a hash (32 hex chars typical)
            if !hash.is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(hash.to_string());
            }
        }
    }
    None
}

/// Extract mod name from a file path relative to mod root
fn extract_mod_name_from_path(file_path: &Path, mod_root: &Path) -> String {
    // Try to find "Public/<ModName>" or "Generated/Public/<ModName>" in path
    if let Ok(relative) = file_path.strip_prefix(mod_root) {
        let parts: Vec<_> = relative.iter().collect();

        // Look for "Public" and take the next component as mod name
        for (i, part) in parts.iter().enumerate() {
            if part.to_string_lossy().eq_ignore_ascii_case("Public") {
                if let Some(mod_name) = parts.get(i + 1) {
                    return mod_name.to_string_lossy().to_string();
                }
            }
        }
    }

    // Fallback: use mod_root directory name
    mod_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

/// Try to find a StackedTexture XML file for a given GTex hash
fn find_stacked_texture_xml(mod_root: &Path, gtex_hash: &str) -> Option<String> {
    // Look for <gtex_hash>.xml files in VirtualTextures subdirectories
    let xml_name = format!("{gtex_hash}.xml");

    for entry in walkdir_simple(mod_root, 5) {
        if entry
            .file_name()
            .is_some_and(|n| n.to_string_lossy() == xml_name)
        {
            // Try to parse and extract useful info
            if let Ok(content) = std::fs::read_to_string(&entry) {
                // Check if it's a StackedTexture XML
                if content.contains("<StackedTexture>") {
                    // Could parse more details here, but for now just confirm it exists
                    return Some(gtex_hash.to_string());
                }
            }
        }
    }
    None
}

/// Simple recursive directory walk with depth limit
fn walkdir_simple(dir: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    walkdir_simple_inner(dir, max_depth, 0, &mut results);
    results
}

fn walkdir_simple_inner(
    dir: &Path,
    max_depth: usize,
    current_depth: usize,
    results: &mut Vec<PathBuf>,
) {
    if current_depth > max_depth {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walkdir_simple_inner(&path, max_depth, current_depth + 1, results);
            } else {
                results.push(path);
            }
        }
    }
}

// ============================================================================
// PAK discovery
// ============================================================================

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
    let mut seen_hashes = HashSet::new();

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
            // Try to parse - check for LSOF magic (LSF format) first
            let parsed = if content.len() >= 4 && &content[0..4] == b"LSOF" {
                // LSF format - parse directly
                parse_lsf_bytes(&content)
                    .ok()
                    .and_then(|doc| parse_vtex_config_from_lsf(&doc))
                    .map(|lsf| {
                        (
                            lsf.tileset_name,
                            lsf.virtual_textures_path,
                            lsf.texture_names,
                        )
                    })
            } else {
                // Plain XML format
                String::from_utf8(content)
                    .ok()
                    .and_then(|s| xml_from_str::<VTexConfigXml>(&s).ok())
                    .map(|xml| {
                        let tileset_name = xml.name;
                        let vt_path = xml.paths.and_then(|p| p.virtual_textures);
                        let texture_names = xml
                            .textures
                            .map(|t| t.textures.into_iter().map(|tex| tex.name).collect())
                            .unwrap_or_default();
                        (tileset_name, vt_path, texture_names)
                    })
            };

            if let Some((tileset_name, vt_path, texture_names)) = parsed {
                // Derive GTS path from VirtualTextures path + TileSet name
                if let Some(ref vt_path) = vt_path {
                    let vt_path_normalized = vt_path.replace('\\', "/");
                    let gts_filename = format!("{tileset_name}.gts");
                    // GTS path is relative to PAK root
                    let gts_path = PathBuf::from(&vt_path_normalized).join(&gts_filename);

                    for texture_name in texture_names {
                        seen_hashes.insert(texture_name.clone());
                        discovered.push(DiscoveredVirtualTexture {
                            mod_name: mod_name.clone(),
                            mod_root: pak_path.to_path_buf(), // PAK file is the "root"
                            tileset_name: Some(tileset_name.clone()),
                            gtex_hash: texture_name,
                            gts_path: gts_path.clone(),
                            source: DiscoverySource::VTexConfigXml,
                        });
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
pub fn discover_virtual_textures(search_paths: &[PathBuf]) -> Result<Vec<DiscoveredVirtualTexture>> {
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

        // Check if this path has GTS files directly (e.g., it IS a mod directory)
        if search_path.is_dir() {
            if let Ok(discovered) = discover_mod_virtual_textures(search_path) {
                if !discovered.is_empty() {
                    all_discovered.extend(discovered);
                    continue;
                }
            }
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
                } else if path.is_dir() {
                    // Try as mod root (with or without Mods/ subdirectory)
                    if let Ok(discovered) = discover_mod_virtual_textures(&path) {
                        all_discovered.extend(discovered);
                    }
                }
            }
        }
    }

    Ok(all_discovered)
}
