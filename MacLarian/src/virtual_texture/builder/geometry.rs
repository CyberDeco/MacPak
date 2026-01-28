//! Geometry calculation for virtual texture tiling
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

use super::config::TileSetConfiguration;

/// Calculated texture layout in virtual space
#[derive(Debug, Clone)]
pub struct TextureLayout {
    /// Texture index in the source list
    pub texture_idx: usize,
    /// Texture name
    pub name: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// X position in virtual texture space (in pixels)
    pub x: u32,
    /// Y position in virtual texture space (in pixels)
    pub y: u32,
    /// Number of mip levels
    pub mip_levels: u32,
}

/// Calculated tile coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileCoord {
    /// Layer index (0=BaseMap, 1=NormalMap, 2=PhysicalMap)
    pub layer: u8,
    /// Mip level (0 = highest resolution)
    pub level: u8,
    /// Tile X coordinate in grid
    pub x: u16,
    /// Tile Y coordinate in grid
    pub y: u16,
}

impl TileCoord {
    /// Create a new tile coordinate
    #[must_use]
    pub const fn new(layer: u8, level: u8, x: u16, y: u16) -> Self {
        Self { layer, level, x, y }
    }

    /// Encode as packed u32 for GTS file
    #[must_use]
    pub fn to_packed_id(&self) -> u32 {
        ((self.x as u32) << 20)
            | ((self.y as u32) << 8)
            | ((self.level as u32) << 4)
            | (self.layer as u32)
    }
}

/// Level information for the tile set
#[derive(Debug, Clone)]
pub struct LevelInfo {
    /// Mip level index
    pub level: u8,
    /// Width in tiles
    pub width_tiles: u32,
    /// Height in tiles
    pub height_tiles: u32,
    /// Width in pixels at this level
    pub width_pixels: u32,
    /// Height in pixels at this level
    pub height_pixels: u32,
}

/// Geometry calculation result
#[derive(Debug, Clone)]
pub struct GeometryResult {
    /// Total virtual texture width in pixels
    pub total_width: u32,
    /// Total virtual texture height in pixels
    pub total_height: u32,
    /// Texture layouts
    pub textures: Vec<TextureLayout>,
    /// Level information for each mip level
    pub levels: Vec<LevelInfo>,
    /// All tile coordinates that need to be generated (per layer)
    pub tiles_per_layer: [Vec<TileCoord>; 3],
}

/// Calculate the geometry for a set of textures
///
/// # Arguments
/// * `textures` - List of (name, width, height) for each texture
/// * `layers_present` - Which layers are present [base, normal, physical]
/// * `config` - Tile set configuration
/// * `max_mip_levels` - Maximum mip levels to generate (None = calculate from dimensions)
pub fn calculate_geometry(
    textures: &[(String, u32, u32)],
    layers_present: [bool; 3],
    config: &TileSetConfiguration,
    max_mip_levels: Option<u32>,
) -> GeometryResult {
    if textures.is_empty() {
        return GeometryResult {
            total_width: 0,
            total_height: 0,
            textures: Vec::new(),
            levels: Vec::new(),
            tiles_per_layer: [Vec::new(), Vec::new(), Vec::new()],
        };
    }

    let raw_tile_width = config.raw_tile_width();
    let raw_tile_height = config.raw_tile_height();

    // For simplicity, use the first texture's dimensions as the virtual texture size
    // A more sophisticated implementation would pack multiple textures
    let (name, tex_width, tex_height) = &textures[0];

    // Use actual texture dimensions (don't pad) - tiles will cover partial edges
    let total_width = *tex_width;
    let total_height = *tex_height;

    // Calculate number of mip levels
    let calculated_mips = calculate_mip_levels(total_width, total_height, raw_tile_width.min(raw_tile_height));
    let mip_levels = max_mip_levels.map_or(calculated_mips, |max| max.min(calculated_mips));

    // Create texture layout
    let layout = TextureLayout {
        texture_idx: 0,
        name: name.clone(),
        width: *tex_width,
        height: *tex_height,
        x: 0,
        y: 0,
        mip_levels,
    };

    // Calculate levels and tiles
    let mut levels = Vec::with_capacity(mip_levels as usize);
    let mut tiles_per_layer: [Vec<TileCoord>; 3] = [Vec::new(), Vec::new(), Vec::new()];

    let mut level_width = total_width;
    let mut level_height = total_height;

    for level in 0..mip_levels {
        let width_tiles = tiles_for_dimension(level_width, raw_tile_width);
        let height_tiles = tiles_for_dimension(level_height, raw_tile_height);

        levels.push(LevelInfo {
            level: level as u8,
            width_tiles,
            height_tiles,
            width_pixels: level_width,
            height_pixels: level_height,
        });

        // Generate tile coordinates for each layer at this level
        for (layer_idx, present) in layers_present.iter().enumerate() {
            if *present {
                for ty in 0..height_tiles {
                    for tx in 0..width_tiles {
                        tiles_per_layer[layer_idx].push(TileCoord::new(
                            layer_idx as u8,
                            level as u8,
                            tx as u16,
                            ty as u16,
                        ));
                    }
                }
            }
        }

        // Next mip level is half the size
        level_width = (level_width / 2).max(1);
        level_height = (level_height / 2).max(1);

        // Stop if we're smaller than a tile
        if level_width < raw_tile_width && level_height < raw_tile_height {
            break;
        }
    }

    GeometryResult {
        total_width,
        total_height,
        textures: vec![layout],
        levels,
        tiles_per_layer,
    }
}

/// Calculate the number of mip levels for given dimensions
#[must_use]
pub fn calculate_mip_levels(width: u32, height: u32, min_size: u32) -> u32 {
    let max_dim = width.max(height);
    let min_dim = min_size.max(1);

    // Count how many times mip levels can be halved before reaching min_size
    let mut levels = 1u32;
    let mut dim = max_dim;
    while dim > min_dim {
        dim /= 2;
        levels += 1;
    }

    levels
}

/// Calculate the number of tiles needed for a given dimension
#[must_use]
pub fn tiles_for_dimension(pixels: u32, tile_size: u32) -> u32 {
    pixels.div_ceil(tile_size)
}

