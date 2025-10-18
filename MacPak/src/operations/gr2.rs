//! GR2 mesh extraction and processing operations
//!
//! High-level API for working with Granny3D GR2 files, including:
//! - Automatic BitKnit decompression
//! - Section extraction to JSON
//! - Mesh metadata inspection
//! - Future: glTF/OBJ export

use crate::error::Result;
use MacLarian::formats::gr2::parser::ParsedGr2File;
use std::path::Path;
use std::fs;

/// GR2 file summary information
#[derive(Debug, Clone, serde::Serialize)]
pub struct Gr2Summary {
    pub filename: String,
    pub file_size: usize,
    pub header_size: u32,
    pub compression_tag: u32,
    pub compression_type: String,
    pub section_count: u32,
    pub total_compressed: usize,
    pub total_decompressed: usize,
    pub overall_ratio: f64,
    pub space_saved_percent: f64,
}

/// Individual section information
#[derive(Debug, Clone, serde::Serialize)]
pub struct SectionInfo {
    pub index: usize,
    pub compressed_size: u32,
    pub decompressed_size: u32,
    pub is_compressed: bool,
    pub compression_ratio: f64,
}

/// Inspect a GR2 file and return summary information
///
/// This provides high-level metadata without extracting the full data.
///
/// # Example
///
/// ```no_run
/// use MacPak::operations::gr2::inspect_gr2;
///
/// let summary = inspect_gr2("mesh.GR2")?;
/// println!("Compression ratio: {:.2}:1", summary.overall_ratio);
/// # Ok::<(), MacPak::error::Error>(())
/// ```
pub fn inspect_gr2(path: impl AsRef<Path>) -> Result<Gr2Summary> {
    let path = path.as_ref();
    let data = fs::read(path)?;
    let file_size = data.len();

    let parsed = ParsedGr2File::from_bytes(&data)?;

    let compression_type = match parsed.header.compression_tag {
        0x80000039 => "BitKnit (Granny custom)".to_string(),
        0x00000001 => "Oodle Kraken".to_string(),
        0x00000000 => "None".to_string(),
        tag => format!("Unknown (0x{:08x})", tag),
    };

    let mut total_compressed = 0;
    let mut total_decompressed = 0;

    for (section, data) in parsed.sections.iter().zip(parsed.decompressed_data.iter()) {
        if section.is_compressed() {
            total_compressed += section.compressed_size as usize;
            total_decompressed += data.len();
        } else if !data.is_empty() {
            total_decompressed += data.len();
        }
    }

    let overall_ratio = if total_compressed > 0 {
        total_decompressed as f64 / total_compressed as f64
    } else {
        0.0
    };

    let space_saved_percent = if total_compressed > 0 {
        100.0 * (1.0 - (total_compressed as f64 / total_decompressed as f64))
    } else {
        0.0
    };

    Ok(Gr2Summary {
        filename: path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        file_size,
        header_size: parsed.header.header_size,
        compression_tag: parsed.header.compression_tag,
        compression_type,
        section_count: parsed.header.section_count,
        total_compressed,
        total_decompressed,
        overall_ratio,
        space_saved_percent,
    })
}

/// Get detailed information about all sections
///
/// # Example
///
/// ```no_run
/// use MacPak::operations::gr2::get_section_info;
///
/// let sections = get_section_info("mesh.GR2")?;
/// for section in sections {
///     println!("Section {}: {:.1}:1 ratio", section.index, section.compression_ratio);
/// }
/// # Ok::<(), MacPak::error::Error>(())
/// ```
pub fn get_section_info(path: impl AsRef<Path>) -> Result<Vec<SectionInfo>> {
    let data = fs::read(path)?;
    let parsed = ParsedGr2File::from_bytes(&data)?;

    let sections = parsed.sections.iter()
        .zip(parsed.decompressed_data.iter())
        .enumerate()
        .map(|(i, (section, decompressed_data))| {
            let compression_ratio = if section.is_compressed() && section.compressed_size > 0 {
                decompressed_data.len() as f64 / section.compressed_size as f64
            } else {
                0.0
            };

            SectionInfo {
                index: i,
                compressed_size: section.compressed_size,
                decompressed_size: section.decompressed_size,
                is_compressed: section.is_compressed(),
                compression_ratio,
            }
        })
        .collect();

    Ok(sections)
}

