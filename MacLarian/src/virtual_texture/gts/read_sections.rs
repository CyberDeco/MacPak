//! GTS section reading methods.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use crate::error::Result;
use super::super::types::{
    GtsHeader, GtsParameterBlock, GtsPageFileInfo, GtsPackedTileId,
    GtsFlatTileInfo, GtsCodec, GtsBCParameterBlock, GtsUniformParameterBlock,
    GtsDataType, GtsLevelInfo,
};

/// Read parameter blocks from GTS file.
pub(super) fn read_parameter_blocks<R: Read + Seek>(
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
                let bc_block = read_bc_parameter_block(reader)?;
                GtsParameterBlock::BC(bc_block)
            }
            GtsCodec::Uniform => {
                let uniform_block = read_uniform_parameter_block(reader)?;
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

/// Read level information from GTS file.
pub(super) fn read_levels<R: Read + Seek>(
    reader: &mut R,
    header: &GtsHeader,
) -> Result<Vec<GtsLevelInfo>> {
    let mut levels = Vec::with_capacity(header.num_levels as usize);

    reader.seek(SeekFrom::Start(header.levels_offset))?;

    let mut buf4 = [0u8; 4];
    let mut buf8 = [0u8; 8];

    for _ in 0..header.num_levels {
        reader.read_exact(&mut buf4)?;
        let width_tiles = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let height_tiles = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf8)?;
        let flat_tile_offset = u64::from_le_bytes(buf8);

        // Try to read extended pixel dimensions (may fail for original BG3 files)
        let (width_pixels, height_pixels) = match reader.read_exact(&mut buf4) {
            Ok(()) => {
                let wp = u32::from_le_bytes(buf4);
                match reader.read_exact(&mut buf4) {
                    Ok(()) => (wp, u32::from_le_bytes(buf4)),
                    Err(_) => (0, 0),
                }
            }
            Err(_) => (0, 0),
        };

        levels.push(GtsLevelInfo {
            width_tiles,
            height_tiles,
            flat_tile_offset,
            width_pixels,
            height_pixels,
        });
    }

    Ok(levels)
}

/// Read page file metadata from GTS file.
pub(super) fn read_page_files<R: Read + Seek>(
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

/// Read packed tile IDs from GTS file.
pub(super) fn read_packed_tiles<R: Read + Seek>(
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

/// Read flat tile infos from GTS file.
pub(super) fn read_flat_tile_infos<R: Read + Seek>(
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
