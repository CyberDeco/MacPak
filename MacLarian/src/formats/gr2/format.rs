//! GR2 file format structures
//!
//! Based on RAD Game Tools Granny2 format, version 2.11.8.0

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

use crate::error::{Error, Result};

/// Magic signatures for different GR2 formats
pub mod magic {
    /// Little-endian 32-bit format
    pub const LE32: [u8; 16] = [
        0x29, 0xDE, 0x6C, 0xC0, 0xBA, 0xA4, 0x53, 0x2B,
        0x25, 0xF5, 0xB7, 0xA5, 0xF6, 0x66, 0xE2, 0xEE,
    ];

    /// Little-endian 64-bit format
    pub const LE64: [u8; 16] = [
        0xE5, 0x9B, 0x49, 0x5E, 0x6F, 0x63, 0x1F, 0x14,
        0x1E, 0x13, 0xEB, 0xA9, 0x90, 0xBE, 0xED, 0xC4,
    ];

    /// Big-endian 32-bit format
    pub const BE32: [u8; 16] = [
        0x0E, 0x11, 0x95, 0xB5, 0x6A, 0xA5, 0xB5, 0x4B,
        0xEB, 0x28, 0x28, 0x50, 0x25, 0x78, 0xB3, 0x04,
    ];

    /// Big-endian 64-bit format
    pub const BE64: [u8; 16] = [
        0x31, 0x95, 0xD4, 0xE3, 0x20, 0xDC, 0x4F, 0x62,
        0xCC, 0x36, 0xD0, 0x3A, 0xB1, 0x82, 0xFF, 0x89,
    ];
}

/// Known game tags
pub mod tags {
    /// Divinity: Original Sin
    pub const DOS: u32 = 0x80000037;
    /// Divinity: Original Sin Enhanced Edition
    pub const DOS_EE: u32 = 0x80000039;
    /// Divinity: Original Sin 2 / Baldur's Gate 3
    pub const DOS2_BG3: u32 = 0xE57F0039;
}

/// Compression formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Compression {
    /// No compression
    None = 0,
    /// Oodle0 compression (legacy)
    Oodle0 = 1,
    /// Oodle1 compression (legacy)
    Oodle1 = 2,
    /// BitKnit compression (modern, used in BG3/DOS2)
    BitKnit = 4,
}

impl Compression {
    pub fn from_u32(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Compression::None),
            1 => Ok(Compression::Oodle0),
            2 => Ok(Compression::Oodle1),
            4 => Ok(Compression::BitKnit),
            _ => Err(Error::Decompression(format!("Unsupported GR2 compression format: {}", value))),
        }
    }
}

/// Pointer size determined from magic signature
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerSize {
    Bit32,
    Bit64,
}

/// Endianness determined from magic signature
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

/// Magic block (32 bytes at offset 0)
#[derive(Debug, Clone)]
pub struct Gr2Magic {
    /// Format signature (16 bytes)
    pub signature: [u8; 16],
    /// Offset where section data begins
    pub headers_size: u32,
    /// Header format (0 = uncompressed)
    pub header_format: u32,
    /// Reserved bytes
    pub reserved: [u8; 8],
}

impl Gr2Magic {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut signature = [0u8; 16];
        reader.read_exact(&mut signature)?;

        let headers_size = reader.read_u32::<LittleEndian>()?;
        let header_format = reader.read_u32::<LittleEndian>()?;

        let mut reserved = [0u8; 8];
        reader.read_exact(&mut reserved)?;

        Ok(Self {
            signature,
            headers_size,
            header_format,
            reserved,
        })
    }

    /// Get pointer size from signature
    pub fn pointer_size(&self) -> Result<PointerSize> {
        if self.signature == magic::LE32 || self.signature == magic::BE32 {
            Ok(PointerSize::Bit32)
        } else if self.signature == magic::LE64 || self.signature == magic::BE64 {
            Ok(PointerSize::Bit64)
        } else {
            Err(Error::Decompression("Invalid GR2 magic signature".to_string()))
        }
    }

    /// Get endianness from signature
    pub fn endian(&self) -> Result<Endian> {
        if self.signature == magic::LE32 || self.signature == magic::LE64 {
            Ok(Endian::Little)
        } else if self.signature == magic::BE32 || self.signature == magic::BE64 {
            Ok(Endian::Big)
        } else {
            Err(Error::Decompression("Invalid GR2 magic signature".to_string()))
        }
    }

    /// Check if the magic signature is valid
    pub fn is_valid(&self) -> bool {
        self.signature == magic::LE32
            || self.signature == magic::LE64
            || self.signature == magic::BE32
            || self.signature == magic::BE64
    }
}

