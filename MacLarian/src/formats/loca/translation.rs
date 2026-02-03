//! Translation workflow support
//!
//! Export LOCA to TSV for translators, import translated TSV back.
//!
//! # TSV Format
//!
//! The TSV (Tab-Separated Values) format is designed for easy editing in spreadsheet
//! applications like Excel, Google Sheets, or LibreOffice Calc.
//!
//! Columns:
//! 1. Key (handle)
//! 2. Version
//! 3. Original text
//! 4. Translation (empty on export, filled by translator)
//!
//! # Example
//!
//! ```tsv
//! Key\tVersion\tOriginal\tTranslation
//! h12345abc\t1\tHello world\t
//! h67890def\t1\tGoodbye\t
//! ```

use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use super::{LocalizedText, LocaResource};
use crate::error::Result;

/// Export format for translation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Tab-separated values (recommended for spreadsheets)
    Tsv,
    /// Comma-separated values
    Csv,
}

impl ExportFormat {
    /// Get the file extension for this format
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Tsv => "tsv",
            Self::Csv => "csv",
        }
    }

    /// Get the delimiter character
    #[must_use]
    pub fn delimiter(&self) -> char {
        match self {
            Self::Tsv => '\t',
            Self::Csv => ',',
        }
    }
}

/// Result of importing translations
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Number of entries updated with translations
    pub translated: usize,
    /// Number of entries that were empty (no translation provided)
    pub skipped: usize,
    /// Number of keys not found in the original resource
    pub not_found: usize,
    /// Keys that were not found
    pub missing_keys: Vec<String>,
}

/// Export LOCA resource to a translation file
///
/// Creates a TSV/CSV file with columns for key, version, original text, and translation.
/// The translation column is empty, ready to be filled by translators.
///
/// # Arguments
/// * `resource` - The LOCA resource to export
/// * `path` - Output file path
/// * `format` - Export format (TSV or CSV)
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn export_for_translation<P: AsRef<Path>>(
    resource: &LocaResource,
    path: P,
    format: ExportFormat,
) -> Result<usize> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    let delimiter = format.delimiter();

    // Write header
    writeln!(writer, "Key{delimiter}Version{delimiter}Original{delimiter}Translation")?;

    // Write entries
    for entry in &resource.entries {
        let escaped_text = escape_for_delimited(&entry.text, delimiter);
        writeln!(
            writer,
            "{}{delimiter}{}{delimiter}{}{delimiter}",
            entry.key, entry.version, escaped_text
        )?;
    }

    writer.flush()?;
    Ok(resource.entries.len())
}

/// Export LOCA resource to a translation file with both source and target
///
/// This version includes a column for existing translations (useful for review).
///
/// # Arguments
/// * `source` - Original LOCA resource
/// * `target` - Translated LOCA resource (can be same as source for new translations)
/// * `path` - Output file path
/// * `format` - Export format
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn export_with_existing<P: AsRef<Path>>(
    source: &LocaResource,
    target: &LocaResource,
    path: P,
    format: ExportFormat,
) -> Result<usize> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    let delimiter = format.delimiter();

    // Write header
    writeln!(
        writer,
        "Key{delimiter}Version{delimiter}Source{delimiter}Translation"
    )?;

    // Write entries
    for entry in &source.entries {
        let escaped_source = escape_for_delimited(&entry.text, delimiter);
        let translation = target
            .get_entry(&entry.key)
            .map(|e| escape_for_delimited(&e.text, delimiter))
            .unwrap_or_default();

        writeln!(
            writer,
            "{}{delimiter}{}{delimiter}{}{delimiter}{}",
            entry.key, entry.version, escaped_source, translation
        )?;
    }

    writer.flush()?;
    Ok(source.entries.len())
}

/// Import translations from a TSV/CSV file
///
/// Updates the resource with translations from the file. Only entries with
/// non-empty translation columns are updated.
///
/// # Arguments
/// * `resource` - The LOCA resource to update
/// * `path` - Path to the translation file
/// * `format` - Import format
///
/// # Returns
/// Import result with statistics
///
/// # Errors
/// Returns an error if the file cannot be read or has an invalid format.
pub fn import_translations<P: AsRef<Path>>(
    resource: &mut LocaResource,
    path: P,
    format: ExportFormat,
) -> Result<ImportResult> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let delimiter = format.delimiter();

    let mut result = ImportResult {
        translated: 0,
        skipped: 0,
        not_found: 0,
        missing_keys: Vec::new(),
    };

    let mut lines = reader.lines();

    // Skip header
    if lines.next().is_none() {
        return Ok(result);
    }

    for line_result in lines {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(delimiter).collect();
        if parts.len() < 4 {
            continue; // Skip malformed lines
        }

        let key = parts[0].trim();
        let translation = unescape_from_delimited(parts[3].trim());

        if translation.is_empty() {
            result.skipped += 1;
            continue;
        }

        if let Some(entry) = resource.get_entry_mut(key) {
            entry.text = translation;
            result.translated += 1;
        } else {
            result.not_found += 1;
            result.missing_keys.push(key.to_string());
        }
    }

    Ok(result)
}

