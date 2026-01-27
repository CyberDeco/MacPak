//! Virtual texture extraction and combining
//!
//! Extracts GTP tiles and combines them into full Albedo, Normal, and Physical textures.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::missing_panics_doc,
    clippy::stable_sort_primitive
)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::error::{Error, Result};
use super::gts::GtsFile;
use super::gtp::GtpFile;
use super::mod_config;
use super::types::VirtualTextureLayer;

/// DDS file writer for BC/DXT5 compressed textures
pub struct DdsWriter;

impl DdsWriter {
    const DDS_MAGIC: u32 = 0x2053_4444; // 'DDS '
    const DDS_HEADER_SIZE: u32 = 124;
    const DDSD_CAPS: u32 = 0x1;
    const DDSD_HEIGHT: u32 = 0x2;
    const DDSD_WIDTH: u32 = 0x4;
    const DDSD_PIXELFORMAT: u32 = 0x1000;
    const DDSD_LINEARSIZE: u32 = 0x80000;
    const DDPF_FOURCC: u32 = 0x4;
    const DDSCAPS_TEXTURE: u32 = 0x1000;
    const FOURCC_DXT5: u32 = 0x3554_5844; // 'DXT5'

    /// Write BC/DXT5 texture data to a DDS file
    ///
    /// # Errors
    /// Returns an error if the file cannot be created or written.
    pub fn write<P: AsRef<Path>>(path: P, data: &[u8], width: u32, height: u32) -> Result<()> {
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);

        // Calculate pitch for BC5/DXT5
        let pitch = width.div_ceil(4) * 16;
        let linear_size = pitch * height.div_ceil(4);

        // Write DDS magic
        writer.write_all(&Self::DDS_MAGIC.to_le_bytes())?;

        // Header size
        writer.write_all(&Self::DDS_HEADER_SIZE.to_le_bytes())?;

        // Flags
        let flags = Self::DDSD_CAPS | Self::DDSD_HEIGHT | Self::DDSD_WIDTH
            | Self::DDSD_PIXELFORMAT | Self::DDSD_LINEARSIZE;
        writer.write_all(&flags.to_le_bytes())?;

        // Height
        writer.write_all(&height.to_le_bytes())?;

        // Width
        writer.write_all(&width.to_le_bytes())?;

        // Linear size
        writer.write_all(&linear_size.to_le_bytes())?;

        // Depth
        writer.write_all(&0u32.to_le_bytes())?;

        // Mipmap count
        writer.write_all(&1u32.to_le_bytes())?;

        // Reserved (44 bytes)
        writer.write_all(&[0u8; 44])?;

        // Pixel format
        // Size
        writer.write_all(&32u32.to_le_bytes())?;
        // Flags
        writer.write_all(&Self::DDPF_FOURCC.to_le_bytes())?;
        // FourCC
        writer.write_all(&Self::FOURCC_DXT5.to_le_bytes())?;
        // RGB bit count
        writer.write_all(&0u32.to_le_bytes())?;
        // R mask
        writer.write_all(&0u32.to_le_bytes())?;
        // G mask
        writer.write_all(&0u32.to_le_bytes())?;
        // B mask
        writer.write_all(&0u32.to_le_bytes())?;
        // A mask
        writer.write_all(&0u32.to_le_bytes())?;

        // Caps
        writer.write_all(&Self::DDSCAPS_TEXTURE.to_le_bytes())?;
        // Caps2
        writer.write_all(&0u32.to_le_bytes())?;
        // Caps3
        writer.write_all(&0u32.to_le_bytes())?;
        // Caps4
        writer.write_all(&0u32.to_le_bytes())?;
        // Reserved2
        writer.write_all(&0u32.to_le_bytes())?;

        // Write texture data
        writer.write_all(data)?;

        writer.flush()?;
        Ok(())
    }
}

/// Options for virtual texture extraction
#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    /// Extract only these specific layers (0=BaseMap, 1=NormalMap, 2=PhysicalMap)
    /// Empty vec means all layers
    pub layers: Vec<usize>,
    /// Extract all layers with numbered suffixes (_0, _1, _2)
    pub all_layers: bool,
}

/// Virtual texture extractor
pub struct VirtualTextureExtractor;

impl VirtualTextureExtractor {
    /// Extract a GTP file to Albedo.dds, Normal.dds, and Physical.dds
    ///
    /// The GTS file is automatically found based on the GTP filename.
    /// Progress should be managed by the caller at the per-file level.
    ///
    /// # Errors
    /// Returns an error if the GTS file cannot be found or extraction fails.
    pub fn extract<P: AsRef<Path>>(
        gtp_path: P,
        output_dir: P,
    ) -> Result<()> {
        let gtp_path = gtp_path.as_ref();
        let output_dir = output_dir.as_ref();

        // Find corresponding GTS file
        let gts_path = Self::find_gts_file(gtp_path)?;

        Self::extract_with_gts(gtp_path, &gts_path, output_dir)
    }

