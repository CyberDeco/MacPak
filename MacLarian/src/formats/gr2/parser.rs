//! Custom GR2 file parser with BitKnit decompression support
//!
//! This parser handles the Granny3D file format used by BG3, which uses
//! BitKnit compression (type 4) that opengr2 doesn't support yet.

use crate::error::{Error, Result};
use crate::formats::gr2::decompressor::{decompress_section, BITKNIT_TAG};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

/// Granny3D file magic signature
const GRANNY_MAGIC: &[u8; 16] = b"\xe5\x9b\x49\x5e\x6f\x63\x1f\x14\x1e\x13\xeb\xa9\x90\xbe\xed\xc4";

/// Granny file header (partial - only fields we need)
#[derive(Debug, Clone)]
pub struct GrannyHeader {
    pub magic: [u8; 16],
    pub header_size: u32,
    pub header_format: u32,
    pub total_size: u32,
    pub crc32: u32,
    pub section_offset: u32,
    pub section_count: u32,
    pub compression_tag: u32,
}

impl GrannyHeader {
    /// Parse header from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 0x150 {
            return Err(Error::Gr2ParseError(
                format!("File too small: {} bytes (need at least 336)", data.len())
            ));
        }

        // Check magic
        let magic: [u8; 16] = data[0..16].try_into().unwrap();
        if &magic != GRANNY_MAGIC {
            return Err(Error::Gr2ParseError(
                "Invalid Granny magic signature".to_string()
            ));
        }

        let mut cursor = Cursor::new(&data[0x10..]);

        let header_size = cursor.read_u32::<LittleEndian>()?;
        let header_format = cursor.read_u32::<LittleEndian>()?;

        // Skip reserved
        cursor.set_position(0x10);

        // Read file info (offset 0x20 from start = 0x10 from cursor)
        cursor.set_position(0x10);
        let _version = cursor.read_u32::<LittleEndian>()?;
        let total_size = cursor.read_u32::<LittleEndian>()?;
        let crc32 = cursor.read_u32::<LittleEndian>()?;
        let section_offset = cursor.read_u32::<LittleEndian>()?;
        let section_count = cursor.read_u32::<LittleEndian>()?;

        // Compression tag at offset 0x44 from start = 0x34 from cursor
        cursor.set_position(0x34);
        let compression_tag = cursor.read_u32::<LittleEndian>()?;

        Ok(Self {
            magic,
            header_size,
            header_format,
            total_size,
            crc32,
            section_offset,
            section_count,
            compression_tag,
        })
    }
}

/// Granny file section descriptor
#[derive(Debug, Clone)]
pub struct GrannySection {
    pub compression: u32,
    pub data_offset: u32,
    pub compressed_size: u32,
    pub decompressed_size: u32,
    pub alignment: u32,
    pub first16_offset: u32,
    pub first16_count: u32,
}

impl GrannySection {
    /// Parse section from bytes at given offset
    pub fn from_bytes(data: &[u8], offset: usize) -> Result<Self> {
        if offset + 0x1c > data.len() {
            return Err(Error::Gr2ParseError(
                format!("Section descriptor at offset 0x{:x} beyond file bounds", offset)
            ));
        }

        let mut cursor = Cursor::new(&data[offset..]);

        Ok(Self {
            compression: cursor.read_u32::<LittleEndian>()?,
            data_offset: cursor.read_u32::<LittleEndian>()?,
            compressed_size: cursor.read_u32::<LittleEndian>()?,
            decompressed_size: cursor.read_u32::<LittleEndian>()?,
            alignment: cursor.read_u32::<LittleEndian>()?,
            first16_offset: cursor.read_u32::<LittleEndian>()?,
            first16_count: cursor.read_u32::<LittleEndian>()?,
        })
    }

    /// Check if this section is compressed
    pub fn is_compressed(&self) -> bool {
        self.compressed_size > 0 && self.compressed_size < self.decompressed_size
    }
}

/// Complete parsed GR2 file with decompressed data
#[derive(Debug)]
pub struct ParsedGr2File {
    pub header: GrannyHeader,
    pub sections: Vec<GrannySection>,
    pub decompressed_data: Vec<Vec<u8>>,
}

