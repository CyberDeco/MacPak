//! GTP file writer
//!
//! SPDX-FileCopyrightText: 2025 CyberDeco
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use std::io::{Write, Seek, SeekFrom};
use crate::error::Result;
use crate::virtual_texture::types::{GtpHeader, GtsCodec};
use byteorder::{LittleEndian, WriteBytesExt};

/// A chunk to be written to a page
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Codec type
    pub codec: GtsCodec,
    /// Parameter block ID
    pub parameter_block_id: u32,
    /// Compressed tile data
    pub data: Vec<u8>,
}

/// A page containing multiple chunks
#[derive(Debug, Clone, Default)]
struct Page {
    chunks: Vec<Chunk>,
    /// Current size used in this page (including headers)
    used_size: u32,
}

/// GTP file writer
pub struct GtpWriter {
    guid: [u8; 16],
    page_size: u32,
    pages: Vec<Page>,
    current_page: usize,
}

impl GtpWriter {
    /// Create a new GTP writer
    #[must_use]
    pub fn new(guid: [u8; 16], page_size: u32) -> Self {
        Self {
            guid,
            page_size,
            pages: vec![Page::default()],
            current_page: 0,
        }
    }

    /// Add a tile chunk and return (page_index, chunk_index)
    pub fn add_chunk(&mut self, chunk: Chunk) -> (u16, u16) {
        // Calculate space needed for this chunk
        // Chunk header (12 bytes) + data
        let chunk_size = 12 + chunk.data.len() as u32;

        // Calculate space needed for page header with one more chunk
        // Page header: chunk_count (4) + offsets (4 * num_chunks)
        let current_page = &self.pages[self.current_page];
        let new_header_size = 4 + 4 * (current_page.chunks.len() as u32 + 1);

        // Check if chunk fits in current page
        let total_needed = new_header_size + current_page.used_size + chunk_size;

        if total_needed > self.page_size && !current_page.chunks.is_empty() {
            // Need a new page
            self.pages.push(Page::default());
            self.current_page = self.pages.len() - 1;
        }

        let page_idx = self.current_page as u16;
        let chunk_idx = self.pages[self.current_page].chunks.len() as u16;

        // Add chunk to current page
        let page = &mut self.pages[self.current_page];
        page.used_size += chunk_size;
        page.chunks.push(chunk);

        (page_idx, chunk_idx)
    }

    /// Get the number of pages
    #[must_use]
    pub fn num_pages(&self) -> u32 {
        self.pages.len() as u32
    }

    /// Write the GTP file
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // Write header at offset 0 (part of page 0)
        writer.write_u32::<LittleEndian>(GtpHeader::MAGIC)?;
        writer.write_u32::<LittleEndian>(4)?; // Version
        writer.write_all(&self.guid)?;

        // Write pages
        // Page 0 starts at offset 0, with GTP header (24 bytes) followed by chunk data
        // Pages 1+ start at offset page_size, 2*page_size, etc.
        for (page_idx, page) in self.pages.iter().enumerate() {
            let page_start = (page_idx as u64) * (self.page_size as u64);
            let data_start = if page_idx == 0 { 24 } else { page_start };

            // Seek to data start (after header for page 0)
            writer.seek(SeekFrom::Start(data_start))?;

            // Write chunk count
            writer.write_u32::<LittleEndian>(page.chunks.len() as u32)?;

            // Calculate and write chunk offsets
            // Offsets are relative to page_start (0 for page 0), not data_start
            // For page 0: first chunk is at 24 (GTP header) + 4 (count) + 4*num_chunks (offsets)
            // For page N: first chunk is at 4 (count) + 4*num_chunks (offsets)
            let page_header_size = if page_idx == 0 { 24 } else { 0 };
            let chunk_table_size = 4 + 4 * page.chunks.len();
            let mut offset = (page_header_size + chunk_table_size) as u32;

            for chunk in &page.chunks {
                writer.write_u32::<LittleEndian>(offset)?;
                offset += 12 + chunk.data.len() as u32;
            }

            // Write chunks
            for chunk in &page.chunks {
                // Chunk header
                writer.write_u32::<LittleEndian>(chunk.codec as u32)?;
                writer.write_u32::<LittleEndian>(chunk.parameter_block_id)?;
                writer.write_u32::<LittleEndian>(chunk.data.len() as u32)?;

                // Chunk data
                writer.write_all(&chunk.data)?;
            }

            // Pad to page size
            let current_pos = writer.stream_position()?;
            let page_end = page_start + self.page_size as u64;
            if current_pos < page_end {
                let padding = (page_end - current_pos) as usize;
                let zeros = vec![0u8; padding];
                writer.write_all(&zeros)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtp_writer_single_chunk() {
        let mut writer = GtpWriter::new([0u8; 16], 0x10000); // 64KB pages

        let chunk = Chunk {
            codec: GtsCodec::Bc,
            parameter_block_id: 0,
            data: vec![1, 2, 3, 4],
        };

        let (page, chunk_idx) = writer.add_chunk(chunk);
        assert_eq!(page, 0);
        assert_eq!(chunk_idx, 0);
        assert_eq!(writer.num_pages(), 1);
    }

    #[test]
    fn test_gtp_writer_multiple_pages() {
        let mut writer = GtpWriter::new([0u8; 16], 100); // Tiny pages for testing

        // Add chunks until we need a new page
        for i in 0..10 {
            let chunk = Chunk {
                codec: GtsCodec::Bc,
                parameter_block_id: 0,
                data: vec![i; 20], // 20 bytes of data
            };
            writer.add_chunk(chunk);
        }

        // Should have needed multiple pages
        assert!(writer.num_pages() > 1);
    }
}
