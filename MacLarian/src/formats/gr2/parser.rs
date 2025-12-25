//! GR2 File Parser - Complete Implementation
//!
//! Parses GR2 file structure and extracts sections for decompression.
//!
//! Based on reverse-engineered file format documentation:
//! - Magic header (32 bytes) with 5 format variants
//! - Extended header (72 bytes normalized)
//! - Section table (44 bytes per section)
//! - Section data (raw or BitKnit compressed)

use crate::formats::gr2::decompressor::GR2Decompressor;
use std::collections::HashMap;
use std::path::Path;

// =============================================================================
// MAGIC SIGNATURES
// =============================================================================

lazy_static::lazy_static! {
    static ref MAGIC_SIGNATURES: HashMap<[u8; 16], FormatInfo> = {
        let mut m = HashMap::new();

        m.insert(
            [0xB8, 0x67, 0xB0, 0xCA, 0xF8, 0x6D, 0xB1, 0x0F, 0x84, 0x72, 0x8C, 0x7E, 0x5E, 0x19, 0x00, 0x1E],
            FormatInfo {
                name: "Old/Legacy".to_string(),
                extended_header_size: 32,
                pointer_size: 32,
                endian: Endianness::Little,
            },
        );

        m.insert(
            [0x29, 0xDE, 0x6C, 0xC0, 0xBA, 0xA4, 0x53, 0x2B, 0x25, 0xF5, 0xB7, 0xA5, 0xF6, 0x66, 0xE2, 0xEE],
            FormatInfo {
                name: "32-bit Little Endian".to_string(),
                extended_header_size: 32,
                pointer_size: 32,
                endian: Endianness::Little,
            },
        );

        m.insert(
            [0x0E, 0x11, 0x95, 0xB5, 0x6A, 0xA5, 0xB5, 0x4B, 0xEB, 0x28, 0x28, 0x50, 0x25, 0x78, 0xB3, 0x04],
            FormatInfo {
                name: "32-bit Big Endian".to_string(),
                extended_header_size: 32,
                pointer_size: 32,
                endian: Endianness::Big,
            },
        );

        m.insert(
            [0xE5, 0x9B, 0x49, 0x5E, 0x6F, 0x63, 0x1F, 0x14, 0x1E, 0x13, 0xEB, 0xA9, 0x90, 0xBE, 0xED, 0xC4],
            FormatInfo {
                name: "64-bit Little Endian".to_string(),
                extended_header_size: 64,
                pointer_size: 64,
                endian: Endianness::Little,
            },
        );

        m.insert(
            [0x31, 0x95, 0xD4, 0xE3, 0x20, 0xDC, 0x4F, 0x62, 0xCC, 0x36, 0xD0, 0x3A, 0xB1, 0x82, 0xFF, 0x89],
            FormatInfo {
                name: "64-bit Big Endian".to_string(),
                extended_header_size: 64,
                pointer_size: 64,
                endian: Endianness::Big,
            },
        );

        m
    };
}

// Section types
const SECTION_NAMES: [&str; 8] = [
    "MainSection",           // 0: FileInfo, Models, Types
    "RigidVertexSection",    // 1: Static mesh vertices
    "RigidIndexSection",     // 2: Static mesh indices
    "DeformableVertexSection", // 3: Skinned vertices
    "DeformableIndexSection", // 4: Skinned indices
    "TextureSection",        // 5: Embedded textures
    "DiscardableSection",    // 6: Optional data
    "UnloadedSection",       // 7: Streaming data
];

// Compression types
const COMPRESSION_NONE: u32 = 0;
const COMPRESSION_BITKNIT: u32 = 4;

// =============================================================================
// TYPES
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub name: String,
    pub extended_header_size: u32,
    pub pointer_size: u32,
    pub endian: Endianness,
}

// =============================================================================
// GR2 SECTION STRUCTURE
// =============================================================================

/// Represents a GR2 file section
#[derive(Debug, Clone)]
pub struct GR2Section {
    pub index: usize,
    pub compression_type: u32,
    pub data_offset: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub alignment: u32,
    pub name: String,
}

