//! Types for GTS/GTP virtual texture files
//!
//! GTS (Game Texture Set) files contain metadata about virtual textures.
//! GTP (Game Texture Page) files contain the actual tile data.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::cast_possible_truncation, clippy::doc_markdown)]

/// GTS codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GtsCodec {
    Uniform = 0,
    Color420 = 1,
    Normal = 2,
    RawColor = 3,
    Binary = 4,
    Codec15Color420 = 5,
    Codec15Normal = 6,
    RawNormal = 7,
    Half = 8,
    BC = 9,
    MultiChannel = 10,
    ASTC = 11,
}

impl GtsCodec {
    #[must_use] 
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Uniform),
            1 => Some(Self::Color420),
            2 => Some(Self::Normal),
            3 => Some(Self::RawColor),
            4 => Some(Self::Binary),
            5 => Some(Self::Codec15Color420),
            6 => Some(Self::Codec15Normal),
            7 => Some(Self::RawNormal),
            8 => Some(Self::Half),
            9 => Some(Self::BC),
            10 => Some(Self::MultiChannel),
            11 => Some(Self::ASTC),
            _ => None,
        }
    }
}

/// GTS data types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GtsDataType {
    R8G8B8Srgb = 0,
    R8G8B8A8Srgb = 1,
    X8Y8Z0Tangent = 2,
    R8G8B8Linear = 3,
    R8G8B8A8Linear = 4,
    X8 = 5,
    X8Y8 = 6,
    X8Y8Z8 = 7,
    X8Y8Z8W8 = 8,
    X16 = 9,
    X16Y16 = 10,
    X16Y16Z16 = 11,
    X16Y16Z16W16 = 12,
    X32 = 13,
    X32Float = 14,
    X32Y32 = 15,
    X32Y32Float = 16,
    X32Y32Z32 = 17,
    X32Y32Z32Float = 18,
    R32G32B32 = 19,
    R32G32B32Float = 20,
    X32Y32Z32W32 = 21,
    X32Y32Z32W32Float = 22,
    R32G32B32A32 = 23,
    R32G32B32A32Float = 24,
    R16G16B16Float = 25,
    R16G16B16A16Float = 26,
}

impl GtsDataType {
    #[must_use] 
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::R8G8B8Srgb),
            1 => Some(Self::R8G8B8A8Srgb),
            2 => Some(Self::X8Y8Z0Tangent),
            3 => Some(Self::R8G8B8Linear),
            4 => Some(Self::R8G8B8A8Linear),
            5 => Some(Self::X8),
            6 => Some(Self::X8Y8),
            7 => Some(Self::X8Y8Z8),
            8 => Some(Self::X8Y8Z8W8),
            9 => Some(Self::X16),
            10 => Some(Self::X16Y16),
            11 => Some(Self::X16Y16Z16),
            12 => Some(Self::X16Y16Z16W16),
            13 => Some(Self::X32),
            14 => Some(Self::X32Float),
            15 => Some(Self::X32Y32),
            16 => Some(Self::X32Y32Float),
            17 => Some(Self::X32Y32Z32),
            18 => Some(Self::X32Y32Z32Float),
            19 => Some(Self::R32G32B32),
            20 => Some(Self::R32G32B32Float),
            21 => Some(Self::X32Y32Z32W32),
            22 => Some(Self::X32Y32Z32W32Float),
            23 => Some(Self::R32G32B32A32),
            24 => Some(Self::R32G32B32A32Float),
            25 => Some(Self::R16G16B16Float),
            26 => Some(Self::R16G16B16A16Float),
            _ => None,
        }
    }
}

/// Tile compression method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileCompression {
    Raw,
    Lz4,
    FastLZ,
}

/// GTS file header (156 bytes)
#[derive(Debug, Clone)]
pub struct GtsHeader {
    pub magic: u32,
    pub version: u32,
    pub unused: u32,
    pub guid: [u8; 16],
    pub num_layers: u32,
    pub layers_offset: u64,
    pub num_levels: u32,
    pub levels_offset: u64,
    pub tile_width: i32,
    pub tile_height: i32,
    pub tile_border: i32,
    pub i2: u32,
    pub num_flat_tile_infos: u32,
    pub flat_tile_info_offset: u64,
    pub i6: u32,
    pub i7: u32,
    pub num_packed_tile_ids: u32,
    pub packed_tile_ids_offset: u64,
    pub m: u32,
    pub n: u32,
    pub o: u32,
    pub p: u32,
    pub q: u32,
    pub r: u32,
    pub s: u32,
    pub page_size: u32,
    pub num_page_files: u32,
    pub page_file_metadata_offset: u64,
    pub fourcc_list_size: u32,
    pub fourcc_list_offset: u64,
    pub parameter_block_headers_count: u32,
    pub parameter_block_headers_offset: u64,
    pub thumbnails_offset: u64,
    pub xjj: u32,
    pub xkk: u32,
    pub xll: u32,
    pub xmm: u32,
}

impl GtsHeader {
    pub const MAGIC: u32 = 0x4750_5247; // 'GRPG'
}

