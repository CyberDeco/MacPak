//! LSPK PAK file format reader/writer
//!
//! This is a replacement for the larian-formats crate's PAK handling,
//! with better error handling, progress callbacks, and error recovery.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0

mod reader;
mod types;
mod writer;

// Reader/Writer exports
pub use reader::LspkReader;
pub(crate) use writer::LspkWriter;

// Public types that users need
pub use types::{CompressionMethod, FileTableEntry, PakContents, PakFile, PakPhase, PakProgress};

// Internal constants
pub(crate) const MAGIC: [u8; 4] = [b'L', b'S', b'P', b'K'];
pub(crate) const MIN_VERSION: u32 = 15;
pub(crate) const MAX_VERSION: u32 = 18;
pub(crate) const PATH_LENGTH: usize = 256;
pub(crate) const TABLE_ENTRY_SIZE: usize = 272;

// Internal header types (used by reader)
pub(crate) use types::{LspkFooter, LspkHeader};
