//! Core PAK archive operations

use super::super::lspk::{
    CompressionMethod, FileTableEntry, LspkReader, LspkWriter, PakPhase, PakProgress,
};
use super::ProgressCallback;
use super::decompression::decompress_data;
use super::helpers::{get_part_path, get_virtual_texture_subfolder, is_virtual_texture_file};
use crate::error::{Error, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Compressed file data ready for parallel decompression
struct CompressedFile {
    entry: FileTableEntry,
    compressed_data: Vec<u8>,
}

/// High-level PAK archive operations.
pub struct PakOperations;

impl PakOperations {
    /// Extract a PAK file to a directory
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened or output directory cannot be created.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    /// Returns [`Error::Lz4DecompressionFailed`] or [`Error::ZlibDecompressionFailed`] if file decompression fails.
    /// Returns [`Error::PakExtractionPartialFailure`] if extraction completes with partial failures.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    /// [`Error::Lz4DecompressionFailed`]: crate::Error::Lz4DecompressionFailed
    /// [`Error::ZlibDecompressionFailed`]: crate::Error::ZlibDecompressionFailed
    /// [`Error::PakExtractionPartialFailure`]: crate::Error::PakExtractionPartialFailure
    pub fn extract<P: AsRef<Path>>(pak_path: P, output_dir: P) -> Result<()> {
        Self::extract_with_progress(pak_path, output_dir, &|_| {})
    }

    /// Extract a PAK file to a directory with progress callback
    ///
    /// The callback receives [`PakProgress`] with phase and file information.
    /// Uses parallel decompression for improved performance on multi-core systems.
    /// Supports multi-part archives (e.g., `Textures.pak` with `Textures_1.pak`, `Textures_2.pak`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened or output directory cannot be created.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    /// Returns [`Error::Lz4DecompressionFailed`] or [`Error::ZlibDecompressionFailed`] if file decompression fails.
    /// Returns [`Error::PakExtractionPartialFailure`] if extraction completes with partial failures.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    /// [`Error::Lz4DecompressionFailed`]: crate::Error::Lz4DecompressionFailed
    /// [`Error::ZlibDecompressionFailed`]: crate::Error::ZlibDecompressionFailed
    /// [`Error::PakExtractionPartialFailure`]: crate::Error::PakExtractionPartialFailure
    pub fn extract_with_progress<P: AsRef<Path>>(
        pak_path: P,
        output_dir: P,
        progress: ProgressCallback,
    ) -> Result<()> {
        let pak_path = pak_path.as_ref();
        let output_dir = output_dir.as_ref();

        let mut reader = LspkReader::with_path(File::open(pak_path)?, pak_path);

        progress(&PakProgress {
            phase: PakPhase::ReadingTable,
            current: 1,
            total: 1,
            current_file: None,
        });

        // Get file list without decompressing
        let entries = reader.list_files()?;

        std::fs::create_dir_all(output_dir)?;

        // Filter entries (skip .DS_Store files)
        let filtered_entries: Vec<_> = entries
            .into_iter()
            .filter(|entry| entry.path.file_name() != Some(std::ffi::OsStr::new(".DS_Store")))
            .collect();

        let total_files = filtered_entries.len();
        let processed = AtomicUsize::new(0);
        let error_count = AtomicUsize::new(0);

        // Single-pass parallel extraction: each thread opens its own file handle
        let errors: Vec<(PathBuf, String)> = filtered_entries
            .par_iter()
            .filter_map(|entry| {
                let file_name = entry.path.file_name().map_or_else(
                    || entry.path.to_string_lossy().to_string(),
                    |n| n.to_string_lossy().to_string(),
                );

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                progress(&PakProgress {
                    phase: PakPhase::DecompressingFiles,
                    current,
                    total: total_files,
                    current_file: Some(file_name.clone()),
                });

                // Get the correct part file path for this entry
                let part_path = if let Some(p) = get_part_path(pak_path, entry.archive_part) {
                    p
                } else {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    return Some((
                        entry.path.clone(),
                        format!(
                            "Cannot determine path for archive part {}",
                            entry.archive_part
                        ),
                    ));
                };

                // Open file handle for this thread, seek, and read
                let mut file = match File::open(&part_path) {
                    Ok(f) => f,
                    Err(e) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        return Some((
                            entry.path.clone(),
                            format!("Failed to open {}: {e}", part_path.display()),
                        ));
                    }
                };

                if let Err(e) = file.seek(SeekFrom::Start(entry.offset)) {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    return Some((entry.path.clone(), format!("Failed to seek: {e}")));
                }

                let mut compressed_data = vec![0u8; entry.size_compressed as usize];
                if let Err(e) = file.read_exact(&mut compressed_data) {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    return Some((entry.path.clone(), format!("Failed to read: {e}")));
                }

                // Decompress
                let data = match decompress_data(
                    &compressed_data,
                    entry.compression,
                    entry.size_decompressed,
                ) {
                    Ok(data) => data,
                    Err(e) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        tracing::warn!("Failed to decompress {}: {}", entry.path.display(), e);
                        return Some((entry.path.clone(), e.to_string()));
                    }
                };

                // Calculate output path (handle virtual texture subfolders)
                let output_path = if is_virtual_texture_file(&file_name) {
                    if let Some(subfolder) = get_virtual_texture_subfolder(&file_name) {
                        if let Some(parent) = entry.path.parent() {
                            output_dir.join(parent).join(&subfolder).join(&file_name)
                        } else {
                            output_dir.join(&subfolder).join(&file_name)
                        }
                    } else {
                        output_dir.join(&entry.path)
                    }
                } else {
                    output_dir.join(&entry.path)
                };

                // Create parent directories (idempotent)
                if let Some(parent) = output_path.parent()
                    && let Err(e) = std::fs::create_dir_all(parent)
                {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    return Some((entry.path.clone(), format!("Failed to create dir: {e}")));
                }

                // Write file
                if let Err(e) = std::fs::write(&output_path, &data) {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    return Some((entry.path.clone(), format!("Failed to write: {e}")));
                }

                None
            })
            .collect();

        // If there were errors, return a summary error
        if !errors.is_empty() {
            return Err(Error::PakExtractionPartialFailure {
                total: total_files,
                failed: errors.len(),
                first_error: errors[0].1.clone(),
            });
        }

        Ok(())
    }

    /// Create a PAK file from a directory
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the source directory cannot be read or output file cannot be written.
    /// Returns [`Error::WalkDirError`] if directory traversal fails.
    /// Returns [`Error::CompressionError`] if file compression fails.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::WalkDirError`]: crate::Error::WalkDirError
    /// [`Error::CompressionError`]: crate::Error::CompressionError
    pub fn create<P: AsRef<Path>>(source_dir: P, output_pak: P) -> Result<()> {
        Self::create_with_progress(source_dir, output_pak, &|_| {})
    }

    /// Create a PAK file from a directory with progress callback
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the source directory cannot be read or output file cannot be written.
    /// Returns [`Error::WalkDirError`] if directory traversal fails.
    /// Returns [`Error::CompressionError`] if file compression fails.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::WalkDirError`]: crate::Error::WalkDirError
    /// [`Error::CompressionError`]: crate::Error::CompressionError
    pub fn create_with_progress<P: AsRef<Path>>(
        source_dir: P,
        output_pak: P,
        progress: ProgressCallback,
    ) -> Result<()> {
        let writer = LspkWriter::new(source_dir.as_ref())?;
        writer.write_with_progress(output_pak.as_ref(), progress)?;
        Ok(())
    }

    /// Create a PAK file from a directory with specified compression
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the source directory cannot be read or output file cannot be written.
    /// Returns [`Error::WalkDirError`] if directory traversal fails.
    /// Returns [`Error::CompressionError`] if file compression fails.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::WalkDirError`]: crate::Error::WalkDirError
    /// [`Error::CompressionError`]: crate::Error::CompressionError
    pub fn create_with_compression<P: AsRef<Path>>(
        source_dir: P,
        output_pak: P,
        compression: CompressionMethod,
    ) -> Result<()> {
        Self::create_with_compression_and_progress(source_dir, output_pak, compression, &|_| {})
    }

    /// Create a PAK file from a directory with compression and progress callback
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the source directory cannot be read or output file cannot be written.
    /// Returns [`Error::WalkDirError`] if directory traversal fails.
    /// Returns [`Error::CompressionError`] if file compression fails.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::WalkDirError`]: crate::Error::WalkDirError
    /// [`Error::CompressionError`]: crate::Error::CompressionError
    pub fn create_with_compression_and_progress<P: AsRef<Path>>(
        source_dir: P,
        output_pak: P,
        compression: CompressionMethod,
        progress: ProgressCallback,
    ) -> Result<()> {
        let writer = LspkWriter::new(source_dir.as_ref())?.with_compression(compression);
        writer.write_with_progress(output_pak.as_ref(), progress)?;
        Ok(())
    }

    /// List contents of a PAK file
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    pub fn list<P: AsRef<Path>>(pak_path: P) -> Result<Vec<String>> {
        Self::list_with_progress(pak_path, &|_| {})
    }

    /// List contents of a PAK file with progress callback
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    pub fn list_with_progress<P: AsRef<Path>>(
        pak_path: P,
        progress: ProgressCallback,
    ) -> Result<Vec<String>> {
        let file = File::open(pak_path.as_ref())?;

        let mut reader = LspkReader::with_path(file, pak_path.as_ref());

        progress(&PakProgress {
            phase: PakPhase::ReadingHeader,
            current: 1,
            total: 1,
            current_file: None,
        });

        let entries = reader.list_files()?;

        progress(&PakProgress {
            phase: PakPhase::Complete,
            current: entries.len(),
            total: entries.len(),
            current_file: None,
        });

        Ok(entries
            .iter()
            .map(|e| e.path.to_string_lossy().to_string())
            .collect())
    }

    /// List contents of a PAK file with detailed information
    ///
    /// Returns full file entries including sizes and compression info.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    pub(crate) fn list_detailed<P: AsRef<Path>>(pak_path: P) -> Result<Vec<FileTableEntry>> {
        let file = File::open(pak_path.as_ref())?;
        let mut reader = LspkReader::with_path(file, pak_path.as_ref());
        reader.list_files()
    }

    /// Extract specific files from a PAK to a directory
    ///
    /// Takes a list of file paths (as they appear in the PAK) and extracts only those files.
    /// File paths should match exactly as returned by `list()`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened or output directory cannot be created.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    /// Returns [`Error::Lz4DecompressionFailed`] or [`Error::ZlibDecompressionFailed`] if file decompression fails.
    /// Returns [`Error::RequestedFilesNotFound`] if none of the requested files are found.
    /// Returns [`Error::PakExtractionPartialFailure`] if extraction completes with partial failures.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    /// [`Error::DecompressionError`]: crate::Error::DecompressionError
    /// [`Error::ConversionError`]: crate::Error::ConversionError
    pub fn extract_files<P: AsRef<Path>, S: AsRef<str>>(
        pak_path: P,
        output_dir: P,
        file_paths: &[S],
    ) -> Result<()> {
        Self::extract_files_with_progress(pak_path, output_dir, file_paths, &|_| {})
    }

    /// Extract specific files from a PAK to a directory with progress callback
    ///
    /// Takes a list of file paths (as they appear in the PAK) and extracts only those files.
    /// The callback receives [`PakProgress`] with phase and file information.
    /// Uses parallel decompression for improved performance on multi-core systems.
    /// Supports multi-part archives (e.g., `Textures.pak` with `Textures_1.pak`, `Textures_2.pak`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened or output directory cannot be created.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    /// Returns [`Error::Lz4DecompressionFailed`] or [`Error::ZlibDecompressionFailed`] if file decompression fails.
    /// Returns [`Error::RequestedFilesNotFound`] if none of the requested files are found.
    /// Returns [`Error::PakExtractionPartialFailure`] if extraction completes with partial failures.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    /// [`Error::DecompressionError`]: crate::Error::DecompressionError
    /// [`Error::ConversionError`]: crate::Error::ConversionError
    pub fn extract_files_with_progress<P: AsRef<Path>, S: AsRef<str>>(
        pak_path: P,
        output_dir: P,
        file_paths: &[S],
        progress: ProgressCallback,
    ) -> Result<()> {
        if file_paths.is_empty() {
            return Ok(());
        }

        let pak_path = pak_path.as_ref();
        let mut reader = LspkReader::with_path(File::open(pak_path)?, pak_path);

        // Build a set of requested paths for fast lookup
        let requested: std::collections::HashSet<&str> =
            file_paths.iter().map(std::convert::AsRef::as_ref).collect();

        // Get file list and filter to only requested files
        let all_entries = reader.list_files()?;
        let entries_to_extract: Vec<_> = all_entries
            .into_iter()
            .filter(|e| {
                // Skip .DS_Store files
                if e.path.file_name() == Some(std::ffi::OsStr::new(".DS_Store")) {
                    return false;
                }
                requested.contains(e.path.to_string_lossy().as_ref())
            })
            .collect();

        if entries_to_extract.is_empty() {
            return Err(Error::RequestedFilesNotFound);
        }

        std::fs::create_dir_all(&output_dir)?;

        let total_files = entries_to_extract.len();
        let processed = AtomicUsize::new(0);
        let output_dir = output_dir.as_ref();

        // Single-pass parallel extraction: each thread opens its own file handle
        let errors: Vec<(PathBuf, String)> = entries_to_extract
            .par_iter()
            .filter_map(|entry| {
                let file_name = entry.path.file_name().map_or_else(
                    || entry.path.to_string_lossy().to_string(),
                    |n| n.to_string_lossy().to_string(),
                );

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                progress(&PakProgress {
                    phase: PakPhase::DecompressingFiles,
                    current,
                    total: total_files,
                    current_file: Some(file_name.clone()),
                });

                // Get the correct part file path for this entry
                let part_path = match get_part_path(pak_path, entry.archive_part) {
                    Some(p) => p,
                    None => {
                        return Some((
                            entry.path.clone(),
                            format!(
                                "Cannot determine path for archive part {}",
                                entry.archive_part
                            ),
                        ));
                    }
                };

                // Open file handle for this thread, seek, and read
                let mut file = match File::open(&part_path) {
                    Ok(f) => f,
                    Err(e) => {
                        return Some((
                            entry.path.clone(),
                            format!("Failed to open {}: {e}", part_path.display()),
                        ));
                    }
                };

                if let Err(e) = file.seek(SeekFrom::Start(entry.offset)) {
                    return Some((entry.path.clone(), format!("Failed to seek: {e}")));
                }

                let mut compressed_data = vec![0u8; entry.size_compressed as usize];
                if let Err(e) = file.read_exact(&mut compressed_data) {
                    return Some((entry.path.clone(), format!("Failed to read: {e}")));
                }

                // Decompress
                let data = match decompress_data(
                    &compressed_data,
                    entry.compression,
                    entry.size_decompressed,
                ) {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::warn!("Failed to decompress {}: {}", entry.path.display(), e);
                        return Some((entry.path.clone(), e.to_string()));
                    }
                };

                // Calculate output path (handle virtual texture subfolders)
                let output_path = if is_virtual_texture_file(&file_name) {
                    if let Some(subfolder) = get_virtual_texture_subfolder(&file_name) {
                        if let Some(parent) = entry.path.parent() {
                            output_dir.join(parent).join(&subfolder).join(&file_name)
                        } else {
                            output_dir.join(&subfolder).join(&file_name)
                        }
                    } else {
                        output_dir.join(&entry.path)
                    }
                } else {
                    output_dir.join(&entry.path)
                };

                // Create parent directories (idempotent)
                if let Some(parent) = output_path.parent()
                    && let Err(e) = std::fs::create_dir_all(parent)
                {
                    return Some((entry.path.clone(), format!("Failed to create dir: {e}")));
                }

                // Write file
                if let Err(e) = std::fs::write(&output_path, &data) {
                    return Some((entry.path.clone(), format!("Failed to write: {e}")));
                }

                None
            })
            .collect();

        // If there were errors, return a summary error
        if !errors.is_empty() {
            return Err(Error::PakExtractionPartialFailure {
                total: total_files,
                failed: errors.len(),
                first_error: errors[0].1.clone(),
            });
        }

        Ok(())
    }

    /// Read a single file's bytes from a PAK without writing to disk
    ///
    /// Returns the decompressed file contents, or an error if the file is not found.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    /// Returns [`Error::FileNotFoundInPak`] if the requested file path is not in the archive.
    /// Returns [`Error::DecompressionError`] if file decompression fails.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    /// [`Error::FileNotFoundInPak`]: crate::Error::FileNotFoundInPak
    /// [`Error::DecompressionError`]: crate::Error::DecompressionError
    pub fn read_file_bytes<P: AsRef<Path>>(pak_path: P, file_path: &str) -> Result<Vec<u8>> {
        let file = File::open(pak_path.as_ref())?;
        let mut reader = LspkReader::with_path(file, pak_path.as_ref());

        // Get file list
        let entries = reader.list_files()?;

        // Find the requested file
        let entry = entries
            .into_iter()
            .find(|e| e.path.to_string_lossy() == file_path)
            .ok_or_else(|| Error::FileNotFoundInPak(file_path.to_string()))?;

        // Decompress and return
        reader.decompress_file(&entry)
    }

    /// Read multiple files' bytes from a PAK without writing to disk
    ///
    /// Returns a map of file paths to their decompressed contents.
    /// Files that fail to decompress are skipped with a warning.
    /// Uses parallel decompression for improved performance on multi-core systems.
    /// Supports multi-part archives (e.g., `Textures.pak` with `Textures_1.pak`, `Textures_2.pak`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    /// Returns [`Error::ConversionError`] if a required archive part file cannot be found.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    /// [`Error::ConversionError`]: crate::Error::ConversionError
    pub fn read_files_bytes<P: AsRef<Path>, S: AsRef<str>>(
        pak_path: P,
        file_paths: &[S],
    ) -> Result<HashMap<String, Vec<u8>>> {
        if file_paths.is_empty() {
            return Ok(HashMap::new());
        }

        let pak_path = pak_path.as_ref();
        let mut reader = LspkReader::with_path(File::open(pak_path)?, pak_path);

        // Build a set of requested paths
        let requested: std::collections::HashSet<&str> =
            file_paths.iter().map(std::convert::AsRef::as_ref).collect();

        // Get file list and filter
        let all_entries = reader.list_files()?;
        let entries_to_read: Vec<_> = all_entries
            .into_iter()
            .filter(|e| requested.contains(e.path.to_string_lossy().as_ref()))
            .collect();

        // Group entries by archive part for multi-part PAK support
        let mut entries_by_part: HashMap<u8, Vec<FileTableEntry>> = HashMap::new();
        for entry in entries_to_read {
            entries_by_part
                .entry(entry.archive_part)
                .or_default()
                .push(entry);
        }

        // Phase 1: Read all compressed data sequentially from each part file
        let mut compressed_files: Vec<(String, CompressedFile)> = Vec::new();

        for (part, part_entries) in &entries_by_part {
            let part_path = get_part_path(pak_path, *part)
                .ok_or(Error::ArchivePartNotFound { part: *part })?;

            if !part_path.exists() {
                tracing::warn!("Archive part file not found: {}", part_path.display());
                continue;
            }

            let mut part_file = File::open(&part_path)?;

            for entry in part_entries {
                // Seek and read compressed data from the correct part file
                if part_file.seek(SeekFrom::Start(entry.offset)).is_err() {
                    tracing::warn!(
                        "Failed to seek to {} in {}",
                        entry.path.display(),
                        part_path.display()
                    );
                    continue;
                }

                let mut compressed_data = vec![0u8; entry.size_compressed as usize];
                if part_file.read_exact(&mut compressed_data).is_err() {
                    tracing::warn!(
                        "Failed to read {} from {}",
                        entry.path.display(),
                        part_path.display()
                    );
                    continue;
                }

                compressed_files.push((
                    entry.path.to_string_lossy().to_string(),
                    CompressedFile {
                        entry: entry.clone(),
                        compressed_data,
                    },
                ));
            }
        }

        // Phase 2: Decompress in parallel and collect results
        let decompressed: Vec<(String, Vec<u8>)> = compressed_files
            .par_iter()
            .filter_map(|(path, cf)| {
                match decompress_data(
                    &cf.compressed_data,
                    cf.entry.compression,
                    cf.entry.size_decompressed,
                ) {
                    Ok(data) => Some((path.clone(), data)),
                    Err(e) => {
                        tracing::warn!("Failed to decompress {}: {}", path, e);
                        None
                    }
                }
            })
            .collect();

        // Collect into HashMap
        Ok(decompressed.into_iter().collect())
    }

    /// Extract meta.lsx from a PAK
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the PAK file cannot be opened.
    /// Returns [`Error::InvalidPakMagic`] if the file is not a valid PAK archive.
    /// Returns [`Error::FileNotFoundInPak`] if `meta.lsx` is not found in the archive.
    /// Returns [`Error::ConversionError`] if `meta.lsx` contains invalid UTF-8.
    ///
    /// [`Error::Io`]: crate::Error::Io
    /// [`Error::InvalidPakMagic`]: crate::Error::InvalidPakMagic
    /// [`Error::FileNotFoundInPak`]: crate::Error::FileNotFoundInPak
    /// [`Error::ConversionError`]: crate::Error::ConversionError
    pub fn extract_meta<P: AsRef<Path>>(pak_path: P) -> Result<String> {
        let file = File::open(pak_path.as_ref())?;

        let mut reader = LspkReader::with_path(file, pak_path.as_ref());
        let contents = reader.read_all(None)?;

        // Find meta.lsx
        let meta_file = contents
            .files
            .iter()
            .find(|f| {
                let path = &f.path;
                let mut components = path.components();

                // Look for Mods/*/meta.lsx pattern
                if let Some(first) = components.next()
                    && first.as_os_str() == "Mods"
                    && components.next().is_some()
                    && let Some(third) = components.next()
                {
                    return third.as_os_str() == "meta.lsx";
                }
                false
            })
            .ok_or_else(|| Error::FileNotFoundInPak("meta.lsx".to_string()))?;

        String::from_utf8(meta_file.data.clone()).map_err(Error::from)
    }
}