impl ParsedGr2File {
    /// Parse and decompress a GR2 file
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        tracing::debug!("Parsing GR2 file ({} bytes)", data.len());

        // Parse header
        let header = GrannyHeader::from_bytes(data)?;

        tracing::debug!(
            "GR2 header: {} sections, compression tag: 0x{:08x}",
            header.section_count,
            header.compression_tag
        );

        if header.compression_tag != BITKNIT_TAG && header.compression_tag != 0 {
            tracing::warn!(
                "Unknown compression tag: 0x{:08x} (expected BitKnit 0x{:08x})",
                header.compression_tag,
                BITKNIT_TAG
            );
        }

        // Parse section table
        let mut sections = Vec::with_capacity(header.section_count as usize);
        for i in 0..header.section_count {
            let section_offset = header.section_offset as usize + (i as usize * 0x1c);
            let section = GrannySection::from_bytes(data, section_offset)?;

            tracing::debug!(
                "Section {}: compressed={}, decompressed={}, offset=0x{:x}",
                i,
                section.compressed_size,
                section.decompressed_size,
                section.data_offset
            );

            sections.push(section);
        }

        // Decompress each section
        let mut decompressed_data = Vec::with_capacity(sections.len());

        for (i, section) in sections.iter().enumerate() {
            if section.decompressed_size == 0 {
                // Empty section
                tracing::debug!("Section {}: empty", i);
                decompressed_data.push(Vec::new());
                continue;
            }

            let data_start = header.header_size as usize + section.data_offset as usize;

            if section.is_compressed() {
                // Compressed section - decompress it
                let data_end = data_start + section.compressed_size as usize;

                if data_end > data.len() {
                    return Err(Error::Gr2ParseError(format!(
                        "Section {} data (0x{:x}..0x{:x}) beyond file bounds (0x{:x})",
                        i, data_start, data_end, data.len()
                    )));
                }

                let compressed = &data[data_start..data_end];

                tracing::debug!(
                    "Decompressing section {}: {} -> {} bytes",
                    i,
                    section.compressed_size,
                    section.decompressed_size
                );

                match decompress_section(
                    compressed,
                    section.decompressed_size as usize,
                    header.compression_tag,
                ) {
                    Ok(decompressed) => {
                        decompressed_data.push(decompressed);
                    }
                    Err(e) => {
                        // Some GR2 files have malformed sections (e.g., 4 bytes claiming to be 1604)
                        // Skip them gracefully if compressed_size is suspiciously small
                        if section.compressed_size < 8 {
                            tracing::warn!(
                                "Skipping malformed section {} ({} bytes): {}",
                                i, section.compressed_size, e
                            );
                            decompressed_data.push(Vec::new());
                        } else {
                            return Err(e);
                        }
                    }
                }
            } else {
                // Uncompressed section - copy directly
                let data_end = data_start + section.decompressed_size as usize;

                if data_end > data.len() {
                    return Err(Error::Gr2ParseError(format!(
                        "Section {} data (0x{:x}..0x{:x}) beyond file bounds (0x{:x})",
                        i, data_start, data_end, data.len()
                    )));
                }

                tracing::debug!("Copying uncompressed section {}: {} bytes", i, section.decompressed_size);
                decompressed_data.push(data[data_start..data_end].to_vec());
            }
        }

        tracing::info!(
            "Successfully parsed GR2 file: {} sections, {} total decompressed bytes",
            sections.len(),
            decompressed_data.iter().map(|d| d.len()).sum::<usize>()
        );

        Ok(Self {
            header,
            sections,
            decompressed_data,
        })
    }

    /// Get total decompressed size
    pub fn total_decompressed_size(&self) -> usize {
        self.decompressed_data.iter().map(|d| d.len()).sum()
    }

    /// Reconstruct a decompressed GR2 file that opengr2 can parse
    ///
    /// This creates a new GR2 file with all sections decompressed
    /// so that opengr2 (which doesn't support BitKnit) can parse it.
    pub fn to_uncompressed_bytes(&self) -> Result<Vec<u8>> {
        // TODO: Implement reconstruction of uncompressed GR2 format
        // For now, we'll work directly with decompressed sections
        Err(Error::Gr2ParseError(
            "GR2 reconstruction not yet implemented".to_string()
        ))
    }
}
