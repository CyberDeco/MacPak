//! Decompression utilities for PAK file data

use crate::error::{Error, Result};
use super::super::lspk::CompressionMethod;
use std::io::Read;

/// Standalone LZ4 decompression (for parallel use)
pub fn decompress_lz4_standalone(compressed: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    // Try standard block decompression first
    if let Ok(data) = lz4_flex::block::decompress(compressed, expected_size) {
        return Ok(data);
    }

    // Try with a larger buffer
    let larger_size = expected_size.saturating_mul(2).max(65536);
    if let Ok(data) = lz4_flex::block::decompress(compressed, larger_size) {
        return Ok(data);
    }

    // Try decompressing without size hint
    if let Ok(data) = lz4_flex::decompress_size_prepended(compressed) {
        return Ok(data);
    }

    // Try treating it as a frame
    let mut decoder = lz4_flex::frame::FrameDecoder::new(compressed);
    let mut decompressed = Vec::with_capacity(expected_size);
    if decoder.read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
        return Ok(decompressed);
    }

    Err(Error::DecompressionError(format!(
        "Failed to decompress LZ4 data: all methods failed (compressed: {} bytes, expected: {} bytes)",
        compressed.len(),
        expected_size
    )))
}

/// Standalone Zlib decompression (for parallel use)
pub fn decompress_zlib_standalone(compressed: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;

    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::with_capacity(expected_size);

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| Error::DecompressionError(format!("Failed to decompress Zlib data: {e}")))?;

    Ok(decompressed)
}

/// Decompress data based on compression method (standalone for parallel use)
pub fn decompress_data(
    compressed: &[u8],
    compression: CompressionMethod,
    size_decompressed: u32,
) -> Result<Vec<u8>> {
    if compression == CompressionMethod::None || size_decompressed == 0 {
        return Ok(compressed.to_vec());
    }

    match compression {
        CompressionMethod::None => Ok(compressed.to_vec()),
        CompressionMethod::Lz4 => decompress_lz4_standalone(compressed, size_decompressed as usize),
        CompressionMethod::Zlib => decompress_zlib_standalone(compressed, size_decompressed as usize),
    }
}
