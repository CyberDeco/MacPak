//! GR2 (Granny2) file operations

use crate::error::Result;

/// Decompress GR2 files
pub fn decompress_gr2(compressed: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    MacLarian::formats::gr2::decompress_gr2(compressed, expected_size)
        .map_err(|e| e.into())
}
