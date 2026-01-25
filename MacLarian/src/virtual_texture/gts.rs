//! GTS (Game Texture Set) file reader
//!
//! GTS files contain metadata about virtual textures, including:
//! - Tile dimensions and layout
//! - Parameter blocks for each codec type
//! - Page file references
//! - Tile mapping information
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(
    clippy::cast_possible_truncation,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::used_underscore_binding,
    clippy::doc_markdown,
    clippy::missing_panics_doc
)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::{Error, Result};
use super::types::{GtsHeader, GtsParameterBlock, GtsPageFileInfo, GtsPackedTileId, GtsFlatTileInfo, GtsCodec, GtsBCParameterBlock, GtsUniformParameterBlock, GtsDataType, TileCompression, TileLocation};

/// GTS file reader and parser
#[derive(Debug)]
pub struct GtsFile {
    pub header: GtsHeader,
    pub(crate) parameter_blocks: HashMap<u32, GtsParameterBlock>,
    pub page_files: Vec<GtsPageFileInfo>,
    pub packed_tiles: Vec<GtsPackedTileId>,
    pub flat_tile_infos: Vec<GtsFlatTileInfo>,
}

impl GtsFile {
    /// Read and parse a GTS file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or has an invalid format.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read(&mut reader)
    }

    /// Read and parse GTS from a reader
    ///
    /// # Errors
    /// Returns an error if reading fails or the data has an invalid format.
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let header = Self::read_header(reader)?;

        if header.magic != GtsHeader::MAGIC {
            let magic = header.magic;
            let expected = GtsHeader::MAGIC;
            return Err(Error::ConversionError(format!(
                "Invalid GTS magic: 0x{magic:08X}, expected 0x{expected:08X}"
            )));
        }

        // Read parameter blocks
        let parameter_blocks = Self::read_parameter_blocks(reader, &header)?;

        // Read page file metadata
        let page_files = Self::read_page_files(reader, &header)?;

        // Read packed tile IDs
        let packed_tiles = Self::read_packed_tiles(reader, &header)?;

        // Read flat tile infos
        let flat_tile_infos = Self::read_flat_tile_infos(reader, &header)?;

        Ok(Self {
            header,
            parameter_blocks,
            page_files,
            packed_tiles,
            flat_tile_infos,
        })
    }

    fn read_header<R: Read + Seek>(reader: &mut R) -> Result<GtsHeader> {
        reader.seek(SeekFrom::Start(0))?;

        let mut buf4 = [0u8; 4];
        let mut buf8 = [0u8; 8];
        let mut buf16 = [0u8; 16];

        // Magic, Version, Unused
        reader.read_exact(&mut buf4)?;
        let magic = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let version = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let unused = u32::from_le_bytes(buf4);

        // GUID
        reader.read_exact(&mut buf16)?;
        let guid = buf16;

        // NumLayers, LayersOffset
        reader.read_exact(&mut buf4)?;
        let num_layers = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf8)?;
        let layers_offset = u64::from_le_bytes(buf8);

        // NumLevels, LevelsOffset
        reader.read_exact(&mut buf4)?;
        let num_levels = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf8)?;
        let levels_offset = u64::from_le_bytes(buf8);

        // TileWidth, TileHeight, TileBorder
        reader.read_exact(&mut buf4)?;
        let tile_width = i32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let tile_height = i32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let tile_border = i32::from_le_bytes(buf4);

        // I2, NumFlatTileInfos, FlatTileInfoOffset, I6, I7
        reader.read_exact(&mut buf4)?;
        let i2 = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let num_flat_tile_infos = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf8)?;
        let flat_tile_info_offset = u64::from_le_bytes(buf8);
        reader.read_exact(&mut buf4)?;
        let i6 = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let i7 = u32::from_le_bytes(buf4);

        // NumPackedTileIDs, PackedTileIDsOffset
        reader.read_exact(&mut buf4)?;
        let num_packed_tile_ids = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf8)?;
        let packed_tile_ids_offset = u64::from_le_bytes(buf8);

        // M, N, O, P, Q, R, S
        reader.read_exact(&mut buf4)?;
        let m = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let n = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let o = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let p = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let q = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let r = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let s = u32::from_le_bytes(buf4);

        // PageSize, NumPageFiles, PageFileMetadataOffset
        reader.read_exact(&mut buf4)?;
        let page_size = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let num_page_files = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf8)?;
        let page_file_metadata_offset = u64::from_le_bytes(buf8);

        // FourCCListSize, FourCCListOffset
        reader.read_exact(&mut buf4)?;
        let fourcc_list_size = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf8)?;
        let fourcc_list_offset = u64::from_le_bytes(buf8);

        // ParameterBlockHeadersCount, ParameterBlockHeadersOffset
        reader.read_exact(&mut buf4)?;
        let parameter_block_headers_count = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf8)?;
        let parameter_block_headers_offset = u64::from_le_bytes(buf8);

        // ThumbnailsOffset, XJJ, XKK, XLL, XMM
        reader.read_exact(&mut buf8)?;
        let thumbnails_offset = u64::from_le_bytes(buf8);
        reader.read_exact(&mut buf4)?;
        let xjj = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let xkk = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let xll = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let xmm = u32::from_le_bytes(buf4);

        Ok(GtsHeader {
            magic,
            version,
            unused,
            guid,
            num_layers,
            layers_offset,
            num_levels,
            levels_offset,
            tile_width,
            tile_height,
            tile_border,
            i2,
            num_flat_tile_infos,
            flat_tile_info_offset,
            i6,
            i7,
            num_packed_tile_ids,
            packed_tile_ids_offset,
            m,
            n,
            o,
            p,
            q,
            r,
            s,
            page_size,
            num_page_files,
            page_file_metadata_offset,
            fourcc_list_size,
            fourcc_list_offset,
            parameter_block_headers_count,
            parameter_block_headers_offset,
            thumbnails_offset,
            xjj,
            xkk,
            xll,
            xmm,
        })
    }

    fn read_parameter_blocks<R: Read + Seek>(
        reader: &mut R,
        header: &GtsHeader,
    ) -> Result<HashMap<u32, GtsParameterBlock>> {
        let mut blocks = HashMap::new();

        reader.seek(SeekFrom::Start(header.parameter_block_headers_offset))?;

        for _ in 0..header.parameter_block_headers_count {
            let mut buf4 = [0u8; 4];
            let mut buf8 = [0u8; 8];

            reader.read_exact(&mut buf4)?;
            let param_id = u32::from_le_bytes(buf4);
            reader.read_exact(&mut buf4)?;
            let codec_val = u32::from_le_bytes(buf4);
            reader.read_exact(&mut buf4)?;
            let _size = u32::from_le_bytes(buf4);
            reader.read_exact(&mut buf8)?;
            let file_info_offset = u64::from_le_bytes(buf8);

            let codec = GtsCodec::from_u32(codec_val).unwrap_or(GtsCodec::Bc);

            // Save position and read parameter block
            let pos = reader.stream_position()?;

            reader.seek(SeekFrom::Start(file_info_offset))?;

            let block = match codec {
                GtsCodec::Bc => {
                    let bc_block = Self::read_bc_parameter_block(reader)?;
                    GtsParameterBlock::BC(bc_block)
                }
                GtsCodec::Uniform => {
                    let uniform_block = Self::read_uniform_parameter_block(reader)?;
                    GtsParameterBlock::Uniform(uniform_block)
                }
                _ => GtsParameterBlock::Unknown,
            };

            blocks.insert(param_id, block);

            // Restore position
            reader.seek(SeekFrom::Start(pos))?;
        }

        Ok(blocks)
    }

    fn read_bc_parameter_block<R: Read>(reader: &mut R) -> Result<GtsBCParameterBlock> {
        let mut buf2 = [0u8; 2];
        let mut buf4 = [0u8; 4];
        let mut buf16 = [0u8; 16];

        reader.read_exact(&mut buf2)?;
        let version = u16::from_le_bytes(buf2);

        reader.read_exact(&mut buf16)?;
        let compression1 = buf16;

        reader.read_exact(&mut buf16)?;
        let compression2 = buf16;

        reader.read_exact(&mut buf4)?;
        let b = u32::from_le_bytes(buf4);

        let mut buf1 = [0u8; 1];
        reader.read_exact(&mut buf1)?;
        let c1 = buf1[0];
        reader.read_exact(&mut buf1)?;
        let c2 = buf1[0];
        reader.read_exact(&mut buf1)?;
        let bc_field3 = buf1[0];
        reader.read_exact(&mut buf1)?;
        let data_type = buf1[0];

        reader.read_exact(&mut buf2)?;
        let d = u16::from_le_bytes(buf2);

        reader.read_exact(&mut buf4)?;
        let fourcc = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf1)?;
        let e1 = buf1[0];
        reader.read_exact(&mut buf1)?;
        let save_mip = buf1[0];
        reader.read_exact(&mut buf1)?;
        let e3 = buf1[0];
        reader.read_exact(&mut buf1)?;
        let e4 = buf1[0];

        reader.read_exact(&mut buf4)?;
        let f = u32::from_le_bytes(buf4);

        Ok(GtsBCParameterBlock {
            version,
            compression1,
            compression2,
            b,
            c1,
            c2,
            bc_field3,
            data_type,
            d,
            fourcc,
            e1,
            save_mip,
            e3,
            e4,
            f,
        })
    }

    fn read_uniform_parameter_block<R: Read>(reader: &mut R) -> Result<GtsUniformParameterBlock> {
        let mut buf2 = [0u8; 2];
        let mut buf4 = [0u8; 4];

        reader.read_exact(&mut buf2)?;
        let version = u16::from_le_bytes(buf2);
        reader.read_exact(&mut buf2)?;
        let a_unused = u16::from_le_bytes(buf2);
        reader.read_exact(&mut buf4)?;
        let width = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let height = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let data_type_val = u32::from_le_bytes(buf4);

        let data_type = GtsDataType::from_u32(data_type_val).unwrap_or(GtsDataType::R8G8B8Srgb);

        Ok(GtsUniformParameterBlock {
            version,
            a_unused,
            width,
            height,
            data_type,
        })
    }

    fn read_page_files<R: Read + Seek>(
        reader: &mut R,
        header: &GtsHeader,
    ) -> Result<Vec<GtsPageFileInfo>> {
        let mut page_files = Vec::with_capacity(header.num_page_files as usize);

        reader.seek(SeekFrom::Start(header.page_file_metadata_offset))?;

        for _ in 0..header.num_page_files {
            // Filename is UTF-16LE, 512 bytes
            let mut filename_buf = [0u8; 512];
            reader.read_exact(&mut filename_buf)?;

            // Find null terminator
            let mut name_len = 0;
            for i in (0..512).step_by(2) {
                if filename_buf[i] == 0 && filename_buf[i + 1] == 0 {
                    break;
                }
                name_len = i + 2;
            }

            // Decode UTF-16LE
            let filename = String::from_utf16_lossy(
                &filename_buf[..name_len]
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect::<Vec<_>>(),
            );

            let mut buf4 = [0u8; 4];
            reader.read_exact(&mut buf4)?;
            let num_pages = u32::from_le_bytes(buf4);

            // Skip remaining metadata (16 bytes GUID + 4 bytes unknown)
            let mut _skip = [0u8; 20];
            reader.read_exact(&mut _skip)?;

            page_files.push(GtsPageFileInfo {
                filename,
                num_pages,
            });
        }

        Ok(page_files)
    }

    fn read_packed_tiles<R: Read + Seek>(
        reader: &mut R,
        header: &GtsHeader,
    ) -> Result<Vec<GtsPackedTileId>> {
        let mut packed_tiles = Vec::with_capacity(header.num_packed_tile_ids as usize);

        reader.seek(SeekFrom::Start(header.packed_tile_ids_offset))?;

        for _ in 0..header.num_packed_tile_ids {
            let mut buf4 = [0u8; 4];
            reader.read_exact(&mut buf4)?;
            let value = u32::from_le_bytes(buf4);
            packed_tiles.push(GtsPackedTileId::from_u32(value));
        }

        Ok(packed_tiles)
    }

    fn read_flat_tile_infos<R: Read + Seek>(
        reader: &mut R,
        header: &GtsHeader,
    ) -> Result<Vec<GtsFlatTileInfo>> {
        let mut tile_infos = Vec::with_capacity(header.num_flat_tile_infos as usize);

        reader.seek(SeekFrom::Start(header.flat_tile_info_offset))?;

        for _ in 0..header.num_flat_tile_infos {
            let mut buf2 = [0u8; 2];
            let mut buf4 = [0u8; 4];

            reader.read_exact(&mut buf2)?;
            let page_file_index = u16::from_le_bytes(buf2);
            reader.read_exact(&mut buf2)?;
            let page_index = u16::from_le_bytes(buf2);
            reader.read_exact(&mut buf2)?;
            let chunk_index = u16::from_le_bytes(buf2);
            reader.read_exact(&mut buf2)?;
            let d = u16::from_le_bytes(buf2);
            reader.read_exact(&mut buf4)?;
            let packed_tile_id_index = u32::from_le_bytes(buf4);

            tile_infos.push(GtsFlatTileInfo {
                page_file_index,
                page_index,
                chunk_index,
                d,
                packed_tile_id_index,
            });
        }

        Ok(tile_infos)
    }

    /// Get compression method for a parameter block
    #[must_use] 
    pub fn get_compression_method(&self, param_block_id: u32) -> TileCompression {
        match self.parameter_blocks.get(&param_block_id) {
            Some(GtsParameterBlock::BC(bc)) => bc.get_compression_method(),
            _ => TileCompression::Raw,
        }
    }

    /// Find the page file index by filename hash
    #[must_use] 
    pub fn find_page_file_index(&self, hash: &str) -> Option<u16> {
        for (i, pf) in self.page_files.iter().enumerate() {
            if pf.filename.contains(hash) {
                return Some(i as u16);
            }
        }
        None
    }

    /// Get tiles for a specific page file, organized by layer
    ///
    /// Prefers level 0 (full resolution) but falls back to higher level numbers
    /// if a layer doesn't have level 0 tiles (e.g., PhysicalMap is often stored
    /// at lower resolution).
    #[must_use]
    pub fn get_tiles_for_page_file(&self, page_file_index: u16) -> [Vec<TileLocation>; 3] {
        // First pass: collect all tiles by layer and level
        let mut tiles_by_layer_level: [std::collections::HashMap<u8, Vec<TileLocation>>; 3] = [
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
        ];

        for tile_info in &self.flat_tile_infos {
            if tile_info.page_file_index != page_file_index {
                continue;
            }

            let packed_idx = tile_info.packed_tile_id_index as usize;
            if packed_idx >= self.packed_tiles.len() {
                continue;
            }

            let packed = &self.packed_tiles[packed_idx];
            let layer_idx = packed.layer as usize;

            if layer_idx >= 3 {
                continue;
            }

            tiles_by_layer_level[layer_idx]
                .entry(packed.level)
                .or_default()
                .push(TileLocation {
                    page: tile_info.page_index,
                    chunk: tile_info.chunk_index,
                    x: packed.x,
                    y: packed.y,
                });
        }

        // Second pass: for each layer, select the best available level (lowest number = highest res)
        let mut tiles_by_layer: [Vec<TileLocation>; 3] = [Vec::new(), Vec::new(), Vec::new()];

        for (layer_idx, level_map) in tiles_by_layer_level.iter().enumerate() {
            if level_map.is_empty() {
                continue;
            }

            // Find the minimum level (highest resolution available)
            let best_level = *level_map.keys().min().expect("level_map is non-empty");
            tiles_by_layer[layer_idx] = level_map.get(&best_level).cloned().unwrap_or_default();

            if best_level != 0 {
                tracing::info!(
                    "Layer {layer_idx} using level {best_level} (level 0 not available)"
                );
            }
        }

        let num_layers = self.header.num_layers;
        let l0 = tiles_by_layer[0].len();
        let l1 = tiles_by_layer[1].len();
        let l2 = tiles_by_layer[2].len();
        tracing::debug!(
            "GTS num_layers={num_layers}, selected tiles by layer: [0]={l0}, [1]={l1}, [2]={l2}"
        );

        tiles_by_layer
    }

    /// Get content dimensions (tile size minus border)
    #[must_use] 
    pub fn content_width(&self) -> i32 {
        self.header.tile_width - self.header.tile_border * 2
    }

    #[must_use] 
    pub fn content_height(&self) -> i32 {
        self.header.tile_height - self.header.tile_border * 2
    }
}
