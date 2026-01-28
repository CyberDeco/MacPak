//! Tile compression utilities
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use super::config::TileCompressionPreference;
use crate::compression::fastlz;
use crate::error::Result;

/// A compressed tile
#[derive(Debug, Clone)]
pub struct CompressedTile {
    /// The compression method used
    pub method: TileCompressionPreference,
    /// Compressed data
    pub data: Vec<u8>,
}

/// Compress tile data using the specified method
pub fn compress_tile(data: &[u8], preference: TileCompressionPreference) -> Result<CompressedTile> {
    match preference {
        TileCompressionPreference::Raw => Ok(CompressedTile {
            method: TileCompressionPreference::Raw,
            data: data.to_vec(),
        }),
        TileCompressionPreference::FastLZ => {
            let compressed = compress_fastlz(data)?;
            Ok(CompressedTile {
                method: TileCompressionPreference::FastLZ,
                data: compressed,
            })
        }
    }
}

/// Compress data using `FastLZ`
fn compress_fastlz(data: &[u8]) -> Result<Vec<u8>> {
    fastlz::compress(data)
}
