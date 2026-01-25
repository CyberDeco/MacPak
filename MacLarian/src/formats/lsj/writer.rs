//! LSJ file writing
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

use super::document::LsjDocument;
use crate::error::Result;
use std::fs;
use std::path::Path;

/// Write an LSJ document to disk
///
/// # Errors
/// Returns an error if serialization or file writing fails.
pub fn write_lsj<P: AsRef<Path>>(doc: &LsjDocument, path: P) -> Result<()> {
    let json = serialize_lsj(doc)?;
    fs::write(path, json)?;
    Ok(())
}

/// Serialize LSJ document to JSON string with tab indentation (matching `LSLib`)
///
/// # Errors
/// Returns an error if JSON serialization fails.
pub fn serialize_lsj(doc: &LsjDocument) -> Result<String> {
    let json = serde_json::to_string_pretty(doc)?;

    // Convert space indentation to tabs (matching `LSLib` output)
    let json = convert_spaces_to_tabs(&json);

    // Convert to Windows line endings (matching `LSLib` output on Windows)
    let json = json.replace('\n', "\r\n");

    Ok(json)
}

/// Convert 2-space indentation to tab indentation (matching `LSLib`'s `JsonTextWriter`)
fn convert_spaces_to_tabs(json: &str) -> String {
    json.lines()
        .map(|line| {
            // Count leading spaces
            let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
            // Convert pairs of spaces to tabs
            let tabs = "\t".repeat(leading_spaces / 2);
            // Return line with tabs instead of spaces
            format!("{tabs}{}", line.trim_start())
        })
        .collect::<Vec<_>>()
        .join("\n")
}