/// Reference to data in a section
#[derive(Debug, Clone, Copy, Default)]
pub struct SectionRef {
    pub section: u32,
    pub offset: u32,
}

impl SectionRef {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Self {
            section: reader.read_u32::<LittleEndian>()?,
            offset: reader.read_u32::<LittleEndian>()?,
        })
    }
}

/// Main file header
#[derive(Debug, Clone)]
pub struct Gr2Header {
    /// Format version (6 or 7)
    pub version: u32,
    /// Total file size
    pub file_size: u32,
    /// CRC32 checksum
    pub crc: u32,
    /// Offset to section headers (from header start at 0x20)
    pub sections_offset: u32,
    /// Number of sections
    pub num_sections: u32,
    /// Reference to root type definition
    pub root_type: SectionRef,
    /// Reference to root data node
    pub root_node: SectionRef,
    /// Game version tag
    pub tag: u32,
    /// Extra tags (4 u32 values)
    pub extra_tags: [u32; 4],
    /// String table CRC (v7 only)
    pub string_table_crc: Option<u32>,
}

impl Gr2Header {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let version = reader.read_u32::<LittleEndian>()?;
        if version != 6 && version != 7 {
            return Err(Error::Decompression(format!("Unsupported GR2 version: {}", version)));
        }

        let file_size = reader.read_u32::<LittleEndian>()?;
        let crc = reader.read_u32::<LittleEndian>()?;
        let sections_offset = reader.read_u32::<LittleEndian>()?;
        let num_sections = reader.read_u32::<LittleEndian>()?;
        let root_type = SectionRef::read(reader)?;
        let root_node = SectionRef::read(reader)?;
        let tag = reader.read_u32::<LittleEndian>()?;

        let mut extra_tags = [0u32; 4];
        for tag in &mut extra_tags {
            *tag = reader.read_u32::<LittleEndian>()?;
        }

        let string_table_crc = if version == 7 {
            let crc = reader.read_u32::<LittleEndian>()?;
            // Skip 12 reserved bytes
            let mut reserved = [0u8; 12];
            reader.read_exact(&mut reserved)?;
            Some(crc)
        } else {
            None
        };

        Ok(Self {
            version,
            file_size,
            crc,
            sections_offset,
            num_sections,
            root_type,
            root_node,
            tag,
            extra_tags,
            string_table_crc,
        })
    }

    /// Get header size based on version
    pub fn size(&self) -> usize {
        if self.version == 7 { 88 } else { 72 }
    }
}

/// Section header (44 bytes each)
#[derive(Debug, Clone)]
pub struct SectionHeader {
    /// Compression format
    pub compression: Compression,
    /// Offset to compressed data in file
    pub offset_in_file: u32,
    /// Size of compressed data
    pub compressed_size: u32,
    /// Size after decompression
    pub uncompressed_size: u32,
    /// Data alignment
    pub alignment: u32,
    /// Oodle stop point 0 (first 16-bit boundary)
    pub first_16bit: u32,
    /// Oodle stop point 1 (first 8-bit boundary)
    pub first_8bit: u32,
    /// Offset to relocation data
    pub relocations_offset: u32,
    /// Number of relocations
    pub num_relocations: u32,
    /// Offset to mixed marshalling data
    pub mixed_marshalling_offset: u32,
    /// Number of mixed marshalling entries
    pub num_mixed_marshalling: u32,
}

impl SectionHeader {
    pub const SIZE: usize = 44;

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let compression_raw = reader.read_u32::<LittleEndian>()?;
        let compression = Compression::from_u32(compression_raw)?;

