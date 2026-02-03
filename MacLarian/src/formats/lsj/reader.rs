//! LSJ file reading
//!
//!

use super::document::LsjDocument;
use crate::error::Result;
use std::fs;
use std::path::Path;

/// Read an LSJ file from disk
///
/// # Errors
/// Returns an error if the file cannot be read or has invalid JSON.
pub fn read_lsj<P: AsRef<Path>>(path: P) -> Result<LsjDocument> {
    let content = fs::read_to_string(path)?;
    parse_lsj(&content)
}

/// Parse LSJ from JSON string
///
/// # Errors
/// Returns an error if the JSON is malformed.
pub fn parse_lsj(content: &str) -> Result<LsjDocument> {
    let doc: LsjDocument = serde_json::from_str(content)?;
    Ok(doc)
}
