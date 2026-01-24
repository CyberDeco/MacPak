//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0
//!
//! LSPK PAK file writer with progress callbacks
//!
//! Uses parallel compression for improved performance on multi-core systems.

use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;

use crate::error::{Error, Result};
use super::{CompressionMethod, MAGIC, MAX_VERSION, PATH_LENGTH, TABLE_ENTRY_SIZE};

/// Progress callback type for write operations
/// Must be Sync + Send to support parallel compression.
pub type WriteProgressCallback<'a> = &'a (dyn Fn(usize, usize, &str) + Sync + Send);

/// File to be written to the PAK
struct FileEntry {
    /// Path relative to the root
    relative_path: PathBuf,
    /// File contents
    data: Vec<u8>,
}

/// Details about a written file entry
struct WrittenEntry {
    path: PathBuf,
    offset: u64,
    size_compressed: u32,
    size_decompressed: u32,
}

/// Compressed file ready for writing
struct CompressedEntry {
    path: PathBuf,
    compressed_data: Vec<u8>,
    size_decompressed: u32,
}

/// LSPK PAK file writer
pub struct LspkWriter {
    /// Root path of the mod directory
    root_path: PathBuf,
    /// Files to include in the PAK
    files: Vec<FileEntry>,
    /// PAK version to write
    version: u32,
    /// Compression method to use
    compression: CompressionMethod,
}

impl LspkWriter {
    /// Create a new writer for the given directory
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read.
    pub fn new(root_path: impl Into<PathBuf>) -> Result<Self> {
        let root_path = root_path.into();
        let files = Self::collect_files(&root_path)?;

        Ok(Self {
            root_path,
            files,
            version: MAX_VERSION, // Use latest supported version
            compression: CompressionMethod::Lz4, // Default to LZ4
        })
    }

    /// Set the PAK version to write
    #[must_use] 
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Set the compression method to use
    #[must_use] 
    pub fn with_compression(mut self, compression: CompressionMethod) -> Self {
        self.compression = compression;
        self
    }

    /// Collect all files from a directory recursively
    fn collect_files(root: &Path) -> Result<Vec<FileEntry>> {
        let mut files = Vec::new();
        let mut dirs_to_check = VecDeque::new();
        dirs_to_check.push_back(root.to_path_buf());

        while let Some(dir) = dirs_to_check.pop_front() {
            let entries = std::fs::read_dir(&dir)?;

            for entry in entries {
                let entry = entry?;
                let file_type = entry.file_type()?;
                let path = entry.path();

                // Skip symlinks
                if file_type.is_symlink() {
                    continue;
                }

                // Skip .DS_Store files
                if entry.file_name() == ".DS_Store" {
                    continue;
                }

                if file_type.is_dir() {
                    dirs_to_check.push_back(path);
                } else {
                    let relative_path = path.strip_prefix(root)
                        .map_err(|_| Error::InvalidPath(path.display().to_string()))?
                        .to_path_buf();

                    let data = std::fs::read(&path)?;

                    files.push(FileEntry {
                        relative_path,
                        data,
                    });
                }
            }
        }

        Ok(files)
    }

    /// Write the PAK file
    ///
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn write(self, output_path: impl AsRef<Path>) -> Result<()> {
        self.write_with_progress(output_path, &|_, _, _| {})
    }

