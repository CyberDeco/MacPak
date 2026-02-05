//! GR2 (Granny2) file format support
//!
//! This module provides parsing and decompression for Granny2 GR2 files
//! used by Baldur's Gate 3 and Divinity: Original Sin 2.
//!
//!

#![allow(clippy::cast_possible_truncation)]

mod decompress;
mod format;
mod inspect;

// Internal format types (used by decompress_gr2 and other internal modules)
use decompress::decompress_bitknit;
use format::{Compression, Gr2File};

// Crate-internal exports for other modules that need these
pub(crate) use decompress::decompress_bitknit as bitknit_decompress;
pub(crate) use format::PointerSize;
pub(crate) use format::magic;

// Public inspection API
pub use inspect::{
    Gr2BoneInfo, Gr2Info, Gr2MeshInfo, Gr2ModelInfo, Gr2SkeletonInfo, SectionInfo,
    extract_gr2_info, inspect_gr2,
};

use crate::error::{Error, Result};

/// Decompress a GR2 file and return the decompressed section data.
///
/// This is a convenience function that parses the GR2 file and decompresses
/// all sections, returning the concatenated decompressed data.
///
/// # Errors
/// Returns an error if the file cannot be parsed or decompression fails.
pub fn decompress_gr2(data: &[u8]) -> Result<Vec<u8>> {
    let gr2 = Gr2File::from_bytes(data)?;

    // Calculate total decompressed size
    let total_size: usize = gr2
        .sections
        .iter()
        .map(|s| s.uncompressed_size as usize)
        .sum();

    let mut output = Vec::with_capacity(total_size);

    for (i, section) in gr2.sections.iter().enumerate() {
        if section.is_empty() {
            // Add zeros for empty sections
            output.extend(std::iter::repeat_n(0u8, section.uncompressed_size as usize));
            continue;
        }

        let compressed = gr2.section_compressed_data(i)?;

        let decompressed = match section.compression {
            Compression::None => compressed.to_vec(),
            Compression::BitKnit => {
                decompress_bitknit(compressed, section.uncompressed_size as usize)?
            }
            Compression::Oodle0 | Compression::Oodle1 => {
                let compression = section.compression;
                return Err(Error::DecompressionError(format!(
                    "Oodle compression not supported (format {compression:?})"
                )));
            }
        };

        output.extend_from_slice(&decompressed);
    }

    Ok(output)
}