        Ok(Self {
            compression,
            offset_in_file: reader.read_u32::<LittleEndian>()?,
            compressed_size: reader.read_u32::<LittleEndian>()?,
            uncompressed_size: reader.read_u32::<LittleEndian>()?,
            alignment: reader.read_u32::<LittleEndian>()?,
            first_16bit: reader.read_u32::<LittleEndian>()?,
            first_8bit: reader.read_u32::<LittleEndian>()?,
            relocations_offset: reader.read_u32::<LittleEndian>()?,
            num_relocations: reader.read_u32::<LittleEndian>()?,
            mixed_marshalling_offset: reader.read_u32::<LittleEndian>()?,
            num_mixed_marshalling: reader.read_u32::<LittleEndian>()?,
        })
    }

    /// Check if section has data
    pub fn is_empty(&self) -> bool {
        self.compressed_size == 0
    }

    /// Get compression ratio
    pub fn compression_ratio(&self) -> Option<f64> {
        if self.compressed_size > 0 {
            Some(self.uncompressed_size as f64 / self.compressed_size as f64)
        } else {
            None
        }
    }
}

/// Relocation entry (12 bytes)
#[derive(Debug, Clone, Copy)]
pub struct Relocation {
    /// Offset within the section to patch
    pub offset_in_section: u32,
    /// Target section index
    pub target_section: u32,
    /// Offset within target section
    pub target_offset: u32,
}

impl Relocation {
    pub const SIZE: usize = 12;

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Self {
            offset_in_section: reader.read_u32::<LittleEndian>()?,
            target_section: reader.read_u32::<LittleEndian>()?,
            target_offset: reader.read_u32::<LittleEndian>()?,
        })
    }
}

/// Parsed GR2 file
#[derive(Debug)]
pub struct Gr2File {
    /// Magic block
    pub magic: Gr2Magic,
    /// Main header
    pub header: Gr2Header,
    /// Section headers
    pub sections: Vec<SectionHeader>,
    /// Raw file data
    data: Vec<u8>,
}

impl Gr2File {
    /// Parse a GR2 file from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read entire file into memory
        let mut data = Vec::new();
        reader.seek(SeekFrom::Start(0))?;
        reader.read_to_end(&mut data)?;

        Self::from_bytes(&data)
    }

    /// Parse a GR2 file from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = std::io::Cursor::new(data);

        // Read magic block (offset 0)
        let magic = Gr2Magic::read(&mut cursor)?;
        if !magic.is_valid() {
            return Err(Error::Decompression("Invalid GR2 magic signature".to_string()));
        }

        // Read main header (offset 0x20)
        cursor.seek(SeekFrom::Start(0x20))?;
        let header = Gr2Header::read(&mut cursor)?;

        // Read section headers
        let section_header_offset = 0x20 + header.sections_offset as u64;
        cursor.seek(SeekFrom::Start(section_header_offset))?;

        let mut sections = Vec::with_capacity(header.num_sections as usize);
        for _ in 0..header.num_sections {
            sections.push(SectionHeader::read(&mut cursor)?);
        }

        Ok(Self {
            magic,
            header,
            sections,
            data: data.to_vec(),
        })
    }

    /// Get compressed data for a section
    pub fn section_compressed_data(&self, index: usize) -> Result<&[u8]> {
        let section = self.sections.get(index)
            .ok_or_else(|| Error::Decompression(format!("Invalid section index: {}", index)))?;

        if section.is_empty() {
            return Ok(&[]);
        }

        let start = section.offset_in_file as usize;
        let end = start + section.compressed_size as usize;

        if end > self.data.len() {
            return Err(Error::UnexpectedEof);
        }

        Ok(&self.data[start..end])
    }

    /// Get raw file data
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }

    /// Get pointer size
    pub fn pointer_size(&self) -> Result<PointerSize> {
        self.magic.pointer_size()
    }

    /// Get endianness
    pub fn endian(&self) -> Result<Endian> {
        self.magic.endian()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_validation() {
        let mut magic_struct = Gr2Magic {
            signature: magic::LE64,
            headers_size: 0x1F4,
            header_format: 0,
            reserved: [0; 8],
        };
        assert!(magic_struct.is_valid());
        assert_eq!(magic_struct.pointer_size().unwrap(), PointerSize::Bit64);
        assert_eq!(magic_struct.endian().unwrap(), Endian::Little);

        magic_struct.signature = magic::BE32;
        assert!(magic_struct.is_valid());
        assert_eq!(magic_struct.pointer_size().unwrap(), PointerSize::Bit32);
        assert_eq!(magic_struct.endian().unwrap(), Endian::Big);
    }
}