/// GTS parameter block header (20 bytes)
#[derive(Debug, Clone)]
pub struct GtsParameterBlockHeader {
    pub parameter_block_id: u32,
    pub codec: GtsCodec,
    pub parameter_block_size: u32,
    pub file_info_offset: u64,
}

/// BC codec parameter block (56 bytes)
#[derive(Debug, Clone)]
pub struct GtsBCParameterBlock {
    pub version: u16,
    pub compression1: [u8; 16],
    pub compression2: [u8; 16],
    pub b: u32,
    pub c1: u8,
    pub c2: u8,
    pub bc_field3: u8,
    pub data_type: u8,
    pub d: u16,
    pub fourcc: u32,
    pub e1: u8,
    pub save_mip: u8,
    pub e3: u8,
    pub e4: u8,
    pub f: u32,
}

impl GtsBCParameterBlock {
    /// Get compression name 1 as string
    #[must_use] 
    pub fn compression_name1(&self) -> String {
        let end = self.compression1.iter().position(|&b| b == 0).unwrap_or(16);
        String::from_utf8_lossy(&self.compression1[..end]).to_string()
    }

    /// Get compression name 2 as string
    #[must_use] 
    pub fn compression_name2(&self) -> String {
        let end = self.compression2.iter().position(|&b| b == 0).unwrap_or(16);
        String::from_utf8_lossy(&self.compression2[..end]).to_string()
    }

    /// Determine the tile compression method
    #[must_use] 
    pub fn get_compression_method(&self) -> TileCompression {
        let name1 = self.compression_name1();
        let name2 = self.compression_name2();

        if name1 == "lz77" && name2 == "fastlz0.1.0" {
            TileCompression::FastLZ
        } else if name1 == "lz4" && name2 == "lz40.1.0" {
            TileCompression::Lz4
        } else if name1 == "raw" {
            TileCompression::Raw
        } else {
            // Default to raw if unknown
            TileCompression::Raw
        }
    }
}

/// Uniform codec parameter block (16 bytes)
#[derive(Debug, Clone)]
pub struct GtsUniformParameterBlock {
    pub version: u16,
    pub a_unused: u16,
    pub width: u32,
    pub height: u32,
    pub data_type: GtsDataType,
}

/// Parameter block data
#[derive(Debug, Clone)]
pub enum GtsParameterBlock {
    BC(GtsBCParameterBlock),
    Uniform(GtsUniformParameterBlock),
    Unknown,
}

/// Page file metadata
#[derive(Debug, Clone)]
pub struct GtsPageFileInfo {
    pub filename: String,
    pub num_pages: u32,
}

/// Flat tile info (12 bytes)
#[derive(Debug, Clone)]
pub struct GtsFlatTileInfo {
    pub page_file_index: u16,
    pub page_index: u16,
    pub chunk_index: u16,
    pub d: u16,
    pub packed_tile_id_index: u32,
}

/// Packed tile ID (decoded from 32-bit value)
#[derive(Debug, Clone)]
pub struct GtsPackedTileId {
    pub layer: u8,
    pub level: u8,
    pub x: u16,
    pub y: u16,
}

impl GtsPackedTileId {
    #[must_use] 
    pub fn from_u32(value: u32) -> Self {
        Self {
            layer: (value & 0xF) as u8,
            level: ((value >> 4) & 0xF) as u8,
            y: ((value >> 8) & 0xFFF) as u16,
            x: (value >> 20) as u16,
        }
    }
}

/// GTP file header (24 bytes)
#[derive(Debug, Clone)]
pub struct GtpHeader {
    pub magic: u32,
    pub version: u32,
    pub guid: [u8; 16],
}

impl GtpHeader {
    pub const MAGIC: u32 = 0x5041_5247; // 'GRAP' (reads as 'PARG' in little endian)
}

/// GTP chunk header (12 bytes)
#[derive(Debug, Clone)]
pub struct GtpChunkHeader {
    pub codec: GtsCodec,
    pub parameter_block_id: u32,
    pub size: u32,
}

/// Tile location info for combining
#[derive(Debug, Clone)]
pub struct TileLocation {
    pub page: u16,
    pub chunk: u16,
    pub x: u16,
    pub y: u16,
}

/// Layer type for virtual textures
///
/// BG3 virtual textures have 3 layers:
/// - Layer 0: BaseMap (color/albedo)
/// - Layer 1: NormalMap (surface normals)
/// - Layer 2: PhysicalMap (roughness/metallic/etc)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualTextureLayer {
    BaseMap = 0,
    NormalMap = 1,
    PhysicalMap = 2,
}

impl VirtualTextureLayer {
    #[must_use]
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::BaseMap),
            1 => Some(Self::NormalMap),
            2 => Some(Self::PhysicalMap),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BaseMap => "BaseMap",
            Self::NormalMap => "NormalMap",
            Self::PhysicalMap => "PhysicalMap",
        }
    }
}

/// Output from virtual texture extraction
#[derive(Debug)]
pub struct VirtualTextureOutput {
    pub layer: VirtualTextureLayer,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}
