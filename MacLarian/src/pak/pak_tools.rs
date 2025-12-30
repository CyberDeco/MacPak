//! PAK archive operations

use crate::error::{Error, Result};
use super::lspk::{LspkReader, LspkWriter, PakPhase, PakProgress};
use std::fs::File;
use std::path::Path;

/// Progress callback for PAK operations
/// Arguments: (current, total, description)
pub type ProgressCallback<'a> = &'a dyn Fn(usize, usize, &str);

pub struct PakOperations;

impl PakOperations {
    /// Extract a PAK file to a directory
    pub fn extract<P: AsRef<Path>>(pak_path: P, output_dir: P) -> Result<()> {
        Self::extract_with_progress(pak_path, output_dir, &|_, _, _| {})
    }

    /// Extract a PAK file to a directory with progress callback
    ///
    /// The callback receives (current, total, phase_description) where:
    /// - During decompression: "Decompressing [filename]" with file count progress
    /// - During writing: "Writing [filename]" with file count progress
    pub fn extract_with_progress<P: AsRef<Path>>(
        pak_path: P,
        output_dir: P,
        progress: ProgressCallback,
    ) -> Result<()> {
        let file = File::open(pak_path.as_ref())?;

        let mut reader = LspkReader::with_path(file, pak_path.as_ref());

        // Read PAK with progress
        let contents = reader.read_all(Some(&|p: &PakProgress| {
            let description = match p.phase {
                PakPhase::ReadingHeader | PakPhase::ReadingTable => {
                    "Reading PAK...".to_string()
                }
                PakPhase::DecompressingFiles => {
                    if let Some(ref name) = p.current_file {
                        format!("Decompressing {}", name)
                    } else {
                        "Decompressing...".to_string()
                    }
                }
                PakPhase::WritingFiles => {
                    if let Some(ref name) = p.current_file {
                        format!("Writing {}", name)
                    } else {
                        "Writing...".to_string()
                    }
                }
                PakPhase::Complete => "Complete".to_string(),
            };
            progress(p.current, p.total, &description);
        }))?;

        // Report any decompression errors but continue
        for (path, error) in &contents.errors {
            tracing::warn!("Failed to decompress {}: {}", path.display(), error);
        }

        std::fs::create_dir_all(&output_dir)?;

        let total_files = contents.files.len();

        for (index, file) in contents.files.iter().enumerate() {
            // Skip .DS_Store files
            if file.path.file_name() == Some(std::ffi::OsStr::new(".DS_Store")) {
                tracing::debug!("Skipping .DS_Store file: {:?}", file.path);
                continue;
            }

            let file_name = file.path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file.path.to_string_lossy().to_string());
            progress(index + 1, total_files, &format!("Writing {}", file_name));

            let output_path = output_dir.as_ref().join(&file.path);

            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&output_path, &file.data)?;
        }

        // If there were errors, return a summary error
        if !contents.errors.is_empty() {
            return Err(Error::ConversionError(format!(
                "Extracted {} files with {} errors. First error: {}",
                contents.files.len(),
                contents.errors.len(),
                contents.errors[0].1
            )));
        }

        Ok(())
    }

    /// Create a PAK file from a directory
    pub fn create<P: AsRef<Path>>(source_dir: P, output_pak: P) -> Result<()> {
        Self::create_with_progress(source_dir, output_pak, &|_, _, _| {})
    }

    /// Create a PAK file from a directory with progress callback
    pub fn create_with_progress<P: AsRef<Path>>(
        source_dir: P,
        output_pak: P,
        progress: ProgressCallback,
    ) -> Result<()> {
        let writer = LspkWriter::new(source_dir.as_ref())?;
        writer.write_with_progress(output_pak.as_ref(), progress)?;
        Ok(())
    }

    /// List contents of a PAK file
    pub fn list<P: AsRef<Path>>(pak_path: P) -> Result<Vec<String>> {
        Self::list_with_progress(pak_path, &|_, _, _| {})
    }

    /// List contents of a PAK file with progress callback
    pub fn list_with_progress<P: AsRef<Path>>(
        pak_path: P,
        progress: ProgressCallback,
    ) -> Result<Vec<String>> {
        let file = File::open(pak_path.as_ref())?;

        let mut reader = LspkReader::with_path(file, pak_path.as_ref());

        progress(0, 1, "Reading PAK...");

        let entries = reader.list_files()?;

        progress(1, 1, "Complete");

        Ok(entries.iter()
            .map(|e| e.path.to_string_lossy().to_string())
            .collect())
    }

    /// Extract meta.lsx from a PAK
    pub fn extract_meta<P: AsRef<Path>>(pak_path: P) -> Result<String> {
        let file = File::open(pak_path.as_ref())?;

        let mut reader = LspkReader::with_path(file, pak_path.as_ref());
        let contents = reader.read_all(None)?;

        // Find meta.lsx
        let meta_file = contents.files.iter()
            .find(|f| {
                let path = &f.path;
                let mut components = path.components();

                // Look for Mods/*/meta.lsx pattern
                if let Some(first) = components.next() {
                    if first.as_os_str() == "Mods" {
                        if components.next().is_some() {
                            if let Some(third) = components.next() {
                                return third.as_os_str() == "meta.lsx";
                            }
                        }
                    }
                }
                false
            })
            .ok_or_else(|| Error::FileNotFoundInPak("meta.lsx".to_string()))?;

        String::from_utf8(meta_file.data.clone())
            .map_err(|e| Error::ConversionError(format!("Invalid UTF-8 in meta.lsx: {}", e)))
    }
}
