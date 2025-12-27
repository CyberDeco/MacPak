//! GR2 file structure and parsing
//!
//! GR2 files have a header followed by section info and section data.
//! Each section can be independently compressed using BitKnit (format 4).

use crate::error::{Error, Result};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::fs::File;

use super::decompress_gr2;

/// GR2 file magic (first 4 bytes)
const GR2_MAGIC: [u8; 4] = [0xE5, 0x9B, 0x49, 0x5E];

/// Compression format: BitKnit
const COMPRESSION_BITKNIT: u32 = 4;

/// GR2 file header (56 bytes for BG3 format)
#[derive(Debug, Clone)]
pub struct Gr2Header {
    pub magic: [u8; 4],
    pub header_size: u32,
    pub format_version: u32,
    pub file_size: u64,
    pub crc32: u32,
    pub section_count: u32,
    pub section_info_offset: u64,
    pub section_info_size: u32,
    pub compression_type: u32,
}

/// Section info describing each section
#[derive(Debug, Clone)]
pub struct Gr2SectionInfo {
    pub compression: u32,
    pub data_offset: u32,
    pub compressed_size: u32,
    pub decompressed_size: u32,
    pub alignment: u32,
    pub first_stop: u32,
    pub second_stop: u32,
    pub relocations_offset: u32,
    pub relocations_count: u32,
    pub mixed_marshal_offset: u32,
    pub mixed_marshal_count: u32,
}

/// Parse a GR2 file and decompress all sections
pub fn decompress_file(input_path: &Path, output_path: &Path) -> Result<()> {
    let mut file = File::open(input_path)?;
    let file_size = file.metadata()?.len();

    // Read the entire file
    let mut data = Vec::with_capacity(file_size as usize);
    file.read_to_end(&mut data)?;

    // Verify magic
    if data.len() < 4 || data[0..4] != GR2_MAGIC {
        return Err(Error::Decompression(format!(
            "Invalid GR2 magic: {:02x?}, expected {:02x?}",
            &data[0..4.min(data.len())],
            GR2_MAGIC
        )));
    }

    // Parse header - BG3 uses a specific format
    // Offset 0x20: section count (4 bytes)
    // Offset 0x24: total file size (4 bytes)
    // Offset 0x30: section count again
    // Offset 0x34: compression type (4 = BitKnit)
    // Offset 0x38: unknown
    // Offset 0x60: section info starts

    if data.len() < 0x70 {
        return Err(Error::Decompression("File too small for GR2 header".to_string()));
    }

    let section_count = u32::from_le_bytes([data[0x30], data[0x31], data[0x32], data[0x33]]) as usize;
    let compression_type = u32::from_le_bytes([data[0x34], data[0x35], data[0x36], data[0x37]]);

    // Each section info is 44 bytes, starting at offset 0x60
    const SECTION_INFO_OFFSET: usize = 0x60;
    const SECTION_INFO_SIZE: usize = 44;

    let mut sections = Vec::with_capacity(section_count);

    for i in 0..section_count {
        let offset = SECTION_INFO_OFFSET + i * SECTION_INFO_SIZE;
        if offset + SECTION_INFO_SIZE > data.len() {
            return Err(Error::Decompression(format!(
                "Section info {} extends beyond file", i
            )));
        }

        let section = Gr2SectionInfo {
            compression: u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]),
            data_offset: u32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]),
            compressed_size: u32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], data[offset+11]]),
            decompressed_size: u32::from_le_bytes([data[offset+12], data[offset+13], data[offset+14], data[offset+15]]),
            alignment: u32::from_le_bytes([data[offset+16], data[offset+17], data[offset+18], data[offset+19]]),
            first_stop: u32::from_le_bytes([data[offset+20], data[offset+21], data[offset+22], data[offset+23]]),
            second_stop: u32::from_le_bytes([data[offset+24], data[offset+25], data[offset+26], data[offset+27]]),
            relocations_offset: u32::from_le_bytes([data[offset+28], data[offset+29], data[offset+30], data[offset+31]]),
            relocations_count: u32::from_le_bytes([data[offset+32], data[offset+33], data[offset+34], data[offset+35]]),
            mixed_marshal_offset: u32::from_le_bytes([data[offset+36], data[offset+37], data[offset+38], data[offset+39]]),
            mixed_marshal_count: u32::from_le_bytes([data[offset+40], data[offset+41], data[offset+42], data[offset+43]]),
        };
        sections.push(section);
    }

    // Calculate header end (where section data starts)
    let header_end = SECTION_INFO_OFFSET + section_count * SECTION_INFO_SIZE;

    // Build output file
    let mut output = Vec::new();

    // Copy header as-is (we'll update section info later)
    output.extend_from_slice(&data[0..header_end]);

    // Track new section offsets and sizes
    let mut new_sections = Vec::with_capacity(section_count);

    // Decompress each section
    for (i, section) in sections.iter().enumerate() {
        let data_offset = section.data_offset as usize;
        let compressed_size = section.compressed_size as usize;
        let decompressed_size = section.decompressed_size as usize;

        if decompressed_size == 0 {
            // Empty section
            new_sections.push((output.len() as u32, 0u32, 0u32));
            continue;
        }

        // Align output position
        let alignment = section.alignment.max(1) as usize;
        while output.len() % alignment != 0 {
            output.push(0);
        }

        let new_offset = output.len() as u32;

        if section.compression == COMPRESSION_BITKNIT && compressed_size != decompressed_size {
            // Section is compressed - decompress it
            if data_offset + compressed_size > data.len() {
                return Err(Error::Decompression(format!(
                    "Section {} data extends beyond file", i
                )));
            }

            let compressed = &data[data_offset..data_offset + compressed_size];
            let decompressed = decompress_gr2(compressed, decompressed_size)?;

            output.extend_from_slice(&decompressed);
            new_sections.push((new_offset, decompressed_size as u32, decompressed_size as u32));
        } else {
            // Section is not compressed - copy as-is
            if data_offset + decompressed_size > data.len() {
                return Err(Error::Decompression(format!(
                    "Section {} data extends beyond file", i
                )));
            }

            output.extend_from_slice(&data[data_offset..data_offset + decompressed_size]);
            new_sections.push((new_offset, decompressed_size as u32, decompressed_size as u32));
        }
    }

    // Update section info in output with new offsets and sizes
    for (i, (new_offset, new_compressed, new_decompressed)) in new_sections.iter().enumerate() {
        let info_offset = SECTION_INFO_OFFSET + i * SECTION_INFO_SIZE;

        // Set compression to 0 (uncompressed)
        output[info_offset..info_offset+4].copy_from_slice(&0u32.to_le_bytes());
        // Set new data offset
        output[info_offset+4..info_offset+8].copy_from_slice(&new_offset.to_le_bytes());
        // Set new compressed size (same as decompressed)
        output[info_offset+8..info_offset+12].copy_from_slice(&new_compressed.to_le_bytes());
        // Decompressed size stays the same
        output[info_offset+12..info_offset+16].copy_from_slice(&new_decompressed.to_le_bytes());
    }

    // Update file size in header (offset 0x24)
    let new_file_size = output.len() as u32;
    output[0x24..0x28].copy_from_slice(&new_file_size.to_le_bytes());

    // Update compression type to 0 (offset 0x34)
    output[0x34..0x38].copy_from_slice(&0u32.to_le_bytes());

    // Write output file
    let mut out_file = File::create(output_path)?;
    out_file.write_all(&output)?;

    Ok(())
}

