//! `.loca` file writing
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::cast_possible_truncation)]

use super::{ENTRY_SIZE, KEY_SIZE, LOCA_SIGNATURE, LocaResource};
use crate::error::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Header size in bytes
const HEADER_SIZE: u32 = 12;

/// Write a .loca file to disk
///
/// # Errors
/// Returns an error if file writing fails.
pub fn write_loca<P: AsRef<Path>>(path: P, resource: &LocaResource) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let num_entries = resource.entries.len() as u32;
    let texts_offset = HEADER_SIZE + (ENTRY_SIZE as u32) * num_entries;

    // Write header
    writer.write_u32::<LittleEndian>(LOCA_SIGNATURE)?;
    writer.write_u32::<LittleEndian>(num_entries)?;
    writer.write_u32::<LittleEndian>(texts_offset)?;

    // Pre-calculate text lengths
    let text_lengths: Vec<u32> = resource
        .entries
        .iter()
        .map(|e| {
            if e.text.is_empty() {
                0
            } else {
                e.text.len() as u32 + 1 // +1 for null terminator
            }
        })
        .collect();

    // Write entry metadata
    for (entry, &length) in resource.entries.iter().zip(&text_lengths) {
        // Key: 64 bytes, null-padded
        let key_bytes = entry.key.as_bytes();
        let mut key_buf = [0u8; KEY_SIZE];
        let copy_len = key_bytes.len().min(KEY_SIZE);
        key_buf[..copy_len].copy_from_slice(&key_bytes[..copy_len]);
        writer.write_all(&key_buf)?;

        // Version: u16
        writer.write_u16::<LittleEndian>(entry.version)?;

        // Length: u32
        writer.write_u32::<LittleEndian>(length)?;
    }

    // Write text data
    for entry in &resource.entries {
        if !entry.text.is_empty() {
            writer.write_all(entry.text.as_bytes())?;
            writer.write_u8(0)?; // null terminator
        }
    }

    writer.flush()?;
    Ok(())
}
