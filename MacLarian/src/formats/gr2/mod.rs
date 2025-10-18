//! GR2 (Granny 3D) file format support for BG3
//!
//! This module provides custom BitKnit-enabled GR2 parsing for BG3.
//! All compressed sections are automatically decompressed during parsing.

pub mod decompressor;
pub mod parser;
pub mod bitknit;

// Public API exports
pub use parser::{ParsedGr2File, GrannyHeader, GrannySection};
pub use decompressor::{decompress_section, BITKNIT_TAG};
pub use bitknit::{decompress_raw_bitknit, BitknitState};