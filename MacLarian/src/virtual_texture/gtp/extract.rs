//! Chunk extraction and decompression.

use super::super::gts::GtsFile;
use super::super::types::{GtpChunkHeader, GtsCodec, TileCompression};
use super::GtpFile;
use crate::compression::fastlz;
use crate::error::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

impl<R: Read + Seek> GtpFile<R> {
    /// Extract and decompress a single chunk.
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
            return Err(Error::InvalidPageIndex { index: page_index });
        }

        if chunk_index >= self.chunk_offsets[page_index].len() {
            return Err(Error::InvalidChunkIndex { index: chunk_index });
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
            GtsCodec::Bc => {
                let method = gts.get_compression_method(chunk_header.parameter_block_id);

                // Calculate expected output size for BC5/DXT5
                // BC5: 16 bytes per 4x4 block
                let main_size = 16
                    * (self.tile_width as usize).div_ceil(4)
                    * (self.tile_height as usize).div_ceil(4);
                // Add embedded mipmap size
                let mip_size = 16
                    * (self.tile_width as usize / 2).div_ceil(4)
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

    pub(super) fn read_chunk_header(&mut self) -> Result<GtpChunkHeader> {
        let mut buf4 = [0u8; 4];

        self.reader.read_exact(&mut buf4)?;
        let codec_val = u32::from_le_bytes(buf4);
        self.reader.read_exact(&mut buf4)?;
        let parameter_block_id = u32::from_le_bytes(buf4);
        self.reader.read_exact(&mut buf4)?;
        let size = u32::from_le_bytes(buf4);

        let codec = GtsCodec::from_u32(codec_val).unwrap_or(GtsCodec::Bc);

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
                lz4_flex::decompress(compressed, output_size).map_err(|e| {
                    Error::Lz4DecompressionFailed {
                        message: e.to_string(),
                    }
                })
            }
            TileCompression::FastLZ => fastlz::decompress(compressed, output_size),
        }
    }
}
