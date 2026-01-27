//! GTS file writer
//!
//! SPDX-FileCopyrightText: 2025 CyberDeco
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use std::io::{Write, Seek, SeekFrom};
use crate::error::Result;
use crate::virtual_texture::types::{
    GtsHeader, GtsBCParameterBlock, GtsFlatTileInfo,
};
use super::fourcc::FourCCTree;
use byteorder::{LittleEndian, WriteBytesExt};

/// Layer information for GTS file
#[derive(Debug, Clone)]
pub struct LayerInfo {
    /// Data type code (e.g., 6 for BC3/DXT5)
    pub data_type: u32,
}

/// Level information for GTS file
#[derive(Debug, Clone)]
pub struct LevelInfo {
    /// Width in tiles
    pub width: u32,
    /// Height in tiles
    pub height: u32,
    /// Actual width in pixels (for non-tile-aligned textures)
    pub width_pixels: u32,
    /// Actual height in pixels (for non-tile-aligned textures)
    pub height_pixels: u32,
}

/// Page file information
#[derive(Debug, Clone)]
pub struct PageFileInfo {
    /// Filename (will be encoded as UTF-16LE)
    pub filename: String,
    /// Number of pages in this file
    pub num_pages: u32,
    /// GUID for validation
    pub guid: [u8; 16],
}

/// GTS file writer
pub struct GtsWriter {
    guid: [u8; 16],
    tile_width: i32,
    tile_height: i32,
    tile_border: i32,
    page_size: u32,
    layers: Vec<LayerInfo>,
    levels: Vec<LevelInfo>,
    parameter_blocks: Vec<GtsBCParameterBlock>,
    page_files: Vec<PageFileInfo>,
    packed_tile_ids: Vec<u32>,
    flat_tile_infos: Vec<GtsFlatTileInfo>,
    per_level_indices: Vec<Vec<u32>>, // Flat tile indices per level
    fourcc_tree: FourCCTree,
}

impl GtsWriter {
    /// Create a new GTS writer
    #[must_use]
    pub fn new(
        guid: [u8; 16],
        tile_width: i32,
        tile_height: i32,
        tile_border: i32,
        page_size: u32,
    ) -> Self {
        Self {
            guid,
            tile_width,
            tile_height,
            tile_border,
            page_size,
            layers: Vec::new(),
            levels: Vec::new(),
            parameter_blocks: Vec::new(),
            page_files: Vec::new(),
            packed_tile_ids: Vec::new(),
            flat_tile_infos: Vec::new(),
            per_level_indices: Vec::new(),
            fourcc_tree: FourCCTree::new(),
        }
    }

    /// Add a layer
    pub fn add_layer(&mut self, layer: LayerInfo) {
        self.layers.push(layer);
    }

    /// Add a level
    pub fn add_level(&mut self, level: LevelInfo) {
        self.levels.push(level);
        self.per_level_indices.push(Vec::new());
    }

    /// Add a parameter block
    pub fn add_parameter_block(&mut self, block: GtsBCParameterBlock) {
        self.parameter_blocks.push(block);
    }

    /// Add a page file
    pub fn add_page_file(&mut self, info: PageFileInfo) {
        self.page_files.push(info);
    }

    /// Add a packed tile ID
    pub fn add_packed_tile_id(&mut self, packed_id: u32) -> u32 {
        let index = self.packed_tile_ids.len() as u32;
        self.packed_tile_ids.push(packed_id);
        index
    }

    /// Add a flat tile info
    pub fn add_flat_tile_info(&mut self, info: GtsFlatTileInfo, level: usize) {
        let index = self.flat_tile_infos.len() as u32;
        self.flat_tile_infos.push(info);

        // Add to per-level index
        if level < self.per_level_indices.len() {
            self.per_level_indices[level].push(index);
        }
    }

    /// Set the FourCC metadata tree
    pub fn set_fourcc_tree(&mut self, tree: FourCCTree) {
        self.fourcc_tree = tree;
    }

    /// Write the GTS file
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // Write placeholder header
        let header_pos = writer.stream_position()?;
        let placeholder_header = [0u8; 156];
        writer.write_all(&placeholder_header)?;

        // Write layers
        let layers_offset = writer.stream_position()?;
        for layer in &self.layers {
            writer.write_u32::<LittleEndian>(layer.data_type)?;
            writer.write_i32::<LittleEndian>(-1)?; // B field, always -1
        }