/// Create a new LOCA resource from translations
///
/// Creates a new resource with only the translated entries from the file.
///
/// # Arguments
/// * `path` - Path to the translation file
/// * `format` - Import format
///
/// # Returns
/// New LOCA resource with translated entries
///
/// # Errors
/// Returns an error if the file cannot be read.
pub fn create_from_translations<P: AsRef<Path>>(
    path: P,
    format: ExportFormat,
) -> Result<LocaResource> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let delimiter = format.delimiter();

    let mut resource = LocaResource::new();
    let mut lines = reader.lines();

    // Skip header
    if lines.next().is_none() {
        return Ok(resource);
    }

    for line_result in lines {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(delimiter).collect();
        if parts.len() < 4 {
            continue;
        }

        let key = parts[0].trim();
        let version: u16 = parts[1].trim().parse().unwrap_or(1);
        let translation = unescape_from_delimited(parts[3].trim());

        if !translation.is_empty() {
            resource.entries.push(LocalizedText {
                key: key.to_string(),
                version,
                text: translation,
            });
        }
    }

    Ok(resource)
}

/// Generate a translation template with side-by-side comparison
///
/// Useful for reviewing translations with the original text alongside.
///
/// # Arguments
/// * `source` - Original language resource
/// * `translated` - Translated resource
///
/// # Returns
/// Formatted string with side-by-side comparison
#[must_use]
pub fn generate_review_report(source: &LocaResource, translated: &LocaResource) -> String {
    let mut output = String::new();
    output.push_str("# Translation Review\n\n");

    let mut translated_count = 0;
    let mut missing_count = 0;

    for entry in &source.entries {
        let translation = translated.get_entry(&entry.key);

        if let Some(trans) = translation {
            if !trans.text.is_empty() && trans.text != entry.text {
                translated_count += 1;
            }
        } else {
            missing_count += 1;
        }
    }

    output.push_str(&format!(
        "Total: {} entries\n",
        source.entries.len()
    ));
    output.push_str(&format!("Translated: {translated_count}\n"));
    output.push_str(&format!("Missing: {missing_count}\n\n"));

    output.push_str("---\n\n");

    for entry in &source.entries {
        output.push_str(&format!("## {}\n\n", entry.key));
        output.push_str(&format!("**Original:** {}\n\n", entry.text));

        if let Some(trans) = translated.get_entry(&entry.key) {
            if trans.text.is_empty() {
                output.push_str("**Translation:** *(empty)*\n\n");
            } else {
                output.push_str(&format!("**Translation:** {}\n\n", trans.text));
            }
        } else {
            output.push_str("**Translation:** *(missing)*\n\n");
        }

        output.push_str("---\n\n");
    }

    output
}

// ============================================================================
// Helper functions
// ============================================================================

/// Escape text for TSV/CSV output
fn escape_for_delimited(text: &str, delimiter: char) -> String {
    // If text contains delimiter, newlines, or quotes, wrap in quotes and escape quotes
    if text.contains(delimiter) || text.contains('\n') || text.contains('\r') || text.contains('"')
    {
        let escaped = text.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        text.to_string()
    }
}

/// Unescape text from TSV/CSV input
fn unescape_from_delimited(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        // Remove surrounding quotes and unescape internal quotes
        trimmed[1..trimmed.len() - 1].replace("\"\"", "\"")
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_unescape() {
        let original = "Hello\tworld";
        let escaped = escape_for_delimited(original, '\t');
        assert!(escaped.starts_with('"'));

        let unescaped = unescape_from_delimited(&escaped);
        assert_eq!(original, unescaped);
    }

    #[test]
    fn test_escape_quotes() {
        let original = "Say \"hello\"";
        let escaped = escape_for_delimited(original, '\t');
        assert_eq!(escaped, "\"Say \"\"hello\"\"\"");

        let unescaped = unescape_from_delimited(&escaped);
        assert_eq!(original, unescaped);
    }
}