impl GR2Section {
    pub fn new(index: usize, descriptor: &SectionDescriptor) -> Self {
        let name = if index < SECTION_NAMES.len() {
            SECTION_NAMES[index].to_string()
        } else {
            format!("Section{}", index)
        };

        Self {
            index,
            compression_type: descriptor.compression_type,
            data_offset: descriptor.data_offset,
            compressed_size: descriptor.compressed_size,
            uncompressed_size: descriptor.uncompressed_size,
            alignment: descriptor.alignment,
            name,
        }
    }

    pub fn compression_name(&self) -> &str {
        match self.compression_type {
            COMPRESSION_NONE => "None",
            COMPRESSION_BITKNIT => "BitKnit",
            _ => "Unknown",
        }
    }
}

impl std::fmt::Display for GR2Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<Section {}: {}, compression={}, offset=0x{:08x}, size={}→{}>",
            self.index,
            self.name,
            self.compression_name(),
            self.data_offset,
            self.compressed_size,
            self.uncompressed_size
        )
    }
}

#[derive(Debug)]
struct SectionDescriptor {
    compression_type: u32,
    data_offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    alignment: u32,
}

// =============================================================================
// GR2 FILE STRUCTURE
// =============================================================================

/// Parser for GR2 files
pub struct GR2File {
    pub filepath: std::path::PathBuf,
    pub format_name: String,
    pub format_info: FormatInfo,
    pub extended_header_size: u32,
    pub pointer_size: u32,
    pub endian: Endianness,

    // Header data
    pub format_revision: u32,
    pub header_format: u32,
    pub file_size: u32,
    pub crc32: u32,
    pub section_count: u32,
    pub section_array_count: u32,

    // Sections
    pub sections: Vec<GR2Section>,

    // File data
    file_data: Vec<u8>,

    // CRITICAL FIX: Create ONE decompressor for entire file
    // This preserves the distance cache across all sections
    decompressor: Option<GR2Decompressor>,
}

impl GR2File {
    /// Parse a GR2 file from the given path
    pub fn new<P: AsRef<Path>>(filepath: P) -> Result<Self, String> {
        let filepath = filepath.as_ref().to_path_buf();
        let file_data = std::fs::read(&filepath)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        if file_data.len() < 32 {
            return Err("File too small to be a valid GR2 file".to_string());
        }

        let mut gr2 = Self {
            filepath,
            format_name: String::new(),
            format_info: FormatInfo {
                name: String::new(),
                extended_header_size: 0,
                pointer_size: 0,
                endian: Endianness::Little,
            },
            extended_header_size: 0,
            pointer_size: 0,
            endian: Endianness::Little,
            format_revision: 0,
            header_format: 0,
            file_size: 0,
            crc32: 0,
            section_count: 0,
            section_array_count: 0,
            sections: Vec::new(),
            file_data,
            decompressor: None,
        };

        gr2.parse()?;
        Ok(gr2)
    }

    fn parse(&mut self) -> Result<(), String> {
        self.parse_magic_header()?;
        self.parse_extended_header()?;
        self.parse_section_table()?;
        Ok(())
    }

    fn parse_magic_header(&mut self) -> Result<(), String> {
        let mut magic_sig = [0u8; 16];
        magic_sig.copy_from_slice(&self.file_data[0..16]);

        let format_info = MAGIC_SIGNATURES
            .get(&magic_sig)
            .ok_or_else(|| format!("Invalid GR2 magic signature: {:02x?}", magic_sig))?;

        self.format_info = format_info.clone();
        self.format_name = format_info.name.clone();
        self.extended_header_size = format_info.extended_header_size;
        self.pointer_size = format_info.pointer_size;
        self.endian = format_info.endian;

        // Read header format field
        self.header_format = u32::from_le_bytes([
            self.file_data[16],
            self.file_data[17],
            self.file_data[18],
            self.file_data[19],
        ]);

        if self.header_format != 0 {
            println!(
                "Note: Extended header format {} (file has additional features)",
                self.header_format
            );
        }

        Ok(())
    }

