//! `.loca` localization file format
//!
//! Binary format for Baldur's Gate 3 localization strings.
//! Can be converted to/from XML format.
//!
//! # Editing LOCA files
//!
//! ```no_run
//! use maclarian::formats::loca::{LocaResource, read_loca, write_loca};
//!
//! // Load a LOCA file
//! let mut resource = read_loca("game.loca")?;
//!
//! // Add or update an entry
//! resource.add_entry("h12345", "New text");
//!
//! // Bulk replace text
//! let result = resource.replace_all("old text", "new text", false);
//! println!("Modified {} entries", result.entries_modified);
//!
//! // Save changes
//! write_loca("game_modified.loca", &resource)?;
//! # Ok::<(), maclarian::Error>(())
//! ```
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

mod editor;
mod reader;
pub mod translation;
mod writer;

pub use editor::{EditResult, ReplaceResult};
pub use reader::{parse_loca_bytes, read_loca};
pub use translation::{
    ExportFormat, ImportResult, create_from_translations, export_for_translation,
    export_with_existing, generate_review_report, import_translations,
};
pub use writer::write_loca;

/// "LOCA" magic signature (little-endian)
pub const LOCA_SIGNATURE: u32 = 0x41434F4C;

/// Size of each entry in the entry table (64 + 2 + 4 = 70 bytes)
pub const ENTRY_SIZE: usize = 70;

/// Size of the key field in each entry
pub const KEY_SIZE: usize = 64;

/// A single localized text entry
#[derive(Debug, Clone)]
pub struct LocalizedText {
    /// Unique identifier key (e.g., "h1234567890abcdef...")
    pub key: String,
    /// Version number
    pub version: u16,
    /// The localized text content
    pub text: String,
}

/// A collection of localized text entries
#[derive(Debug, Clone, Default)]
pub struct LocaResource {
    pub entries: Vec<LocalizedText>,
}

impl LocaResource {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}
