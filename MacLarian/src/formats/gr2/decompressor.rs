//! BitKnit and Oodle decompression support for GR2 sectors
//! Based on reverse engineering of BG3's Granny3D implementation
//!
//! Granny uses **raw BitKnit streams** without Oodle container headers.
//! See `bitknit.rs` for Granny-specific decompression.

use crate::error::{Error, Result};
use crate::formats::gr2::bitknit;

/// BitKnit compression tag in Granny files (from BG3 RE: 0x80000039)
pub const BITKNIT_TAG: u32 = 0x80000039;

/// Oodle Kraken compression tag in Granny files
pub const OODLE_KRAKEN_TAG: u32 = 0x00000001;

/// Auto-detect compression type and decompress GR2 section
///
/// BG3 uses BitKnit compression (tag 0x80000039) for GR2 files.
/// This uses the Granny-specific BitKnit decompressor.
pub fn decompress_section(
    compressed: &[u8],
    decompressed_size: usize,
    compression_tag: u32,
) -> Result<Vec<u8>> {
    match compression_tag {
        // BitKnit compression (BG3's standard)
        BITKNIT_TAG => {
            tracing::debug!(
                "Decompressing BitKnit section: {} -> {} bytes",
                compressed.len(),
                decompressed_size
            );
            bitknit::decompress_raw_bitknit(compressed, decompressed_size)
        }

        // Uncompressed section
        0x00000000 => {
            tracing::debug!("Copying uncompressed section: {} bytes", compressed.len());

            if compressed.len() != decompressed_size {
                return Err(Error::DecompressionError(format!(
                    "Uncompressed section size mismatch: {} != {}",
                    compressed.len(),
                    decompressed_size
                )));
            }
            Ok(compressed.to_vec())
        }

        // Unknown compression (including Oodle Kraken, which we don't expect in BG3)
        _ => Err(Error::DecompressionError(format!(
            "Unknown compression tag: 0x{:08x} (expected BitKnit 0x{:08x})",
            compression_tag, BITKNIT_TAG
        ))),
    }
}