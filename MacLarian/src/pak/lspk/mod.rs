//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0
//!
//! LSPK PAK file format reader/writer
//!
//! This is a replacement for the larian-formats crate's PAK handling,
//! with better error handling, progress callbacks, and error recovery.

mod reader;
mod types;
mod writer;

pub use reader::LspkReader;
pub use types::*;
pub use writer::LspkWriter;

/// LSPK magic bytes
pub const MAGIC: [u8; 4] = [b'L', b'S', b'P', b'K'];

/// Minimum supported PAK version
pub const MIN_VERSION: u32 = 15;

/// Maximum supported PAK version
pub const MAX_VERSION: u32 = 18;

/// Length of file path in table entry
pub const PATH_LENGTH: usize = 256;

/// Size of a decompressed table entry
pub const TABLE_ENTRY_SIZE: usize = 272;
