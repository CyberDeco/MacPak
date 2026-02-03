//! `FastLZ` (LZ77) decompression for virtual texture tiles

use crate::error::{Error, Result};

/// Decompress `FastLZ` data with automatic level detection.
///
/// The compression level is encoded in bits 5-7 of the first byte.
///
/// # Errors
/// Returns an error if decompression fails.
pub fn decompress(compressed: &[u8], output_size: usize) -> Result<Vec<u8>> {
    if compressed.is_empty() {
        return Ok(vec![0u8; output_size]);
    }

    fastlz_rs::decompress_to_vec(compressed, Some(output_size))
        .map_err(|e| Error::FastLzDecompressionFailed {
            message: format!("{e:?}"),
        })
}

/// Compress data using `FastLZ`.
///
/// # Errors
/// Returns an error if compression fails.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    use fastlz_rs::{CompressState, CompressionLevel};

    let mut state = CompressState::new();
    state
        .compress_to_vec(data, CompressionLevel::Level1)
        .map_err(|e| Error::FastLzCompressionFailed {
            message: e.to_string(),
        })
}
