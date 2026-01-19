//! Compression utilities

use crate::error::{Error, Result};

pub mod lz4;
pub mod fastlz;
// pub mod zstd;

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

// Decompress zstd for ?
// pub fn decompress_zstd(&self, compressed: &[u8], _expected_size: usize, path: &PathBuf) -> Result<Vec<u8>> {
//       let mut decoder = StreamingDecoder::new(compressed)
//           .map_err(|e| Error::DecompressionError(format!(
//               "Failed to init Zstd decoder for {}: {:?}", path.display(), e
//           )))?;

//       let mut result = Vec::new();
//       decoder.read_to_end(&mut result)
//           .map_err(|e| Error::DecompressionError(format!(
//               "Failed to decompress Zstd data for {}: {}", path.display(), e
//           )))?;

//       Ok(result)
//   }

