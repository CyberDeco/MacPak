//! GTP header and chunk offset reading.

use std::io::{Read, Seek, SeekFrom};
use crate::error::Result;
use super::super::types::GtpHeader;

/// Read GTP header from a reader.
pub(super) fn read_header<R: Read>(reader: &mut R) -> Result<GtpHeader> {
    let mut buf4 = [0u8; 4];
    let mut buf16 = [0u8; 16];

    reader.read_exact(&mut buf4)?;
    let magic = u32::from_le_bytes(buf4);
    reader.read_exact(&mut buf4)?;
    let version = u32::from_le_bytes(buf4);
    reader.read_exact(&mut buf16)?;
    let guid = buf16;

    Ok(GtpHeader {
        magic,
        version,
        guid,
    })
}

/// Read chunk offsets for all pages.
pub(super) fn read_chunk_offsets<R: Read + Seek>(
    reader: &mut R,
    page_size: u32,
    num_pages: usize,
) -> Result<Vec<Vec<u32>>> {
    let mut chunk_offsets = Vec::with_capacity(num_pages);

    for page in 0..num_pages {
        let page_start = (page as u64) * u64::from(page_size);

        // Position at start of page
        if page == 0 {
            // After GTP header (24 bytes)
            reader.seek(SeekFrom::Start(24))?;
        } else {
            reader.seek(SeekFrom::Start(page_start))?;
        }

        let mut buf4 = [0u8; 4];
        reader.read_exact(&mut buf4)?;
        let num_chunks = u32::from_le_bytes(buf4) as usize;

        let mut offsets = Vec::with_capacity(num_chunks);
        for _ in 0..num_chunks {
            reader.read_exact(&mut buf4)?;
            offsets.push(u32::from_le_bytes(buf4));
        }

        chunk_offsets.push(offsets);
    }

    Ok(chunk_offsets)
}
