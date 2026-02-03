//! Type definitions for mod configuration files
//!
//!

use serde::Deserialize;
use std::path::PathBuf;

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
    /// Discovered by scanning for GTS files (tertiary fallback)
    GtsFileScan,
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
