//! Tile extraction and processing
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

use crate::error::{Error, Result};
use super::config::TileSetConfiguration;
use super::geometry::TileCoord;
use std::path::Path;

/// A processed tile ready for compression
#[derive(Debug, Clone)]
pub struct ProcessedTile {
    /// Tile coordinate information
    pub coord: TileCoord,
    /// Packed tile ID (layer|level|y|x encoded)
    pub packed_id: u32,
    /// Raw tile data (BC-compressed pixels with borders)
    pub data: Vec<u8>,
    /// Embedded mip data (if enabled)
    pub mip_data: Option<Vec<u8>>,
}

impl ProcessedTile {
    /// Get the full tile data including embedded mip
    #[must_use]
    pub fn full_data(&self) -> Vec<u8> {
        let mut result = self.data.clone();
        if let Some(ref mip) = self.mip_data {
            result.extend_from_slice(mip);
        }
        result
    }
}

/// DDS texture data loaded for tile extraction
pub struct DdsTexture {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// BC block size (8 for BC1, 16 for BC3/BC5/BC7)
    pub block_size: usize,
    /// Raw BC-compressed data
    pub data: Vec<u8>,
    /// Number of mip levels
    pub mip_count: u32,
    /// Offsets to each mip level in the data
    pub mip_offsets: Vec<usize>,
}

impl DdsTexture {
    /// Load a DDS texture from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path.as_ref())?;
        Self::from_bytes(&data)
    }

    /// Parse a DDS texture from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        use ddsfile::Dds;

        let dds = Dds::read(std::io::Cursor::new(data))
            .map_err(|e| Error::DdsError(format!("Failed to parse DDS: {e}")))?;

        let width = dds.get_width();
        let height = dds.get_height();
        let mip_count = dds.get_num_mipmap_levels();

        // Determine block size from format
        let block_size = match dds.get_dxgi_format() {
            Some(ddsfile::DxgiFormat::BC1_UNorm | ddsfile::DxgiFormat::BC1_UNorm_sRGB) => 8,
            Some(ddsfile::DxgiFormat::BC3_UNorm | ddsfile::DxgiFormat::BC3_UNorm_sRGB) => 16,
            Some(ddsfile::DxgiFormat::BC5_UNorm | ddsfile::DxgiFormat::BC5_SNorm) => 16,
            Some(ddsfile::DxgiFormat::BC7_UNorm | ddsfile::DxgiFormat::BC7_UNorm_sRGB) => 16,
            _ => {
                // Try to infer from D3D format
                match dds.get_d3d_format() {
                    Some(ddsfile::D3DFormat::DXT1) => 8,
                    Some(ddsfile::D3DFormat::DXT3 | ddsfile::D3DFormat::DXT5) => 16,
                    _ => return Err(Error::DdsError("Unsupported DDS format".to_string())),
                }
            }
        };

        // Calculate mip offsets
        let mut mip_offsets = Vec::with_capacity(mip_count as usize);
        let mut offset = 0usize;
        let mut mip_width = width;
        let mut mip_height = height;

        for _ in 0..mip_count {
            mip_offsets.push(offset);

            let blocks_wide = mip_width.div_ceil(4);
            let blocks_high = mip_height.div_ceil(4);
            let mip_size = (blocks_wide * blocks_high) as usize * block_size;
            offset += mip_size;

            mip_width = (mip_width / 2).max(1);
            mip_height = (mip_height / 2).max(1);
        }

        Ok(Self {
            width,
            height,
            block_size,
            data: dds.get_data(0).map_err(|e| Error::DdsError(format!("{e}")))?.to_vec(),
            mip_count,
            mip_offsets,
        })
    }

    /// Get the data for a specific mip level
    pub fn get_mip_data(&self, level: u32) -> Option<(&[u8], u32, u32)> {
        if level >= self.mip_count {
            return None;
        }

        let start = self.mip_offsets[level as usize];
        let mip_width = (self.width >> level).max(1);
        let mip_height = (self.height >> level).max(1);

        let blocks_wide = mip_width.div_ceil(4);
        let blocks_high = mip_height.div_ceil(4);
        let mip_size = (blocks_wide * blocks_high) as usize * self.block_size;

        let end = start + mip_size;
        if end <= self.data.len() {
            Some((&self.data[start..end], mip_width, mip_height))
        } else {
            None
        }
    }
}