/// Extract GR2 file to JSON format
///
/// This decompresses all sections and exports them to a structured JSON file.
///
/// # Example
///
/// ```no_run
/// use MacPak::operations::gr2::extract_to_json;
///
/// extract_to_json("mesh.GR2", "mesh.json")?;
/// # Ok::<(), MacPak::error::Error>(())
/// ```
pub fn extract_to_json(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<()> {
    let data = fs::read(source)?;
    let parsed = ParsedGr2File::from_bytes(&data)?;

    // Create export structure
    #[derive(serde::Serialize)]
    struct Gr2Export {
        header: HeaderInfo,
        sections: Vec<SectionExport>,
    }

    #[derive(serde::Serialize)]
    struct HeaderInfo {
        header_size: u32,
        total_size: u32,
        crc32: String,
        compression_tag: String,
        section_count: u32,
    }

    #[derive(serde::Serialize)]
    struct SectionExport {
        index: usize,
        compression: u32,
        compressed_size: u32,
        decompressed_size: u32,
        #[serde(skip_serializing_if = "String::is_empty")]
        data_hex: String,
    }

    let export = Gr2Export {
        header: HeaderInfo {
            header_size: parsed.header.header_size,
            total_size: parsed.header.total_size,
            crc32: format!("0x{:08x}", parsed.header.crc32),
            compression_tag: format!("0x{:08x}", parsed.header.compression_tag),
            section_count: parsed.header.section_count,
        },
        sections: parsed.sections.iter()
            .zip(parsed.decompressed_data.iter())
            .enumerate()
            .map(|(i, (section, data))| {
                // Only include first 256 bytes as hex preview
                let hex_preview = if !data.is_empty() {
                    data.iter()
                        .take(256)
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    String::new()
                };

                SectionExport {
                    index: i,
                    compression: section.compression,
                    compressed_size: section.compressed_size,
                    decompressed_size: section.decompressed_size,
                    data_hex: hex_preview,
                }
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&export)?;
    fs::write(destination, json)?;

    Ok(())
}

/// Extract all decompressed section data
///
/// Returns the raw decompressed bytes for each section.
/// Useful for further processing or custom export formats.
///
/// # Example
///
/// ```no_run
/// use MacPak::operations::gr2::extract_sections;
///
/// let sections = extract_sections("mesh.GR2")?;
/// for (i, data) in sections.iter().enumerate() {
///     println!("Section {}: {} bytes", i, data.len());
/// }
/// # Ok::<(), MacPak::error::Error>(())
/// ```
pub fn extract_sections(path: impl AsRef<Path>) -> Result<Vec<Vec<u8>>> {
    let data = fs::read(path)?;
    let parsed = ParsedGr2File::from_bytes(&data)?;
    Ok(parsed.decompressed_data)
}

/// Batch process multiple GR2 files
///
/// Processes all GR2 files in a directory and extracts them to JSON.
///
/// # Example
///
/// ```no_run
/// use MacPak::operations::gr2::batch_extract;
///
/// let results = batch_extract("input/", "output/")?;
/// println!("Processed {} files", results.len());
/// # Ok::<(), MacPak::error::Error>(())
/// ```
pub fn batch_extract(
    source_dir: impl AsRef<Path>,
    dest_dir: impl AsRef<Path>,
) -> Result<Vec<String>> {
    let source_dir = source_dir.as_ref();
    let dest_dir = dest_dir.as_ref();

    // Create destination directory if needed
    fs::create_dir_all(dest_dir)?;

    let mut processed = Vec::new();

    // Find all .GR2 files
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("gr2") {
                let filename = path.file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| crate::error::Error::Io(
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename")
                    ))?;

                let dest_path = dest_dir.join(format!("{}.json", filename));

                match extract_to_json(&path, &dest_path) {
                    Ok(_) => {
                        processed.push(filename.to_string());
                        tracing::info!("Extracted: {}", filename);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to extract {}: {}", filename, e);
                    }
                }
            }
        }
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_gr2() {
        // Skip if test file not available
        if !Path::new("ELF_M_NKD_Hair_Astarion_Base.GR2").exists() {
            return;
        }

        let summary = inspect_gr2("ELF_M_NKD_Hair_Astarion_Base.GR2").unwrap();

        assert_eq!(summary.compression_tag, 0x80000039);
        assert_eq!(summary.section_count, 5);
        assert!(summary.overall_ratio > 1.0);
        assert!(summary.space_saved_percent > 0.0);
    }

    #[test]
    fn test_get_section_info() {
        if !Path::new("ELF_M_NKD_Hair_Astarion_Base.GR2").exists() {
            return;
        }

        let sections = get_section_info("ELF_M_NKD_Hair_Astarion_Base.GR2").unwrap();

        assert_eq!(sections.len(), 5);
        // Section 1 should be compressed
        assert!(sections[1].is_compressed);
        assert!(sections[1].compression_ratio > 1.0);
    }
}