    /// Write the PAK file with progress callback
    ///
    /// Uses parallel compression for improved performance on multi-core systems.
    /// Files are compressed in parallel, then written sequentially to maintain
    /// correct file offsets in the PAK.
    ///
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn write_with_progress(
        self,
        output_path: impl AsRef<Path>,
        progress: WriteProgressCallback,
    ) -> Result<()> {
        let output_path = output_path.as_ref();

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let total_files = self.files.len();
        let processed = AtomicUsize::new(0);
        let compression = self.compression;

        let compress_label = match compression {
            CompressionMethod::None => "Storing",
            CompressionMethod::Lz4 => "Compressing",
            CompressionMethod::Zlib => "Compressing",
        };

        // Phase 1: Compress all files in parallel
        progress(0, total_files, "Compressing files...");

        let compression_results: Vec<std::result::Result<CompressedEntry, String>> = self
            .files
            .par_iter()
            .map(|file| {
                let file_name = file
                    .relative_path
                    .file_name().map_or_else(|| file.relative_path.to_string_lossy().to_string(), |n| n.to_string_lossy().to_string());

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                progress(current, total_files, &format!("{compress_label} {file_name}"));

                let size_decompressed = file.data.len();

                // Validate size fits in u32
                let size_decompressed: u32 = size_decompressed
                    .try_into()
                    .map_err(|_| format!("File {file_name} is too large: {size_decompressed} bytes"))?;

                let compressed_data = match compression {
                    CompressionMethod::None => file.data.clone(),
                    CompressionMethod::Lz4 => lz4_flex::block::compress(&file.data),
                    CompressionMethod::Zlib => {
                        use flate2::write::ZlibEncoder;
                        use flate2::Compression;
                        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                        encoder
                            .write_all(&file.data)
                            .map_err(|e| format!("Failed to compress {file_name}: {e}"))?;
                        encoder
                            .finish()
                            .map_err(|e| format!("Failed to finish compression for {file_name}: {e}"))?
                    }
                };

                Ok(CompressedEntry {
                    path: file.relative_path.clone(),
                    compressed_data,
                    size_decompressed,
                })
            })
            .collect();

        // Check for compression errors
        let mut compressed_entries = Vec::with_capacity(total_files);
        for result in compression_results {
            match result {
                Ok(entry) => compressed_entries.push(entry),
                Err(e) => return Err(Error::ConversionError(e)),
            }
        }

        // Phase 2: Write compressed data sequentially (to maintain correct offsets)
        progress(total_files, total_files, "Writing PAK file...");

        let mut output = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output_path)?;

        // Write header
        output.write_all(&MAGIC)?;
        output.write_all(&self.version.to_le_bytes())?;
        // Placeholder for footer offset (will be filled in later)
        output.write_all(&0u64.to_le_bytes())?;

        let mut written_entries = Vec::with_capacity(compressed_entries.len());

        for entry in compressed_entries {
            let size_compressed: u32 = entry.compressed_data.len().try_into().map_err(|_| {
                Error::ConversionError(format!(
                    "Compressed file {} is too large: {} bytes",
                    entry.path.display(),
                    entry.compressed_data.len()
                ))
            })?;

            let offset = output.stream_position()?;
            output.write_all(&entry.compressed_data)?;

            written_entries.push(WrittenEntry {
                path: entry.path,
                offset,
                size_compressed,
                size_decompressed: entry.size_decompressed,
            });
        }

        // Record footer position
        let footer_offset = output.stream_position()?;

        // Write footer: number of files
        let num_files: u32 = written_entries.len().try_into().map_err(|_| {
            Error::ConversionError(format!("Too many files: {}", written_entries.len()))
        })?;
        output.write_all(&num_files.to_le_bytes())?;

        // Build file table
        progress(total_files, total_files, "Building file table...");

        let mut table_data = Vec::with_capacity(TABLE_ENTRY_SIZE * written_entries.len());

        for entry in &written_entries {
            let entry_start = table_data.len();

            // Path (256 bytes, null-padded)
            let path_bytes = entry.path.as_os_str().as_encoded_bytes();
            table_data.extend_from_slice(path_bytes);
            table_data.resize(entry_start + PATH_LENGTH, 0);

            // Offset (8 bytes)
            table_data.extend_from_slice(&entry.offset.to_le_bytes());

            // Compressed size (4 bytes)
            table_data.extend_from_slice(&entry.size_compressed.to_le_bytes());

            // Decompressed size (4 bytes)
            table_data.extend_from_slice(&entry.size_decompressed.to_le_bytes());
        }

        // Compress and write file table
        let compressed_table = lz4_flex::block::compress(&table_data);
        let table_size: u32 = compressed_table.len().try_into().map_err(|_| {
            Error::ConversionError(format!(
                "File table too large: {} bytes",
                compressed_table.len()
            ))
        })?;

        output.write_all(&table_size.to_le_bytes())?;
        output.write_all(&compressed_table)?;

        // Go back and write the footer offset
        output.seek(SeekFrom::Start(8))?;
        output.write_all(&footer_offset.to_le_bytes())?;

        progress(total_files, total_files, "Complete");

        Ok(())
    }

    /// Get the number of files that will be written
    #[must_use] 
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Get the root path
    #[must_use] 
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }
}