/// Extract a tile from BC-compressed source with proper border handling
///
/// This extracts a tile where the content starts at (`content_x`, `content_y`) and
/// borders sample from adjacent pixels (or clamp at texture edges).
///
/// # Arguments
/// * `src_data` - Source BC-compressed data
/// * `src_width` - Source texture width in pixels
/// * `src_height` - Source texture height in pixels
/// * `content_x` - Content area X position in source (pixels)
/// * `content_y` - Content area Y position in source (pixels)
/// * `content_width` - Content area width (pixels, typically 128)
/// * `content_height` - Content area height (pixels)
/// * `border` - Border size in pixels (typically 8)
/// * `block_size` - BC block size (8 or 16 bytes)
fn extract_tile_with_borders(
    src_data: &[u8],
    src_width: u32,
    src_height: u32,
    content_x: u32,
    content_y: u32,
    content_width: u32,
    content_height: u32,
    border: u32,
    block_size: usize,
) -> Vec<u8> {
    let padded_width = content_width + 2 * border;
    let padded_height = content_height + 2 * border;

    let src_blocks_wide = src_width.div_ceil(4);
    let src_blocks_high = src_height.div_ceil(4);
    let tile_blocks_wide = padded_width.div_ceil(4);
    let tile_blocks_high = padded_height.div_ceil(4);
    let border_blocks = border / 4;

    let mut tile = Vec::with_capacity((tile_blocks_wide * tile_blocks_high) as usize * block_size);

    // Content starts at block position
    let content_block_x = content_x / 4;
    let content_block_y = content_y / 4;

    for tile_by in 0..tile_blocks_high {
        for tile_bx in 0..tile_blocks_wide {
            // Calculate source block position
            // Tile block (0,0) should map to content_block - border_blocks
            let rel_bx = tile_bx as i32 - border_blocks as i32;
            let rel_by = tile_by as i32 - border_blocks as i32;

            let src_bx = (content_block_x as i32 + rel_bx)
                .max(0)
                .min(src_blocks_wide as i32 - 1) as u32;
            let src_by = (content_block_y as i32 + rel_by)
                .max(0)
                .min(src_blocks_high as i32 - 1) as u32;

            let src_offset = ((src_by * src_blocks_wide + src_bx) as usize) * block_size;

            if src_offset + block_size <= src_data.len() {
                tile.extend_from_slice(&src_data[src_offset..src_offset + block_size]);
            } else {
                tile.resize(tile.len() + block_size, 0);
            }
        }
    }

    tile
}

/// Extract a tile from BC-compressed source data with edge clamping (simple version)
///
/// # Arguments
/// * `src_data` - Source BC-compressed data
/// * `src_width` - Source texture width in pixels
/// * `src_height` - Source texture height in pixels
/// * `tile_x` - Tile X position in pixels
/// * `tile_y` - Tile Y position in pixels
/// * `tile_width` - Tile width in pixels
/// * `tile_height` - Tile height in pixels
/// * `block_size` - BC block size (8 or 16 bytes)
pub fn extract_bc_tile_with_clamp(
    src_data: &[u8],
    src_width: u32,
    src_height: u32,
    tile_x: u32,
    tile_y: u32,
    tile_width: u32,
    tile_height: u32,
    block_size: usize,
) -> Vec<u8> {
    let src_blocks_wide = src_width.div_ceil(4);
    let src_blocks_high = src_height.div_ceil(4);
    let tile_blocks_wide = tile_width.div_ceil(4);
    let tile_blocks_high = tile_height.div_ceil(4);

    // Tile position in blocks
    let tile_block_x = tile_x / 4;
    let tile_block_y = tile_y / 4;

    let mut tile = Vec::with_capacity((tile_blocks_wide * tile_blocks_high) as usize * block_size);

    for by in 0..tile_blocks_high {
        for bx in 0..tile_blocks_wide {
            // Calculate source block position, clamping to texture bounds
            let src_block_x = (tile_block_x + bx).min(src_blocks_wide.saturating_sub(1));
            let src_block_y = (tile_block_y + by).min(src_blocks_high.saturating_sub(1));

            let src_offset = ((src_block_y * src_blocks_wide + src_block_x) as usize) * block_size;

            if src_offset + block_size <= src_data.len() {
                tile.extend_from_slice(&src_data[src_offset..src_offset + block_size]);
            } else {
                // Out of data bounds, fill with zeros
                tile.resize(tile.len() + block_size, 0);
            }
        }
    }

    tile
}

