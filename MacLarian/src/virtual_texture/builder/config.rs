//! Configuration types for virtual texture building
//!
//! SPDX-FileCopyrightText: 2025 CyberDeco
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use std::path::PathBuf;

/// Compression preference for tile data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileCompressionPreference {
    /// Raw (uncompressed)
    Raw,
    /// LZ4 compression (recommended, best compatibility)
    #[default]
    Lz4,
    /// FastLZ compression (currently has compatibility issues)
    FastLZ,
    /// Automatically choose best compression (currently uses LZ4)
    Best,
}

impl TileCompressionPreference {
    /// Get compression name strings for BC parameter block
    #[must_use]
    pub fn compression_strings(&self) -> (&'static [u8; 16], &'static [u8; 16]) {
        match self {
            Self::Raw => (
                b"raw\0\0\0\0\0\0\0\0\0\0\0\0\0",
                b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            Self::Lz4 | Self::Best => (
                b"lz4\0\0\0\0\0\0\0\0\0\0\0\0\0",
                b"lz40.1.0\0\0\0\0\0\0\0\0",
            ),
            Self::FastLZ => (
                b"lz77\0\0\0\0\0\0\0\0\0\0\0\0",
                b"fastlz0.1.0\0\0\0\0\0",
            ),
        }
    }
}

/// BC format for texture layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BcFormat {
    /// BC1 (DXT1) - RGB with optional 1-bit alpha
    Bc1,
    /// BC3 (DXT5) - RGBA with interpolated alpha
    #[default]
    Bc3,
    /// BC5 - Two-channel (for normal maps)
    Bc5,
    /// BC7 - High quality RGBA
    Bc7,
}

impl BcFormat {
    /// Get the FourCC code for this format
    #[must_use]
    pub const fn fourcc(&self) -> u32 {
        match self {
            Self::Bc1 => 0x3154_5844, // 'DXT1'
            Self::Bc3 => 0x3554_5844, // 'DXT5'
            Self::Bc5 => 0x3254_4341, // 'ATI2' (BC5 uses ATI2 FourCC)
            Self::Bc7 => 0x3758_4344, // 'DX10' marker (BC7 uses DX10 extension)
        }
    }

    /// Get the bytes per BC block (4x4 pixels)
    #[must_use]
    pub const fn block_size(&self) -> usize {
        match self {
            Self::Bc1 => 8,
            Self::Bc3 | Self::Bc5 | Self::Bc7 => 16,
        }
    }
}

/// Configuration for building a virtual texture set
#[derive(Debug, Clone)]
pub struct TileSetConfiguration {
    /// Tile width in pixels (typically 128 or 256, must be power of 2)
    pub tile_width: u32,
    /// Tile height in pixels (typically 128 or 256, must be power of 2)
    pub tile_height: u32,
    /// Border size in pixels for texture filtering (typically 4 or 8)
    pub tile_border: u32,
    /// Page size in bytes (typically 1MB = 0x100000)
    pub page_size: u32,
    /// Compression preference for tile data
    pub compression: TileCompressionPreference,
    /// Whether to embed next mip level in tile data
    pub embed_mip: bool,
    /// Whether to deduplicate identical tiles
    pub deduplicate: bool,
}

impl Default for TileSetConfiguration {
    fn default() -> Self {
        Self {
            // BG3 uses 144x144 tiles with 8px border = 128px content per tile
            tile_width: 144,
            tile_height: 144,
            tile_border: 8,
            page_size: 0x0010_0000, // 1MB
            compression: TileCompressionPreference::Best,
            embed_mip: true,
            deduplicate: true,
        }
    }
}

impl TileSetConfiguration {
    /// Create a new configuration with default BG3 settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Tile dimensions must be divisible by 4 (BC block alignment)
        if self.tile_width % 4 != 0 {
            return Err(format!("tile_width must be divisible by 4, got {}", self.tile_width));
        }
        if self.tile_height % 4 != 0 {
            return Err(format!("tile_height must be divisible by 4, got {}", self.tile_height));
        }

