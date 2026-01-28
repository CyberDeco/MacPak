//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! LOCA to XML conversion

use crate::error::Result;
use crate::formats::loca;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::borrow::Cow;
use std::path::Path;

/// Escape only the characters required in XML text content (not attributes).
/// In text content, only < and & need escaping. Apostrophes and quotes are fine.
fn escape_text_minimal(s: &str) -> Cow<'_, str> {
    if s.contains('&') || s.contains('<') {
        Cow::Owned(s.replace('&', "&amp;").replace('<', "&lt;"))
    } else {
        Cow::Borrowed(s)
    }
}

/// Convert .loca file to XML format
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_loca_to_xml<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    convert_loca_to_xml_with_progress(source, dest, &|_| {})
}

/// Convert .loca file to XML format with progress callback
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_loca_to_xml_with_progress<P: AsRef<Path>>(
    source: P,
    dest: P,
    progress: crate::converter::ConvertProgressCallback,
) -> Result<()> {
    use crate::converter::{ConvertPhase, ConvertProgress};

    tracing::info!(
        "Converting LOCA→XML: {:?} → {:?}",
        source.as_ref(),
        dest.as_ref()
    );

    progress(&ConvertProgress::with_file(
        ConvertPhase::ReadingSource,
        1,
        3,
        "Reading LOCA file...",
    ));
    let resource = loca::read_loca(&source)?;

    progress(&ConvertProgress::with_file(
        ConvertPhase::Converting,
        2,
        3,
        format!("Converting {} entries to XML...", resource.entries.len()),
    ));
    let xml = to_xml(&resource)?;

    progress(&ConvertProgress::with_file(
        ConvertPhase::WritingOutput,
        3,
        3,
        "Writing XML file...",
    ));
    std::fs::write(dest, xml)?;

    progress(&ConvertProgress::new(ConvertPhase::Complete, 3, 3));
    tracing::info!("Conversion complete");
    Ok(())
}

/// Convert `LocaResource` to XML string
///
/// # Errors
/// Returns an error if XML serialization fails.
pub fn to_xml(resource: &loca::LocaResource) -> Result<String> {
    let mut output = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output, b'\t', 1);

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

    // <contentList>
    writer.write_event(Event::Start(BytesStart::new("contentList")))?;

    for entry in &resource.entries {
        let mut content = BytesStart::new("content");
        content.push_attribute(("contentuid", entry.key.as_str()));
        content.push_attribute(("version", entry.version.to_string().as_str()));

        if entry.text.is_empty() {
            // Self-closing for empty text
            writer.write_event(Event::Empty(content))?;
        } else {
            writer.write_event(Event::Start(content.borrow()))?;
            // Use minimal escaping - only < and & need escaping in text content.
            // Apostrophes and quotes are shown literally for better readability.
            let escaped = escape_text_minimal(&entry.text);
            writer.write_event(Event::Text(BytesText::from_escaped(escaped)))?;
            writer.write_event(Event::End(BytesEnd::new("content")))?;
        }
    }

    writer.write_event(Event::End(BytesEnd::new("contentList")))?;

    let mut xml = String::from_utf8(output)?;
    // Add trailing newline
    xml.push('\n');
    Ok(xml)
}
