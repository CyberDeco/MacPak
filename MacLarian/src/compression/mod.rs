//! Compression utilities

use crate::error::{Error, Result};

pub mod lz4;

/// Compress data using LZ4
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    Ok(lz4_flex::compress_prepend_size(data))
}

/// Decompress LZ4 data
pub fn decompress(data: &[u8], decompressed_size: usize) -> Result<Vec<u8>> {
    lz4_flex::decompress(data, decompressed_size)
        .map_err(|e| Error::DecompressionError(format!("LZ4: {}", e)))
}

/// Decompress LZ4 data with size prepended
pub fn decompress_with_size(data: &[u8]) -> Result<Vec<u8>> {
    lz4_flex::decompress_size_prepended(data)
        .map_err(|e| Error::DecompressionError(format!("LZ4: {}", e)))
}