    /// Extract a GTP file using a specific GTS file
    ///
    /// Progress should be managed by the caller at the per-file level.
    ///
    /// # Errors
    /// Returns an error if the files cannot be read or extraction fails.
    ///
    /// # Panics
    /// This function does not panic under normal conditions.
    pub fn extract_with_gts<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(
        gtp_path: P1,
        gts_path: P2,
        output_dir: P3,
    ) -> Result<()> {
        Self::extract_with_options(gtp_path, gts_path, output_dir, &ExtractOptions::default())
    }

    /// Extract a GTP file with specific options (layer filtering, etc.)
    ///
    /// # Errors
    /// Returns an error if the files cannot be read or extraction fails.
    pub fn extract_with_options<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(
        gtp_path: P1,
        gts_path: P2,
        output_dir: P3,
        options: &ExtractOptions,
    ) -> Result<()> {
        let gtp_path = gtp_path.as_ref();
        let gts_path = gts_path.as_ref();
        let output_dir = output_dir.as_ref();

        // Parse GTS file
        let gts = GtsFile::open(gts_path)?;

        // Open GTP file
        let mut gtp = GtpFile::open(gtp_path, &gts)?;

        // Extract GTP filename for matching
        let gtp_name = gtp_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::ConversionError("Invalid GTP filename".to_string()))?;

        // Find page file index and gather naming info
        // Format: {mod_name}_{gtex_hash}_[Layer].dds
        let (page_file_idx, mod_name, gtex_hash): (u16, String, String) =
            if let Some(hash) = Self::extract_hash_from_filename(gtp_name) {
                // BG3 vanilla: hash in filename (e.g., "Name_<32hexhash>.gtp")
                let idx = gts.find_page_file_index(hash)
                    .ok_or_else(|| Error::ConversionError(
                        format!("Hash '{hash}' not found in GTS metadata")
                    ))?;
                // Extract base name before the hash
                let base_name = gtp_name
                    .strip_suffix(".gtp")
                    .and_then(|n| n.rfind('_').map(|pos| &n[..pos]))
                    .unwrap_or("BG3")
                    .to_string();
                (idx, base_name, hash.to_string())
            } else {
                // Mod: read config files and match by filename
                let config = mod_config::load_mod_config(gtp_path);
                let idx = gts.find_page_file_index_by_name(gtp_name)
                    .ok_or_else(|| Error::ConversionError(
                        format!("GTP file '{gtp_name}' not found in GTS metadata")
                    ))?;

                // Get name and GTex hash from config files
                let (name, gtex_hash) = if let Some(ref cfg) = config {
                    // Use TileSet name from VTexConfig.xml, or mod_name as fallback
                    let name = cfg.tileset_name.clone()
                        .unwrap_or_else(|| cfg.mod_name.clone());

                    // Get GTex hash: first from VTexConfig.xml, then from VirtualTextures.json
                    let hash = cfg.gtex_hashes.first().cloned()
                        .or_else(|| {
                            // Fallback: find in VirtualTextures.json by GTS path
                            let gts_filename = gts_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("");
                            cfg.mappings.iter()
                                .find(|m| m.gts_path.ends_with(gts_filename))
                                .map(|m| m.gtex_name.clone())
                        })
                        .unwrap_or_else(|| "unknown".to_string());

                    tracing::debug!(
                        "Loaded mod config: tileset='{}', mod='{}', gtex_hash='{}'",
                        cfg.tileset_name.as_deref().unwrap_or("none"),
                        cfg.mod_name,
                        hash
                    );
                    (name, hash)
                } else {
                    // Fallback: use GTP stem as name
                    let stem = gtp_name.strip_suffix(".gtp").unwrap_or(gtp_name);
                    (stem.to_string(), "unknown".to_string())
                };

                (idx, name, gtex_hash)
            };

        // Get tile locations for each layer
        let tiles_by_layer = gts.get_tiles_for_page_file(page_file_idx);

        // Create output directory
        std::fs::create_dir_all(output_dir)?;

        // BC block dimensions
        let tile_width = gts.header.tile_width as usize;
        let tile_height = gts.header.tile_height as usize;
        let tile_border = gts.header.tile_border as usize;
        let content_width = gts.content_width() as usize;
        let content_height = gts.content_height() as usize;

        let src_block_width = tile_width.div_ceil(4);
        let src_block_height = tile_height.div_ceil(4);
        let bytes_per_block = 16;
        let tile_bc_size = src_block_width * src_block_height * bytes_per_block;

        let border_blocks = tile_border / 4;
        let content_block_width = content_width.div_ceil(4);
        let content_block_height = content_height.div_ceil(4);

        // Determine which layers to extract based on options
        let layers_to_extract: Vec<usize> = if options.layers.is_empty() {
            vec![0, 1, 2]
        } else {
            // Validate and deduplicate layers
            let mut layers = options.layers.clone();
            layers.sort();
            layers.dedup();
            for &layer in &layers {
                if layer >= 3 {
                    return Err(Error::ConversionError(format!(
                        "Invalid layer index: {layer}. Must be 0 (BaseMap), 1 (NormalMap), or 2 (PhysicalMap)"
                    )));
                }
            }
            layers
        };

