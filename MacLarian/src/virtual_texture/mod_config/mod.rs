//! Mod configuration file handlers for virtual textures
//!
//!
//!
//! This module handles parsing mod configuration files that map `GTex` hashes
//! to GTS paths:
//! - `VTexConfig.xml` (primary) - has `TileSet` name, paths, and texture definitions
//! - `VirtualTextures.json` (Script Extender, fallback) - has `GTex` hash → GTS path mapping

mod discovery;
mod loaders;
mod lookup;
mod types;

// Re-export types
pub use types::{
    DiscoveredVirtualTexture, DiscoverySource, ModConfig, VTexConfigPaths, VTexConfigTexture,
    VTexConfigTextures, VTexConfigXml, VTexMapping, VirtualTexturesJson,
};

// Re-export loaders
pub use loaders::{
    find_mod_name_from_mods_dir, find_mod_root, load_mod_config, load_virtual_textures_json,
    load_vtex_config_xml, parse_vtex_config_from_lsf, LsfVTexConfig,
};

// Re-export discovery functions
pub use discovery::{
    discover_mod_virtual_textures, discover_pak_virtual_textures, discover_virtual_textures,
};

// Re-export lookup functions
pub use lookup::{extract_by_gtex, find_gts_for_gtex, find_virtual_texture};

