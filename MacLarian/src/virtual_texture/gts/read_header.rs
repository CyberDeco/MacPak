//! GTS header reading.

use super::super::types::GtsHeader;
use crate::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read and parse a GTS header from a reader.
pub(super) fn read_header<R: Read + Seek>(reader: &mut R) -> Result<GtsHeader> {
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
