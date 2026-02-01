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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pak::PakOperations;
    use quick_xml::de::from_str as xml_from_str;
    use std::path::PathBuf;

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

        let config: VirtualTexturesJson =
            serde_json::from_str(json).expect("Failed to parse JSON");

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

    #[test]
    fn test_discover_pak_virtual_textures() {
        // Test with the original mod PAK
        let pak_path = PathBuf::from(
            "/Users/corrine/Documents/Larian Studios/Baldur's Gate 3/Mods/Medusae_Outfits.pak",
        );
        if !pak_path.exists() {
            eprintln!("Skipping test: Medusae_Outfits.pak not found");
            return;
        }

        // Read the VTexConfig.xml bytes directly from PAK
        let vtex_config_path = "Mods/Medusae_Outfits/VTexConfig.xml";
        let bytes = PakOperations::read_file_bytes(&pak_path, vtex_config_path)
            .expect("Should be able to read VTexConfig.xml from PAK");

        // Verify it's valid XML (starts with <?xml or just <)
        assert!(
            bytes.starts_with(b"<?xml") || bytes.starts_with(b"<"),
            "VTexConfig.xml should be valid XML, got: {:?}",
            &bytes[..bytes.len().min(20)]
        );

        // Test the discovery function
        let discovered =
            discover_pak_virtual_textures(&pak_path).expect("Discovery should not fail");

        assert!(
            !discovered.is_empty(),
            "Should discover at least one texture from PAK"
        );

        let first = &discovered[0];
        assert_eq!(first.mod_name, "Medusae_Outfits");
        assert_eq!(first.gtex_hash, "2d1fe47f16484210a68d71256662a676");
        assert_eq!(
            first.tileset_name,
            Some("Medusae_Outfits_Textures".to_string())
        );
        assert_eq!(first.source, DiscoverySource::VTexConfigXml);
    }
}
