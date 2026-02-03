//! `.loca` file reading and parsing
//!
//!

#![allow(clippy::cast_possible_truncation)]

use super::{KEY_SIZE, LOCA_SIGNATURE, LocaResource, LocalizedText};
use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

/// Read a .loca file from disk
///
/// # Errors
///
/// Returns [`Error::Io`] if the file cannot be opened or read.
/// Returns [`Error::InvalidLocaMagic`] if the file does not have a valid LOCA header.
///
/// [`Error::Io`]: crate::Error::Io
/// [`Error::InvalidLocaMagic`]: crate::Error::InvalidLocaMagic
pub fn read_loca<P: AsRef<Path>>(path: P) -> Result<LocaResource> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    parse_loca_bytes(&buffer)
}

/// Parse .loca data from bytes
///
/// # Errors
///
/// Returns [`Error::InvalidLocaMagic`] if the data does not have a valid LOCA header.
/// Returns [`Error::Io`] if reading from the byte buffer fails (e.g., truncated data).
///
/// [`Error::InvalidLocaMagic`]: crate::Error::InvalidLocaMagic
/// [`Error::Io`]: crate::Error::Io
pub fn parse_loca_bytes(data: &[u8]) -> Result<LocaResource> {
    let mut cursor = Cursor::new(data);

    // Read header (12 bytes)
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;

    let signature = u32::from_le_bytes(magic);
    if signature != LOCA_SIGNATURE {
        return Err(Error::InvalidLocaMagic(magic));
    }

    let num_entries = cursor.read_u32::<LittleEndian>()? as usize;
    let texts_offset = u64::from(cursor.read_u32::<LittleEndian>()?);

    // Read entry metadata
    let mut entries = Vec::with_capacity(num_entries);
    let mut entry_metadata = Vec::with_capacity(num_entries);

    for _ in 0..num_entries {
        // Key: 64 bytes UTF-8 null-padded
        let mut key_bytes = [0u8; KEY_SIZE];
        cursor.read_exact(&mut key_bytes)?;

        // Find null terminator and extract key string
        let key_len = key_bytes.iter().position(|&b| b == 0).unwrap_or(KEY_SIZE);
        let key = String::from_utf8_lossy(&key_bytes[..key_len]).into_owned();

        // Version: u16
        let version = cursor.read_u16::<LittleEndian>()?;

        // Length: u32 (includes null terminator)
        let length = cursor.read_u32::<LittleEndian>()? as usize;

        entry_metadata.push((key, version, length));
    }

    // Seek to text data section
    cursor.seek(SeekFrom::Start(texts_offset))?;

    // Read text data for each entry
    for (key, version, length) in entry_metadata {
        if length > 0 {
            // Read text bytes (length - 1 for actual text, 1 for null terminator)
            let text_len = length.saturating_sub(1);
            let mut text_bytes = vec![0u8; text_len];
            cursor.read_exact(&mut text_bytes)?;

            // Skip the null terminator
            cursor.read_u8()?;

            let text = String::from_utf8_lossy(&text_bytes).into_owned();
            entries.push(LocalizedText { key, version, text });
        } else {
            // Empty text
            entries.push(LocalizedText {
                key,
                version,
                text: String::new(),
            });
        }
    }

    Ok(LocaResource { entries })
}