    fn parse_extended_header(&mut self) -> Result<(), String> {
        // Extended header starts at offset 0x20 (32)
        let ext_header_start = 32;
        let ext_header_end = ext_header_start + 72;

        if self.file_data.len() < ext_header_end {
            return Err("Extended header too small".to_string());
        }

        let ext_header = &self.file_data[ext_header_start..ext_header_end];

        // Parse key fields (using little endian for now)
        self.format_revision = u32::from_le_bytes([
            ext_header[0],
            ext_header[1],
            ext_header[2],
            ext_header[3],
        ]);

        self.file_size = u32::from_le_bytes([
            ext_header[4],
            ext_header[5],
            ext_header[6],
            ext_header[7],
        ]);

        self.crc32 = u32::from_le_bytes([
            ext_header[8],
            ext_header[9],
            ext_header[10],
            ext_header[11],
        ]);

        // Section count location depends on format
        if self.pointer_size == 64 {
            self.section_count = u32::from_le_bytes([
                ext_header[16],
                ext_header[17],
                ext_header[18],
                ext_header[19],
            ]);
            self.section_array_count = u32::from_le_bytes([
                ext_header[20],
                ext_header[21],
                ext_header[22],
                ext_header[23],
            ]);
        } else {
            self.section_count = u32::from_le_bytes([
                ext_header[44],
                ext_header[45],
                ext_header[46],
                ext_header[47],
            ]);
            self.section_array_count = self.section_count;
        }

        // Validate
        if self.format_revision != 7 {
            println!(
                "Warning: Unexpected format revision {} (expected 7)",
                self.format_revision
            );
        }

        if self.section_count > 31 {
            return Err(format!("Invalid section count: {}", self.section_count));
        }

        if self.section_count == 0 {
            println!("Warning: File has 0 sections (unusual but valid)");
        }

        Ok(())
    }

    fn parse_section_table(&mut self) -> Result<(), String> {
        // Section table starts at file offset 0x68 (104)
        let section_table_offset = 0x68;

        for i in 0..self.section_count as usize {
            let offset = section_table_offset + (i * 44);
            let section_end = offset + 44;

            if self.file_data.len() < section_end {
                return Err(format!("Section {} descriptor truncated", i));
            }

            let section_data = &self.file_data[offset..section_end];

            // Parse section descriptor (44 bytes)
            let descriptor = SectionDescriptor {
                compression_type: u32::from_le_bytes([
                    section_data[0],
                    section_data[1],
                    section_data[2],
                    section_data[3],
                ]),
                data_offset: u32::from_le_bytes([
                    section_data[4],
                    section_data[5],
                    section_data[6],
                    section_data[7],
                ]),
                compressed_size: u32::from_le_bytes([
                    section_data[8],
                    section_data[9],
                    section_data[10],
                    section_data[11],
                ]),
                uncompressed_size: u32::from_le_bytes([
                    section_data[12],
                    section_data[13],
                    section_data[14],
                    section_data[15],
                ]),
                alignment: u32::from_le_bytes([
                    section_data[16],
                    section_data[17],
                    section_data[18],
                    section_data[19],
                ]),
            };

            let section = GR2Section::new(i, &descriptor);
            self.sections.push(section);
        }

        Ok(())
    }

    /// Get raw (possibly compressed) section data
    pub fn get_raw_section_data(&self, section_index: usize) -> Option<&[u8]> {
        if section_index >= self.sections.len() {
            eprintln!("Error: Invalid section index {}", section_index);
            return None;
        }

        let section = &self.sections[section_index];

        // Extract data from file
        let start = section.data_offset as usize;
        let end = start + section.compressed_size as usize;

        if end > self.file_data.len() {
            eprintln!("Error: Section {} data extends beyond file", section_index);
            return None;
        }

        Some(&self.file_data[start..end])
    }

