//! GTP (Game Texture Page) file reader
//!
//! GTP files contain the actual tile data for virtual textures.
//! Each file consists of pages, and each page contains multiple chunks (tiles).
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::compression::fastlz;
use crate::error::{Error, Result};
use super::gts::GtsFile;
use super::types::{GtpHeader, GtsCodec, GtpChunkHeader, TileCompression};

/// GTP file reader
pub struct GtpFile<R: Read + Seek> {
    reader: BufReader<R>,
    pub header: GtpHeader,
    /// Chunk offsets for each page, indexed by page then chunk
    pub chunk_offsets: Vec<Vec<u32>>,
    page_size: u32,
    tile_width: i32,
    tile_height: i32,
}

impl GtpFile<File> {
    /// Open a GTP file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or has an invalid format.
    pub fn open<P: AsRef<Path>>(path: P, gts: &GtsFile) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        Self::new(file, gts)
    }
}

impl<R: Read + Seek> GtpFile<R> {
    /// Create a new GTP reader from any Read + Seek source
    ///
    /// # Errors
    /// Returns an error if reading fails or the data has an invalid format.
    pub fn new(reader: R, gts: &GtsFile) -> Result<Self> {
        let mut reader = BufReader::new(reader);

        // Get file size
        let file_size = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        // Read header
        let header = Self::read_header(&mut reader)?;

        if header.magic != GtpHeader::MAGIC {
            let magic = header.magic;
            let expected = GtpHeader::MAGIC;
            return Err(Error::ConversionError(format!(
                "Invalid GTP magic: 0x{magic:08X}, expected 0x{expected:08X}"
            )));
        }

        let page_size = gts.header.page_size;
        let num_pages = (file_size / u64::from(page_size)) as usize;

        // Read chunk offsets for each page
        let chunk_offsets = Self::read_chunk_offsets(&mut reader, page_size, num_pages)?;

        Ok(Self {
            reader,
            header,
            chunk_offsets,
            page_size,
            tile_width: gts.header.tile_width,
            tile_height: gts.header.tile_height,
        })
    }

    fn read_header<RR: Read>(reader: &mut RR) -> Result<GtpHeader> {
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

    fn read_chunk_offsets<RR: Read + Seek>(
        reader: &mut RR,
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

    /// Extract and decompress a single chunk
    ///
    /// # Errors
    /// Returns an error if the chunk is out of range or decompression fails.
    pub fn extract_chunk(
        &mut self,
        page_index: usize,
        chunk_index: usize,
        gts: &GtsFile,
    ) -> Result<Vec<u8>> {
        if page_index >= self.chunk_offsets.len() {
            let max = self.chunk_offsets.len();
            return Err(Error::ConversionError(format!(
                "Page index {page_index} out of range (max {max})"
            )));
        }

        if chunk_index >= self.chunk_offsets[page_index].len() {
            let max = self.chunk_offsets[page_index].len();
            return Err(Error::ConversionError(format!(
                "Chunk index {chunk_index} out of range for page {page_index} (max {max})"
            )));
        }

        let page_start = (page_index as u64) * u64::from(self.page_size);
        let chunk_offset = self.chunk_offsets[page_index][chunk_index];
        let absolute_offset = page_start + u64::from(chunk_offset);

        self.reader.seek(SeekFrom::Start(absolute_offset))?;

        // Read chunk header
        let chunk_header = self.read_chunk_header()?;

        // Read compressed data
        let mut compressed = vec![0u8; chunk_header.size as usize];
        self.reader.read_exact(&mut compressed)?;

        // Decompress based on codec
        match chunk_header.codec {
            GtsCodec::BC => {
                let method = gts.get_compression_method(chunk_header.parameter_block_id);

                // Calculate expected output size for BC5/DXT5
                // BC5: 16 bytes per 4x4 block
                let main_size = 16 * (self.tile_width as usize).div_ceil(4)
                                   * (self.tile_height as usize).div_ceil(4);
                // Add embedded mipmap size
                let mip_size = 16 * (self.tile_width as usize / 2).div_ceil(4)
                                  * (self.tile_height as usize / 2).div_ceil(4);
                let output_size = main_size + mip_size;

                self.decompress_tile(&compressed, output_size, method)
            }
            GtsCodec::Uniform => {
                // Uniform tiles are solid color, return zeros
                let size = (self.tile_width * self.tile_height) as usize;
                Ok(vec![0u8; size])
            }
            _ => {
                // Unknown codec, return raw data
                Ok(compressed)
            }
        }
    }

    fn read_chunk_header(&mut self) -> Result<GtpChunkHeader> {
        let mut buf4 = [0u8; 4];

        self.reader.read_exact(&mut buf4)?;
        let codec_val = u32::from_le_bytes(buf4);
        self.reader.read_exact(&mut buf4)?;
        let parameter_block_id = u32::from_le_bytes(buf4);
        self.reader.read_exact(&mut buf4)?;
        let size = u32::from_le_bytes(buf4);

        let codec = GtsCodec::from_u32(codec_val).unwrap_or(GtsCodec::BC);

        Ok(GtpChunkHeader {
            codec,
            parameter_block_id,
            size,
        })
    }

    fn decompress_tile(
        &self,
        compressed: &[u8],
        output_size: usize,
        method: TileCompression,
    ) -> Result<Vec<u8>> {
        match method {
            TileCompression::Raw => Ok(compressed.to_vec()),
            TileCompression::Lz4 => {
                lz4_flex::decompress(compressed, output_size)
                    .map_err(|e| Error::DecompressionError(format!("LZ4: {e}")))
            }
            TileCompression::FastLZ => {
                fastlz::decompress(compressed, output_size)
            }
        }
    }

    /// Get the number of pages in this GTP file
    pub fn num_pages(&self) -> usize {
        self.chunk_offsets.len()
    }

    /// Get the number of chunks in a specific page
    pub fn num_chunks(&self, page_index: usize) -> usize {
        self.chunk_offsets.get(page_index).map_or(0, std::vec::Vec::len)
    }
}