        // Check reasonable ranges
        if self.tile_width < 16 || self.tile_width > 1024 {
            return Err(format!("tile_width must be 16-1024, got {}", self.tile_width));
        }
        if self.tile_height < 16 || self.tile_height > 1024 {
            return Err(format!("tile_height must be 16-1024, got {}", self.tile_height));
        }

        // Border must be divisible by 4 (BC block size)
        if self.tile_border % 4 != 0 {
            return Err(format!("tile_border must be divisible by 4, got {}", self.tile_border));
        }

        // Border must be less than half the tile size
        if self.tile_border >= self.tile_width / 2 || self.tile_border >= self.tile_height / 2 {
            return Err("tile_border must be less than half the tile dimensions".to_string());
        }

        // Content area must be positive and divisible by 4
        let content_width = self.tile_width - 2 * self.tile_border;
        let content_height = self.tile_height - 2 * self.tile_border;
        if content_width == 0 || content_height == 0 {
            return Err("Content area (tile - 2*border) must be positive".to_string());
        }
        if content_width % 4 != 0 || content_height % 4 != 0 {
            return Err(format!(
                "Content area must be divisible by 4, got {}x{}",
                content_width, content_height
            ));
        }

        // Page size must be reasonable
        if self.page_size < 0x1_0000 {
            return Err("page_size must be at least 64KB".to_string());
        }

        Ok(())
    }

    /// Get the raw tile width (content area without borders)
    #[must_use]
    pub const fn raw_tile_width(&self) -> u32 {
        self.tile_width - 2 * self.tile_border
    }

    /// Get the raw tile height (content area without borders)
    #[must_use]
    pub const fn raw_tile_height(&self) -> u32 {
        self.tile_height - 2 * self.tile_border
    }

    /// Get the padded tile width (including borders)
    #[must_use]
    pub const fn padded_tile_width(&self) -> u32 {
        self.tile_width
    }

    /// Get the padded tile height (including borders)
    #[must_use]
    pub const fn padded_tile_height(&self) -> u32 {
        self.tile_height
    }
}

/// A source texture to be included in the virtual texture set
#[derive(Debug, Clone)]
pub struct SourceTexture {
    /// Name/identifier for this texture (used in FourCC metadata)
    pub name: String,
    /// Path to the base map DDS (color/albedo) - optional
    pub base_map: Option<PathBuf>,
    /// Path to the normal map DDS - optional
    pub normal_map: Option<PathBuf>,
    /// Path to the physical map DDS (roughness/metallic) - optional
    pub physical_map: Option<PathBuf>,
}

impl SourceTexture {
    /// Create a new source texture with the given name
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_map: None,
            normal_map: None,
            physical_map: None,
        }
    }

    /// Set the base map path
    #[must_use]
    pub fn with_base_map(mut self, path: impl Into<PathBuf>) -> Self {
        self.base_map = Some(path.into());
        self
    }

    /// Set the normal map path
    #[must_use]
    pub fn with_normal_map(mut self, path: impl Into<PathBuf>) -> Self {
        self.normal_map = Some(path.into());
        self
    }

    /// Set the physical map path
    #[must_use]
    pub fn with_physical_map(mut self, path: impl Into<PathBuf>) -> Self {
        self.physical_map = Some(path.into());
        self
    }

    /// Get the layer paths as an array (BaseMap=0, NormalMap=1, PhysicalMap=2)
    #[must_use]
    pub fn layer_paths(&self) -> [Option<&PathBuf>; 3] {
        [
            self.base_map.as_ref(),
            self.normal_map.as_ref(),
            self.physical_map.as_ref(),
        ]
    }

    /// Check if any layers are defined
    #[must_use]
    pub fn has_any_layer(&self) -> bool {
        self.base_map.is_some() || self.normal_map.is_some() || self.physical_map.is_some()
    }
}
