//! Tile compression utilities
//!
//! SPDX-FileCopyrightText: 2025 CyberDeco
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use crate::error::Result;
use super::config::TileCompressionPreference;

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
        TileCompressionPreference::Raw => {
            Ok(CompressedTile {
                method: TileCompressionPreference::Raw,
                data: data.to_vec(),
            })
        }
        TileCompressionPreference::Lz4 => {
            // Use compress() without prepended size - extractor expects raw LZ4 frames
            let compressed = lz4_flex::compress(data);
            Ok(CompressedTile {
                method: TileCompressionPreference::Lz4,
                data: compressed,
            })
        }
        TileCompressionPreference::FastLZ => {
            let compressed = compress_fastlz(data)?;
            Ok(CompressedTile {
                method: TileCompressionPreference::FastLZ,
                data: compressed,
            })
        }
        TileCompressionPreference::Best => {
            // Use LZ4 - FastLZ compression has compatibility issues with BG3's decompressor
            // TODO: Investigate fastlz_rs compression level/format to match BG3
            let compressed = lz4_flex::compress(data);
            Ok(CompressedTile {
                method: TileCompressionPreference::Lz4,
                data: compressed,
            })
        }
    }
}

/// Compress data using FastLZ
fn compress_fastlz(data: &[u8]) -> Result<Vec<u8>> {
    use fastlz_rs::{CompressState, CompressionLevel};
    use crate::error::Error;

    let mut state = CompressState::new();
    state.compress_to_vec(data, CompressionLevel::Default)
        .map_err(|e| Error::CompressionError(format!("FastLZ: {e}")))
}
