//! GR2 (Granny2) file operations

use crate::error::Result;
use std::path::Path;

/// Decompress raw BitKnit data
pub fn decompress_gr2(compressed: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    MacLarian::formats::gr2::decompress_gr2(compressed, expected_size)
        .map_err(|e| e.into())
}

/// Decompress a GR2 file (all sections)
pub fn decompress_file(input: &Path, output: &Path) -> Result<()> {
    MacLarian::formats::gr2::decompress_file(input, output)
        .map_err(|e| e.into())
}

/// Get GR2 file info
pub fn get_file_info(path: &Path) -> Result<(u32, Vec<MacLarian::formats::gr2::Gr2SectionInfo>)> {
    MacLarian::formats::gr2::get_file_info(path)
        .map_err(|e| e.into())
}
