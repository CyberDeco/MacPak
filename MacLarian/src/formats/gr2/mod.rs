//! GR2 (Granny 3D) file format support
//!
//! This module provides parsing and decompression for GR2 mesh files
//! used in Baldur's Gate 3.
//!
//! ## Features
//!
//! - **BitKnit Decompression**: Clean-room implementation of RAD Game Tools' BitKnit algorithm
//! - **Complete GR2 Parser**: Supports all 5 format variants (32/64-bit, little/big endian)
//! - **Multi-section Support**: Preserves distance cache across compressed sections
//! - **Zero Dependencies**: Pure Rust implementation with no proprietary libraries
//!
//! ## Usage
//!
//! ```no_run
//! use MacLarian::formats::gr2::parser::GR2File;
//!
//! // Parse a GR2 file
//! let mut gr2 = GR2File::new("mesh.gr2")?;
//!
//! // Print file information
//! gr2.print_info();
//!
//! // Extract all sections
//! gr2.extract_all_sections("output/")?;
//!
//! // Or get specific section
//! if let Some(data) = gr2.get_decompressed_section(1) {
//!     println!("Section 1: {} bytes", data.len());
//! }
//! # Ok::<(), String>(())
//! ```
//!
//! ## Architecture
//!
//! - `decompressor.rs`: BitKnit decompression engine and state management
//! - `range_decoder.rs`: Range decoder with LZ77 match copying
//! - `parser.rs`: GR2 file parser and section extractor

pub mod decompressor;
pub mod parser;
pub mod range_decoder;

// Re-export main types for convenience
pub use decompressor::{decompress, BitKnitContext, GR2Decompressor};
pub use parser::{GR2File, GR2Section};