        // Write per-level flat tile indices
        let mut level_offsets = Vec::new();
        for indices in &self.per_level_indices {
            level_offsets.push(writer.stream_position()?);
            for &idx in indices {
                writer.write_u32::<LittleEndian>(idx)?;
            }
        }

        // Write levels
        let levels_offset = writer.stream_position()?;
        for (i, level) in self.levels.iter().enumerate() {
            writer.write_u32::<LittleEndian>(level.width)?;
            writer.write_u32::<LittleEndian>(level.height)?;
            writer.write_u64::<LittleEndian>(level_offsets.get(i).copied().unwrap_or(0))?;
            // Extended fields: actual pixel dimensions (not in original BG3 format)
            writer.write_u32::<LittleEndian>(level.width_pixels)?;
            writer.write_u32::<LittleEndian>(level.height_pixels)?;
        }

        // Write parameter block headers
        let param_headers_offset = writer.stream_position()?;
        let mut param_data_positions = Vec::new();
        for (i, _block) in self.parameter_blocks.iter().enumerate() {
            writer.write_u32::<LittleEndian>(i as u32)?; // ParameterBlockID
            writer.write_u32::<LittleEndian>(9)?; // Codec = BC
            writer.write_u32::<LittleEndian>(56)?; // Size of BC param block
            param_data_positions.push(writer.stream_position()?);
            writer.write_u64::<LittleEndian>(0)?; // Placeholder for file offset
        }

        // Write parameter block data
        let mut param_block_offsets = Vec::new();
        for block in &self.parameter_blocks {
            param_block_offsets.push(writer.stream_position()?);
            self.write_bc_parameter_block(writer, block)?;
        }

        // Write page file metadata
        let page_files_offset = writer.stream_position()?;
        for page_file in &self.page_files {
            self.write_page_file_info(writer, page_file)?;
        }

        // Write FourCC metadata
        let fourcc_offset = writer.stream_position()?;
        let fourcc_size = self.fourcc_tree.write(writer)?;

        // Write thumbnails (empty for now)
        let thumbnails_offset = writer.stream_position()?;
        writer.write_u32::<LittleEndian>(0)?; // No thumbnails

        // Write packed tile IDs
        let packed_tiles_offset = writer.stream_position()?;
        for &id in &self.packed_tile_ids {
            writer.write_u32::<LittleEndian>(id)?;
        }

        // Write flat tile infos
        let flat_tiles_offset = writer.stream_position()?;
        for info in &self.flat_tile_infos {
            self.write_flat_tile_info(writer, info)?;
        }

        // Go back and update parameter block header offsets
        for (i, &pos) in param_data_positions.iter().enumerate() {
            writer.seek(SeekFrom::Start(pos))?;
            writer.write_u64::<LittleEndian>(param_block_offsets[i])?;
        }

        // Write final header
        writer.seek(SeekFrom::Start(header_pos))?;
        self.write_header(
            writer,
            layers_offset,
            levels_offset,
            param_headers_offset,
            page_files_offset,
            fourcc_offset,
            fourcc_size,
            thumbnails_offset,
            packed_tiles_offset,
            flat_tiles_offset,
        )?;

