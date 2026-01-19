//! Dialog format handling for Baldur's Gate 3
//!
//! This module provides types and parsers for BG3 dialog files (.lsf/.lsj),
//! as well as localization caching for dialog text display.
//!
//! # Overview
//!
//! BG3 dialogs are stored in LSF/LSJ format with a specific structure:
//! - Root nodes define entry points
//! - Each node has a constructor type (`TagAnswer`, `TagQuestion`, `ActiveRoll`, etc.)
//! - Nodes contain tagged text with localization handles
//! - Flags track dialog state and conditions
//!
//! # Usage
//!
//! ```no_run
//! use maclarian::dialog::{parse_dialog, LocalizationCache};
//! use maclarian::formats::lsj::read_lsj;
//!
//! // Load and parse a dialog file
//! let doc = read_lsj("path/to/dialog.lsj").unwrap();
//! let dialog = parse_dialog(&doc).unwrap();
//!
//! // Load localization
//! let mut cache = LocalizationCache::new();
//! cache.load_language_pak("path/to/game/Data").unwrap();
//!
//! // Get text for a dialog node
//! for node in dialog.nodes.values() {
//!     if let Some(text_entry) = dialog.get_node_text(node) {
//!         let text = cache.get_text(&text_entry.handle);
//!         println!("{}: {}", node.constructor.display_name(), text);
//!     }
//! }
//! ```

mod types;
mod parser;
mod localization;
mod flags;
mod speakers;
mod difficulty;
pub mod export;

pub use types::*;
pub use parser::{parse_dialog, DialogParseError};
pub use localization::{
    LocalizationCache,
    LocalizedEntry,
    LocalizationError,
    get_available_languages,
    load_localization_from_pak_parallel,
};
pub use flags::{
    FlagCache,
    FlagCacheError,
};
pub use speakers::{
    SpeakerCache,
    SpeakerCacheError,
};
pub use difficulty::{
    DifficultyClassCache,
    DifficultyClassInfo,
    DifficultyClassError,
};

/// Parse dialog from LSJ bytes
///
/// # Errors
/// Returns an error if the data is not valid UTF-8 or cannot be parsed as a dialog.
pub fn parse_dialog_bytes(data: &[u8]) -> Result<Dialog, DialogParseError> {
    let content = std::str::from_utf8(data)
        .map_err(|e| DialogParseError::InvalidFormat(format!("Invalid UTF-8: {e}")))?;
    let doc = crate::formats::lsj::parse_lsj(content)
        .map_err(|e| DialogParseError::InvalidFormat(e.to_string()))?;
    parse_dialog(&doc)
}

/// Parse dialog from a file path
///
/// # Errors
/// Returns an error if the file cannot be read or parsed as a dialog.
pub fn parse_dialog_file<P: AsRef<std::path::Path>>(path: P) -> Result<Dialog, DialogParseError> {
    let doc = crate::formats::lsj::read_lsj(path)
        .map_err(|e| DialogParseError::IoError(std::io::Error::other(
            e.to_string()
        )))?;
    parse_dialog(&doc)
}

/// Parse dialog from an LSF file (converts to LSJ internally)
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn parse_dialog_lsf<P: AsRef<std::path::Path>>(path: P) -> Result<Dialog, DialogParseError> {
    let data = std::fs::read(path.as_ref())
        .map_err(DialogParseError::IoError)?;
    parse_dialog_lsf_bytes(&data)
}

/// Parse dialog from LSF bytes (converts through LSF → LSX → LSJ → Dialog pipeline)
///
/// This is useful when reading dialog data from PAK files or other sources
/// where you have the raw bytes rather than a file path.
///
/// # Errors
/// Returns an error if the data cannot be parsed through the conversion pipeline.
pub fn parse_dialog_lsf_bytes(data: &[u8]) -> Result<Dialog, DialogParseError> {
    use crate::converter::{to_lsx, to_lsj};
    use crate::formats::lsf::parse_lsf_bytes;
    use crate::formats::lsx::parse_lsx;

    // Parse LSF binary
    let lsf_doc = parse_lsf_bytes(data)
        .map_err(|e| DialogParseError::InvalidFormat(format!("LSF parse error: {e}")))?;

    // Convert LSF to LSX XML string
    let lsx_xml = to_lsx(&lsf_doc)
        .map_err(|e| DialogParseError::InvalidFormat(format!("LSF→LSX error: {e}")))?;

    // Parse LSX XML string to document
    let lsx_doc = parse_lsx(&lsx_xml)
        .map_err(|e| DialogParseError::InvalidFormat(format!("LSX parse error: {e}")))?;

    // Convert to LSJ
    let lsj_doc = to_lsj(&lsx_doc)
        .map_err(|e| DialogParseError::InvalidFormat(format!("LSX→LSJ error: {e}")))?;

    parse_dialog(&lsj_doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_constructor_display() {
        assert_eq!(NodeConstructor::TagAnswer.display_name(), "Answer");
        assert_eq!(NodeConstructor::TagQuestion.display_name(), "Question");
        assert_eq!(NodeConstructor::ActiveRoll.display_name(), "Active Roll");
    }

    #[test]
    fn test_dialog_default() {
        let dialog = Dialog::new();
        assert!(dialog.nodes.is_empty());
        assert!(dialog.root_nodes.is_empty());
    }
}
