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
    /// The callback receives (current, total, filename) during file extraction.
    /// Files are decompressed and written one at a time for smooth progress updates.
    pub fn extract_with_progress<P: AsRef<Path>>(
        pak_path: P,
        output_dir: P,
        progress: ProgressCallback,
    ) -> Result<()> {
        let file = File::open(pak_path.as_ref())?;

        let mut reader = LspkReader::with_path(file, pak_path.as_ref());

        // Get file list without decompressing
        let entries = reader.list_files()?;
        let total_files = entries.len();

        std::fs::create_dir_all(&output_dir)?;

        let mut errors: Vec<(std::path::PathBuf, String)> = Vec::new();

        // Decompress and write each file one at a time
        for (index, entry) in entries.iter().enumerate() {
            // Skip .DS_Store files
            if entry.path.file_name() == Some(std::ffi::OsStr::new(".DS_Store")) {
                tracing::debug!("Skipping .DS_Store file: {:?}", entry.path);
                continue;
            }

            let file_name = entry.path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.path.to_string_lossy().to_string());

            progress(index + 1, total_files, &file_name);

            // Decompress the file
            let data = match reader.decompress_file(entry) {
                Ok(data) => data,
                Err(e) => {
                    tracing::warn!("Failed to decompress {}: {}", entry.path.display(), e);
                    errors.push((entry.path.clone(), e.to_string()));
                    continue;
                }
            };

            // For virtual texture files (.gts/.gtp), organize into subfolders
            let output_path = if is_virtual_texture_file(&file_name) {
                if let Some(subfolder) = get_virtual_texture_subfolder(&file_name) {
                    // Insert subfolder before the filename
                    if let Some(parent) = entry.path.parent() {
                        output_dir.as_ref().join(parent).join(&subfolder).join(&file_name)
                    } else {
                        output_dir.as_ref().join(&subfolder).join(&file_name)
                    }
                } else {
                    output_dir.as_ref().join(&entry.path)
                }
            } else {
                output_dir.as_ref().join(&entry.path)
            };

            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&output_path, &data)?;
        }

        // If there were errors, return a summary error
        if !errors.is_empty() {
            return Err(Error::ConversionError(format!(
                "Extracted {} files with {} errors. First error: {}",
                total_files - errors.len(),
                errors.len(),
                errors[0].1
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

/// Check if a filename is a virtual texture file (.gts or .gtp)
fn is_virtual_texture_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".gts") || lower.ends_with(".gtp")
}

/// Extract the subfolder name for a virtual texture file
/// e.g., "Albedo_Normal_Physical_0.gts" -> "Albedo_Normal_Physical_0"
/// e.g., "Albedo_Normal_Physical_0_abc123def.gtp" -> "Albedo_Normal_Physical_0"
fn get_virtual_texture_subfolder(filename: &str) -> Option<String> {
    let stem = filename.strip_suffix(".gts")
        .or_else(|| filename.strip_suffix(".gtp"))
        .or_else(|| filename.strip_suffix(".GTS"))
        .or_else(|| filename.strip_suffix(".GTP"))?;

    // For .gts files, the stem is already the subfolder name
    // e.g., "Albedo_Normal_Physical_0" from "Albedo_Normal_Physical_0.gts"
    if filename.to_lowercase().ends_with(".gts") {
        return Some(stem.to_string());
    }

    // For .gtp files, strip the hash suffix
    // e.g., "Albedo_Normal_Physical_0_abc123...def" -> "Albedo_Normal_Physical_0"
    if let Some(last_underscore) = stem.rfind('_') {
        let suffix = &stem[last_underscore + 1..];
        // Hash is 32 hex characters
        if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(stem[..last_underscore].to_string());
        }
    }

    // Fallback: use the full stem
    Some(stem.to_string())
}
