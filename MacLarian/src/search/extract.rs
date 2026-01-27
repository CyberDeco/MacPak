//! Text extraction for full-text indexing
//!
//! Extracts searchable text from various file formats for indexing.

use crate::formats::common::extract_value;
use crate::formats::lsf::parse_lsf_bytes;

use super::FileType;

/// Extract all searchable text from file bytes based on file type.
///
/// Returns a single string with all extractable text, suitable for indexing.
/// For LSF files, extracts names table and string attribute values.
/// For text files (LSX, XML, LSJ, JSON), returns the raw content.
#[must_use] 
pub fn extract_text(bytes: &[u8], file_type: FileType) -> String {
    match file_type {
        FileType::Lsf => extract_lsf_text(bytes),
        FileType::Lsx | FileType::Xml => extract_text_content(bytes),
        FileType::Lsj | FileType::Json => extract_text_content(bytes),
        _ => String::new(),
    }
}

/// Extract searchable text from LSF binary format.
///
/// Extracts:
/// - All names from the names table (node/attribute names)
/// - All string-type attribute values (strings, UUIDs, translated strings)
fn extract_lsf_text(bytes: &[u8]) -> String {
    let Ok(doc) = parse_lsf_bytes(bytes) else {
        return String::new();
    };

    let mut text_parts = Vec::new();

    // Extract all names from the names table
    for name_list in &doc.names {
        for name in name_list {
            if !name.is_empty() {
                text_parts.push(name.clone());
            }
        }
    }

    // Extract string-type attribute values
    // Type IDs: 20-23 (strings), 28-31 (translated strings, UUIDs)
    for attr in &doc.attributes {
        let type_id = attr.type_info & 0x3F;
        let value_length = (attr.type_info >> 6) as usize;

        if matches!(type_id, 20 | 21 | 22 | 23 | 28 | 29 | 30 | 31)
            && let Ok(value) = extract_value(&doc.values, attr.offset, value_length, type_id)
                && !value.is_empty() {
                    text_parts.push(value);
                }
    }

    // Join with newlines so each value is a separate "line" for match counting
    text_parts.join("\n")
}

/// Extract text content from UTF-8 encoded files.
fn extract_text_content(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

