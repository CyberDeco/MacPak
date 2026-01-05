//! .loca localization file format
//!
//! Binary format for Baldur's Gate 3 localization strings.
//! Can be converted to/from XML format.

mod reader;
mod writer;

pub use reader::{read_loca, parse_loca_bytes};
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
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }
}