        // Process selected layers
        for layer_idx in layers_to_extract {
            let layer = VirtualTextureLayer::from_index(layer_idx as u8)
                .ok_or_else(|| Error::ConversionError("Invalid layer index".to_string()))?;

            let tiles = &tiles_by_layer[layer_idx];

            if tiles.is_empty() {
                continue;
            }

            // Find bounds
            let min_x = tiles.iter().map(|t| t.x).min().expect("tiles is non-empty") as usize;
            let max_x = tiles.iter().map(|t| t.x).max().expect("tiles is non-empty") as usize;
            let min_y = tiles.iter().map(|t| t.y).min().expect("tiles is non-empty") as usize;
            let max_y = tiles.iter().map(|t| t.y).max().expect("tiles is non-empty") as usize;

            let width_tiles = max_x - min_x + 1;
            let height_tiles = max_y - min_y + 1;

            // Output dimensions (content size, no border)
            let output_width = width_tiles * content_width;
            let output_height = height_tiles * content_height;
            let output_block_width = width_tiles * content_block_width;
            let output_block_height = height_tiles * content_block_height;
            let output_bc_size = output_block_width * output_block_height * bytes_per_block;

            let mut output_data = vec![0u8; output_bc_size];

            // Extract and place each tile
            for tile in tiles {
                // Extract tile
                let tile_data = match gtp.extract_chunk(
                    tile.page as usize,
                    tile.chunk as usize,
                    &gts,
                ) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Warning: Failed to extract chunk: {e}");
                        continue;
                    }
                };

                if tile_data.len() < tile_bc_size {
                    continue;
                }

                let out_x = tile.x as usize - min_x;
                let out_y = tile.y as usize - min_y;

                // Copy content blocks only (skip border)
                for content_row in 0..content_block_height {
                    let src_row = border_blocks + content_row;
                    let src_offset = (src_row * src_block_width + border_blocks) * bytes_per_block;

                    let dst_block_row = out_y * content_block_height + content_row;
                    let dst_block_col = out_x * content_block_width;
                    let dst_offset = (dst_block_row * output_block_width + dst_block_col) * bytes_per_block;

                    let row_size = content_block_width * bytes_per_block;

                    if src_offset + row_size <= tile_data.len()
                        && dst_offset + row_size <= output_data.len()
                    {
                        output_data[dst_offset..dst_offset + row_size]
                            .copy_from_slice(&tile_data[src_offset..src_offset + row_size]);
                    }
                }
            }

            // Write DDS file: {mod_name}_{gtex_hash}_{Layer}.dds
            let filename = if options.all_layers {
                format!("{}_{}_{}.dds", mod_name, gtex_hash, layer_idx)
            } else {
                format!("{}_{}_{}.dds", mod_name, gtex_hash, layer.as_str())
            };
            let output_path = output_dir.join(filename);
            DdsWriter::write(&output_path, &output_data, output_width as u32, output_height as u32)?;
        }

        Ok(())
    }

    /// Extract the 32-character hash from a GTP filename
    /// Format: "`SomeName`_<32hexchars>.gtp"
    fn extract_hash_from_filename(filename: &str) -> Option<&str> {
        // Must end with .gtp
        let name = filename.strip_suffix(".gtp")?;

        // Find last underscore
        let underscore_pos = name.rfind('_')?;

        // Hash should be 32 characters after the underscore
        let hash = &name[underscore_pos + 1..];

        // Validate it's a 32-char hex string
        if hash.len() == 32 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(hash)
        } else {
            None
        }
    }

    /// Find the corresponding GTS file for a GTP file
    fn find_gts_file(gtp_path: &Path) -> Result<std::path::PathBuf> {
        let gtp_name = gtp_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::ConversionError("Invalid GTP filename".to_string()))?;

        let gtp_dir = gtp_path.parent()
            .ok_or_else(|| Error::ConversionError("No parent directory".to_string()))?;

        // Extract base name (e.g., "Albedo_Normal_Physical_1" from "Albedo_Normal_Physical_1_xxx.gtp")
        let base_name = if let Some(pos) = gtp_name.rfind('_') {
            let suffix = &gtp_name[pos + 1..];
            if suffix.to_lowercase().ends_with(".gtp") && suffix.len() > 4 {
                &gtp_name[..pos]
            } else {
                gtp_name.trim_end_matches(".gtp")
            }
        } else {
            gtp_name.trim_end_matches(".gtp")
        };

        // Try exact match first
        let exact_gts = gtp_dir.join(format!("{base_name}.gts"));
        if exact_gts.exists() {
            return Ok(exact_gts);
        }

        // Try to find any .gts file in the directory
        if let Ok(entries) = std::fs::read_dir(gtp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("gts") {
                    return Ok(path);
                }
            }
        }

        Err(Error::ConversionError(format!(
            "Could not find GTS file for {gtp_name}"
        )))
    }
}

