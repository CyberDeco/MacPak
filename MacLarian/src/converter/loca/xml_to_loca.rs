//! XML to LOCA conversion

use crate::error::{Error, Result};
use crate::formats::loca::{self, LocaResource, LocalizedText};

use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs;
use std::path::Path;

/// Convert XML file to .loca format
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_xml_to_loca<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    convert_xml_to_loca_with_progress(source, dest, &|_| {})
}

/// Convert XML file to .loca format with progress callback
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_xml_to_loca_with_progress<P: AsRef<Path>>(
    source: P,
    dest: P,
    progress: crate::converter::ConvertProgressCallback,
) -> Result<()> {
    use crate::converter::{ConvertPhase, ConvertProgress};

    tracing::info!(
        "Converting XML→LOCA: {:?} → {:?}",
        source.as_ref(),
        dest.as_ref()
    );

    progress(&ConvertProgress::with_file(
        ConvertPhase::ReadingSource,
        1,
        3,
        "Reading XML file...",
    ));
    let content = fs::read_to_string(&source)?;

    progress(&ConvertProgress::with_file(
        ConvertPhase::Parsing,
        2,
        3,
        "Parsing XML content...",
    ));
    let resource = from_xml(&content)?;

    progress(&ConvertProgress::with_file(
        ConvertPhase::WritingOutput,
        3,
        3,
        format!("Writing {} entries to LOCA...", resource.entries.len()),
    ));
    loca::write_loca(dest, &resource)?;

    progress(&ConvertProgress::new(ConvertPhase::Complete, 3, 3));
    tracing::info!("Conversion complete");
    Ok(())
}

/// Parse XML string to `LocaResource`
///
/// # Errors
/// Returns an error if XML parsing fails.
pub fn from_xml(content: &str) -> Result<LocaResource> {
    let mut reader = Reader::from_str(content);
    // Don't trim text - preserve trailing/leading whitespace in localization strings
    reader.trim_text(false);

    let mut entries = Vec::new();
    let mut buf = Vec::new();

    // Current entry being parsed
    let mut current_key: Option<String> = None;
    let mut current_version: u16 = 1;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"content" {
                    // Parse attributes
                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"contentuid" => {
                                current_key =
                                    Some(String::from_utf8_lossy(&attr.value).into_owned());
                            }
                            b"version" => {
                                current_version =
                                    String::from_utf8_lossy(&attr.value).parse().unwrap_or(1);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                // Text content inside <content> element
                if let Some(key) = current_key.take() {
                    let text = e.unescape().map_err(Error::XmlError)?;
                    entries.push(LocalizedText {
                        key,
                        version: current_version,
                        text: text.into_owned(),
                    });
                    current_version = 1;
                }
            }
            Ok(Event::Empty(e)) => {
                // Self-closing <content ... /> element (empty text)
                if e.name().as_ref() == b"content" {
                    let mut key = String::new();
                    let mut version: u16 = 1;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"contentuid" => {
                                key = String::from_utf8_lossy(&attr.value).into_owned();
                            }
                            b"version" => {
                                version = String::from_utf8_lossy(&attr.value).parse().unwrap_or(1);
                            }
                            _ => {}
                        }
                    }

                    entries.push(LocalizedText {
                        key,
                        version,
                        text: String::new(),
                    });
                }
            }
            Ok(Event::End(e)) => {
                // Handle </content> with no text content
                if e.name().as_ref() == b"content"
                    && let Some(key) = current_key.take()
                {
                    entries.push(LocalizedText {
                        key,
                        version: current_version,
                        text: String::new(),
                    });
                    current_version = 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Error::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(LocaResource { entries })
}