/// Get information about a GR2 file's sections
pub fn get_file_info(path: &Path) -> Result<(u32, Vec<Gr2SectionInfo>)> {
    let mut file = File::open(path)?;
    let mut data = vec![0u8; 0x200]; // Read enough for header and section info
    file.read(&mut data)?;

    if data.len() < 4 || data[0..4] != GR2_MAGIC {
        return Err(Error::Decompression("Invalid GR2 magic".to_string()));
    }

    let section_count = u32::from_le_bytes([data[0x30], data[0x31], data[0x32], data[0x33]]) as usize;
    let compression_type = u32::from_le_bytes([data[0x34], data[0x35], data[0x36], data[0x37]]);

    const SECTION_INFO_OFFSET: usize = 0x60;
    const SECTION_INFO_SIZE: usize = 44;

    // Read more if needed
    let needed = SECTION_INFO_OFFSET + section_count * SECTION_INFO_SIZE;
    if data.len() < needed {
        file.seek(SeekFrom::Start(0))?;
        data = vec![0u8; needed];
        file.read(&mut data)?;
    }

    let mut sections = Vec::with_capacity(section_count);

    for i in 0..section_count {
        let offset = SECTION_INFO_OFFSET + i * SECTION_INFO_SIZE;
        let section = Gr2SectionInfo {
            compression: u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]),
            data_offset: u32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]),
            compressed_size: u32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], data[offset+11]]),
            decompressed_size: u32::from_le_bytes([data[offset+12], data[offset+13], data[offset+14], data[offset+15]]),
            alignment: u32::from_le_bytes([data[offset+16], data[offset+17], data[offset+18], data[offset+19]]),
            first_stop: u32::from_le_bytes([data[offset+20], data[offset+21], data[offset+22], data[offset+23]]),
            second_stop: u32::from_le_bytes([data[offset+24], data[offset+25], data[offset+26], data[offset+27]]),
            relocations_offset: u32::from_le_bytes([data[offset+28], data[offset+29], data[offset+30], data[offset+31]]),
            relocations_count: u32::from_le_bytes([data[offset+32], data[offset+33], data[offset+34], data[offset+35]]),
            mixed_marshal_offset: u32::from_le_bytes([data[offset+36], data[offset+37], data[offset+38], data[offset+39]]),
            mixed_marshal_count: u32::from_le_bytes([data[offset+40], data[offset+41], data[offset+42], data[offset+43]]),
        };
        sections.push(section);
    }

    Ok((compression_type, sections))
}