/// Extract a tile from BC-compressed source data (simple version without clamping)
///
/// # Arguments
/// * `src_data` - Source BC-compressed data
/// * `src_width` - Source texture width in pixels
/// * `tile_x` - Tile X position in pixels
/// * `tile_y` - Tile Y position in pixels
/// * `tile_width` - Tile width in pixels
/// * `tile_height` - Tile height in pixels
/// * `block_size` - BC block size (8 or 16 bytes)
pub fn extract_bc_tile(
    src_data: &[u8],
    src_width: u32,
    tile_x: u32,
    tile_y: u32,
    tile_width: u32,
    tile_height: u32,
    block_size: usize,
) -> Vec<u8> {
    let src_blocks_wide = src_width.div_ceil(4);
    let tile_blocks_wide = tile_width.div_ceil(4);
    let tile_blocks_high = tile_height.div_ceil(4);

    // Tile position in blocks
    let tile_block_x = tile_x / 4;
    let tile_block_y = tile_y / 4;

    let mut tile = Vec::with_capacity((tile_blocks_wide * tile_blocks_high) as usize * block_size);

    for by in 0..tile_blocks_high {
        let src_block_row = tile_block_y + by;
        let src_block_col = tile_block_x;

        // Check bounds
        if src_block_row >= src_width.div_ceil(4) {
            // Out of bounds, fill with zeros
            tile.resize(tile.len() + (tile_blocks_wide as usize * block_size), 0);
            continue;
        }

        let src_offset = ((src_block_row * src_blocks_wide + src_block_col) as usize) * block_size;
        let row_bytes = tile_blocks_wide as usize * block_size;

        if src_offset + row_bytes <= src_data.len() {
            tile.extend_from_slice(&src_data[src_offset..src_offset + row_bytes]);
        } else if src_offset < src_data.len() {
            // Partial row
            tile.extend_from_slice(&src_data[src_offset..]);
            tile.resize(tile.len() + (row_bytes - (src_data.len() - src_offset)), 0);
        } else {
            // Completely out of bounds
            tile.resize(tile.len() + row_bytes, 0);
        }
    }

    tile
}

/// Extract all tiles from a DDS texture for a given layer
pub fn extract_tiles_from_dds(
    dds: &DdsTexture,
    coords: &[TileCoord],
    config: &TileSetConfiguration,
) -> Result<Vec<ProcessedTile>> {
    let raw_tile_width = config.raw_tile_width();
    let raw_tile_height = config.raw_tile_height();
    let border = config.tile_border;

    let mut tiles = Vec::with_capacity(coords.len());

    for coord in coords {
        let level = coord.level as u32;

        // Get mip level data
        let (mip_data, mip_width, mip_height) = dds.get_mip_data(level)
            .ok_or_else(|| Error::VirtualTexture(
                format!("Mip level {level} not available in texture")
            ))?;

        // Calculate tile content position in the source texture
        let content_pixel_x = (coord.x as u32) * raw_tile_width;
        let content_pixel_y = (coord.y as u32) * raw_tile_height;

        // Extract full tile with proper border handling
        // The tile includes border pixels that sample from adjacent content
        // For edge tiles, borders are clamped to edge pixels
        let tile_data = extract_tile_with_borders(
            mip_data,
            mip_width,
            mip_height,
            content_pixel_x,
            content_pixel_y,
            raw_tile_width,
            raw_tile_height,
            border,
            dds.block_size,
        );

        // Handle embedded mip if enabled
        let mip_data = if config.embed_mip && level + 1 < dds.mip_count {
            // Extract a quarter-size tile from the next mip level
            let next_mip = dds.get_mip_data(level + 1);
            if let Some((next_data, next_width, next_height)) = next_mip {
                // Mip level content is at half the position
                let mip_content_x = content_pixel_x / 2;
                let mip_content_y = content_pixel_y / 2;
                let mip_content_width = raw_tile_width / 2;
                let mip_content_height = raw_tile_height / 2;
                let mip_border = border / 2;

                let mip_tile = extract_tile_with_borders(
                    next_data,
                    next_width,
                    next_height,
                    mip_content_x,
                    mip_content_y,
                    mip_content_width,
                    mip_content_height,
                    mip_border,
                    dds.block_size,
                );
                Some(mip_tile)
            } else {
                None
            }
        } else {
            None
        };

        tiles.push(ProcessedTile {
            coord: *coord,
            packed_id: coord.to_packed_id(),
            data: tile_data,
            mip_data,
        });
    }

    Ok(tiles)
}

