//! Compression utilities

use crate::error::{Error, Result};

// Internal compression backends (not public API)
pub(crate) mod fastlz;
pub(crate) mod lz4;

/// Compress data using LZ4
///
/// # Errors
/// Returns an error if compression fails.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    Ok(lz4_flex::compress_prepend_size(data))
}

/// Decompress LZ4 data
///
/// # Errors
/// Returns an error if decompression fails.
pub fn decompress(data: &[u8], decompressed_size: usize) -> Result<Vec<u8>> {
    lz4_flex::decompress(data, decompressed_size)
        .map_err(|e| Error::DecompressionError(format!("LZ4: {e}")))
}

/// Decompress LZ4 data with size prepended
///
/// # Errors
/// Returns an error if decompression fails.
pub fn decompress_with_size(data: &[u8]) -> Result<Vec<u8>> {
    lz4_flex::decompress_size_prepended(data)
        .map_err(|e| Error::DecompressionError(format!("LZ4: {e}")))
}