    /// Get decompressed section data
    pub fn get_decompressed_section(&mut self, section_index: usize) -> Option<Vec<u8>> {
        if section_index >= self.sections.len() {
            eprintln!("Error: Invalid section index {}", section_index);
            return None;
        }

        // Clone section info to avoid borrow conflicts
        let compression_type = self.sections[section_index].compression_type;
        let section_name = self.sections[section_index].name.clone();
        let uncompressed_size = self.sections[section_index].uncompressed_size;

        // Get raw data (this borrows self)
        let raw_data = self.get_raw_section_data(section_index)?.to_vec();

        // Now we can modify self again
        match compression_type {
            COMPRESSION_NONE => {
                // No compression
                Some(raw_data)
            }
            COMPRESSION_BITKNIT => {
                // BitKnit compression - decompress
                // CRITICAL FIX: Use persistent decompressor
                if self.decompressor.is_none() {
                    self.decompressor = Some(GR2Decompressor::new());
                    println!("[INFO] Created persistent decompressor for GR2 file");
                }

                println!("Decompressing section {} ({})...", section_index, section_name);
                println!("  Input size: {} bytes", raw_data.len());
                println!("  Expected output: {} bytes", uncompressed_size);

                let decompressed = self.decompressor.as_mut().unwrap().decompress_section(
                    &raw_data,
                    uncompressed_size as usize,
                );

                if let Some(ref data) = decompressed {
                    println!("  Success: {} bytes", data.len());
                } else {
                    println!("  Failed to decompress!");
                }

                decompressed
            }
            _ => {
                eprintln!(
                    "Error: Unknown compression type {}",
                    compression_type
                );
                None
            }
        }
    }

    /// Extract and decompress all sections to files
    pub fn extract_all_sections<P: AsRef<Path>>(
        &mut self,
        output_dir: P,
    ) -> Result<(), String> {
        let output_path = output_dir.as_ref();
        std::fs::create_dir_all(output_path)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        println!("\nExtracting sections from: {}", self.filepath.display());
        println!("Output directory: {}", output_path.display());
        println!("{}", "=".repeat(60));

        for i in 0..self.sections.len() {
            // Clone section name to avoid borrow conflicts
            let section_name = self.sections[i].name.clone();
            println!("\n[Section {}: {}]", i, section_name);

            if let Some(data) = self.get_decompressed_section(i) {
                // Save to file
                let filename = format!("section_{:02}_{}.bin", i, section_name);
                let output_file = output_path.join(&filename);

                std::fs::write(&output_file, &data)
                    .map_err(|e| format!("Failed to write file: {}", e))?;

                println!("  Saved to: {}", filename);
                println!("  Size: {} bytes", data.len());
            } else {
                println!("  Failed to extract");
            }
        }

        println!("\n{}", "=".repeat(60));
        println!("Extraction complete!");

        Ok(())
    }

    /// Print file information
    pub fn print_info(&self) {
        println!("{}", "=".repeat(60));
        println!("GR2 File: {}", self.filepath.display());
        println!("{}", "=".repeat(60));
        println!("Format: {}", self.format_name);
        println!("Format Revision: {}", self.format_revision);
        println!("Header Format: {}", self.header_format);
        println!("File Size: {} bytes", self.file_size);
        println!("CRC32: 0x{:08X}", self.crc32);
        println!("Pointer Size: {}-bit", self.pointer_size);
        println!("Endianness: {:?}", self.endian);
        println!("Section Count: {}", self.section_count);
        println!();

        println!("Sections:");
        println!("{}", "-".repeat(60));
        for section in &self.sections {
            println!(
                "  {}: {:<25} Compression: {:<8} Size: {:>7} → {:>7}",
                section.index,
                section.name,
                section.compression_name(),
                section.compressed_size,
                section.uncompressed_size
            );
        }
        println!("{}", "=".repeat(60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_signatures() {
        // Verify that all magic signatures are 16 bytes
        for (sig, info) in MAGIC_SIGNATURES.iter() {
            assert_eq!(sig.len(), 16);
            assert!(!info.name.is_empty());
        }
    }

    #[test]
    fn test_section_names() {
        assert_eq!(SECTION_NAMES.len(), 8);
        assert_eq!(SECTION_NAMES[0], "MainSection");
        assert_eq!(SECTION_NAMES[7], "UnloadedSection");
    }
}