        Ok(())
    }

    /// Write the GTS header
    fn write_header<W: Write>(
        &self,
        writer: &mut W,
        layers_offset: u64,
        levels_offset: u64,
        param_headers_offset: u64,
        page_files_offset: u64,
        fourcc_offset: u64,
        fourcc_size: u32,
        thumbnails_offset: u64,
        packed_tiles_offset: u64,
        flat_tiles_offset: u64,
    ) -> Result<()> {
        writer.write_u32::<LittleEndian>(GtsHeader::MAGIC)?;
        writer.write_u32::<LittleEndian>(5)?; // Version
        writer.write_u32::<LittleEndian>(0)?; // Unused
        writer.write_all(&self.guid)?;

        writer.write_u32::<LittleEndian>(self.layers.len() as u32)?;
        writer.write_u64::<LittleEndian>(layers_offset)?;
        writer.write_u32::<LittleEndian>(self.levels.len() as u32)?;
        writer.write_u64::<LittleEndian>(levels_offset)?;

        writer.write_i32::<LittleEndian>(self.tile_width)?;
        writer.write_i32::<LittleEndian>(self.tile_height)?;
        writer.write_i32::<LittleEndian>(self.tile_border)?;
        writer.write_u32::<LittleEndian>(0)?; // i2

        writer.write_u32::<LittleEndian>(self.flat_tile_infos.len() as u32)?;
        writer.write_u64::<LittleEndian>(flat_tiles_offset)?;
        writer.write_u32::<LittleEndian>(0)?; // i6
        writer.write_u32::<LittleEndian>(0)?; // i7

        writer.write_u32::<LittleEndian>(self.packed_tile_ids.len() as u32)?;
        writer.write_u64::<LittleEndian>(packed_tiles_offset)?;

        // Unknown fields m-s
        for _ in 0..7 {
            writer.write_u32::<LittleEndian>(0)?;
        }

        writer.write_u32::<LittleEndian>(self.page_size)?;
        writer.write_u32::<LittleEndian>(self.page_files.len() as u32)?;
        writer.write_u64::<LittleEndian>(page_files_offset)?;

        writer.write_u32::<LittleEndian>(fourcc_size)?;
        writer.write_u64::<LittleEndian>(fourcc_offset)?;

        writer.write_u32::<LittleEndian>(self.parameter_blocks.len() as u32)?;
        writer.write_u64::<LittleEndian>(param_headers_offset)?;

        writer.write_u64::<LittleEndian>(thumbnails_offset)?;

        // Unknown fields xjj-xmm
        for _ in 0..4 {
            writer.write_u32::<LittleEndian>(0)?;
        }

        Ok(())
    }

    /// Write a BC parameter block (56 bytes)
    fn write_bc_parameter_block<W: Write>(&self, writer: &mut W, block: &GtsBCParameterBlock) -> Result<()> {
        writer.write_u16::<LittleEndian>(block.version)?;
        writer.write_all(&block.compression1)?;
        writer.write_all(&block.compression2)?;
        writer.write_u32::<LittleEndian>(block.b)?;
        writer.write_u8(block.c1)?;
        writer.write_u8(block.c2)?;
        writer.write_u8(block.bc_field3)?;
        writer.write_u8(block.data_type)?;
        writer.write_u16::<LittleEndian>(block.d)?;
        writer.write_u32::<LittleEndian>(block.fourcc)?;
        writer.write_u8(block.e1)?;
        writer.write_u8(block.save_mip)?;
        writer.write_u8(block.e3)?;
        writer.write_u8(block.e4)?;
        writer.write_u32::<LittleEndian>(block.f)?;
        Ok(())
    }

    /// Write page file info (536 bytes)
    fn write_page_file_info<W: Write>(&self, writer: &mut W, info: &PageFileInfo) -> Result<()> {
        // Filename as UTF-16LE, padded to 512 bytes
        let utf16: Vec<u16> = info.filename.encode_utf16().collect();
        let mut filename_bytes = [0u8; 512];
        for (i, &c) in utf16.iter().enumerate().take(255) {
            let bytes = c.to_le_bytes();
            filename_bytes[i * 2] = bytes[0];
            filename_bytes[i * 2 + 1] = bytes[1];
        }
        writer.write_all(&filename_bytes)?;

        writer.write_u32::<LittleEndian>(info.num_pages)?;
        writer.write_all(&info.guid)?;
        writer.write_u32::<LittleEndian>(2)?; // F field, always 2

        Ok(())
    }

    /// Write flat tile info (12 bytes)
    fn write_flat_tile_info<W: Write>(&self, writer: &mut W, info: &GtsFlatTileInfo) -> Result<()> {
        writer.write_u16::<LittleEndian>(info.page_file_index)?;
        writer.write_u16::<LittleEndian>(info.page_index)?;
        writer.write_u16::<LittleEndian>(info.chunk_index)?;
        writer.write_u16::<LittleEndian>(info.d)?;
        writer.write_u32::<LittleEndian>(info.packed_tile_id_index)?;
        Ok(())
    }
}

/// Create a BC parameter block with the given compression method
#[must_use]
pub fn create_bc_parameter_block(
    compression1: &[u8; 16],
    compression2: &[u8; 16],
    data_type: u8,
    fourcc: u32,
    embed_mip: bool,
) -> GtsBCParameterBlock {
    GtsBCParameterBlock {
        version: 0x238e,
        compression1: *compression1,
        compression2: *compression2,
        b: 0,
        c1: 0,
        c2: 0,
        bc_field3: 0,
        data_type,
        d: 0,
        fourcc,
        e1: 0,
        save_mip: if embed_mip { 1 } else { 0 },
        e3: 0,
        e4: 0,
        f: 0,
    }
}
