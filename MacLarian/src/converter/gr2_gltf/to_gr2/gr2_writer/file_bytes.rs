//! GR2 file bytes generation

use crate::error::Result;

use super::super::utils::crc32;
use super::Gr2Writer;
use super::constants::{MAGIC_LE64, NUM_SECTIONS, TAG_BG3, VERSION};
use super::section::Section;

impl Gr2Writer {
    pub(super) fn build_file_bytes(
        &self,
        sections_data: &(Vec<Section>, u32, u32),
    ) -> Result<Vec<u8>> {
        let (sections, root_offset, root_type_offset) = sections_data;

        // Calculate offsets - NO COMPRESSION, write uncompressed
        let magic_size = 32;
        let header_size = 72; // v7 header
        let section_header_size = 44 * NUM_SECTIONS as usize;
        let headers_total = magic_size + header_size + section_header_size;

        // Align to 16 bytes
        let data_start = (headers_total + 15) & !15;

        // Calculate section offsets (uncompressed)
        let mut section_offsets = Vec::new();
        let mut current_offset = data_start;
        for section in sections {
            section_offsets.push(current_offset);
            current_offset += section.len();
            // Align each section to 4 bytes
            current_offset = (current_offset + 3) & !3;
        }

        // Calculate relocation table offsets (uncompressed)
        let mut reloc_offsets = Vec::new();
        for section in sections {
            reloc_offsets.push(current_offset);
            if !section.fixups.is_empty() {
                current_offset += section.fixups.len() * 12;
            }
        }

        let file_size = current_offset;

        // Build output buffer
        let mut output = Vec::with_capacity(file_size);

        // Write magic block (32 bytes)
        output.extend_from_slice(&MAGIC_LE64);
        output.extend_from_slice(&(data_start as u32).to_le_bytes());
        output.extend_from_slice(&0u32.to_le_bytes()); // header_format (uncompressed)
        output.extend_from_slice(&[0u8; 8]);

        // Write header (72 bytes for v7)
        output.extend_from_slice(&VERSION.to_le_bytes());
        output.extend_from_slice(&(file_size as u32).to_le_bytes());
        output.extend_from_slice(&0u32.to_le_bytes()); // CRC placeholder
        output.extend_from_slice(&(header_size as u32).to_le_bytes());
        output.extend_from_slice(&NUM_SECTIONS.to_le_bytes());
        // root_type reference
        output.extend_from_slice(&4u32.to_le_bytes()); // section 4
        output.extend_from_slice(&root_type_offset.to_le_bytes());
        // root_node reference
        output.extend_from_slice(&0u32.to_le_bytes()); // section 0
        output.extend_from_slice(&root_offset.to_le_bytes());
        // tag
        output.extend_from_slice(&TAG_BG3.to_le_bytes());
        // extra_tags
        output.extend_from_slice(&[0u8; 16]);
        // string_table_crc
        output.extend_from_slice(&0u32.to_le_bytes());
        // reserved (12 bytes)
        output.extend_from_slice(&[0u8; 12]);

        // Write section headers (uncompressed format)
        for (i, section) in sections.iter().enumerate() {
            output.extend_from_slice(&0u32.to_le_bytes()); // compression = 0 (none)
            output.extend_from_slice(&(section_offsets[i] as u32).to_le_bytes());
            output.extend_from_slice(&(section.len() as u32).to_le_bytes()); // compressed = uncompressed
            output.extend_from_slice(&(section.len() as u32).to_le_bytes());
            output.extend_from_slice(&4u32.to_le_bytes()); // alignment
            output.extend_from_slice(&0u32.to_le_bytes()); // first_16bit
            output.extend_from_slice(&0u32.to_le_bytes()); // first_8bit
            output.extend_from_slice(&(reloc_offsets[i] as u32).to_le_bytes());
            output.extend_from_slice(&(section.fixups.len() as u32).to_le_bytes());
            output.extend_from_slice(&0u32.to_le_bytes()); // mixed_marshalling_offset
            output.extend_from_slice(&0u32.to_le_bytes()); // num_mixed_marshalling
        }

        // Pad to data_start
        while output.len() < data_start {
            output.push(0);
        }

        // Write section data (uncompressed)
        for (i, section) in sections.iter().enumerate() {
            while output.len() < section_offsets[i] {
                output.push(0);
            }
            output.extend_from_slice(&section.data);
            // Align to 4 bytes
            while output.len() % 4 != 0 {
                output.push(0);
            }
        }

        // Write relocation tables (uncompressed)
        for (i, section) in sections.iter().enumerate() {
            if !section.fixups.is_empty() {
                while output.len() < reloc_offsets[i] {
                    output.push(0);
                }
                for fixup in &section.fixups {
                    output.extend_from_slice(&fixup.offset_in_section.to_le_bytes());
                    output.extend_from_slice(&fixup.target_section.to_le_bytes());
                    output.extend_from_slice(&fixup.target_offset.to_le_bytes());
                }
            }
        }

        // Calculate and update CRC
        let crc = crc32(&output[0x20 + 8..]);
        output[0x20 + 8..0x20 + 12].copy_from_slice(&crc.to_le_bytes());

        Ok(output)
    }
}
