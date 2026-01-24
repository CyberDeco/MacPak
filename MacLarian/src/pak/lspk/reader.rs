//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0
//!
//! LSPK PAK file reader with progress callbacks and error recovery

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
// use ruzstd::StreamingDecoder;

use crate::error::{Error, Result};
use super::{
    CompressionMethod, FileTableEntry, LspkFooter, LspkHeader, PakContents, PakFile,
    PakPhase, PakProgress, MAGIC, MAX_VERSION, MIN_VERSION, PATH_LENGTH, TABLE_ENTRY_SIZE,
};

/// Progress callback type
pub type ProgressCallback<'a> = &'a dyn Fn(&PakProgress);

/// LSPK PAK file reader
pub struct LspkReader<R: Read + Seek> {
    reader: BufReader<R>,
    header: Option<LspkHeader>,
    footer: Option<LspkFooter>,
    file_table: Vec<FileTableEntry>,
    /// Base path for the main PAK file (used to find part files)
    pak_path: Option<PathBuf>,
    /// Cached readers for archive parts (lazily opened)
    part_readers: HashMap<u8, BufReader<File>>,
}

impl<R: Read + Seek> LspkReader<R> {
    /// Create a new reader from a Read + Seek source
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            header: None,
            footer: None,
            file_table: Vec::new(),
            pak_path: None,
            part_readers: HashMap::new(),
        }
    }

    /// Create a new reader with path information for multi-part archive support
    pub fn with_path(reader: R, path: impl AsRef<Path>) -> Self {
        Self {
            reader: BufReader::new(reader),
            header: None,
            footer: None,
            file_table: Vec::new(),
            pak_path: Some(path.as_ref().to_path_buf()),
            part_readers: HashMap::new(),
        }
    }

    /// Get the path for a specific archive part
    fn get_part_path(&self, part: u8) -> Option<PathBuf> {
        let base_path = self.pak_path.as_ref()?;
        if part == 0 {
            return Some(base_path.clone());
        }

        let stem = base_path.file_stem()?.to_str()?;
        let ext = base_path.extension()?.to_str()?;
        let parent = base_path.parent()?;

        Some(parent.join(format!("{stem}_{part}.{ext}")))
    }

    /// Get or open a reader for a specific archive part
    fn get_part_reader(&mut self, part: u8) -> Result<&mut dyn ReadSeek> {
        if part == 0 {
            return Ok(&mut self.reader);
        }

        // Check if we already have this part open
        if !self.part_readers.contains_key(&part) {
            let part_path = self.get_part_path(part)
                .ok_or_else(|| Error::ConversionError(
                    format!("Cannot determine path for archive part {part}")
                ))?;

            if !part_path.exists() {
                return Err(Error::ConversionError(
                    format!("Archive part file not found: {}", part_path.display())
                ));
            }

            let file = File::open(&part_path)?;
            self.part_readers.insert(part, BufReader::new(file));
        }

        Ok(self.part_readers.get_mut(&part).unwrap())
    }
}

/// Trait for types that can Read and Seek
trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

impl<R: Read + Seek> LspkReader<R> {
    /// Read and parse the PAK file header
    ///
    /// # Errors
    /// Returns an error if reading fails or the magic number is invalid.
    ///
    /// # Panics
    /// This function does not panic under normal conditions.
    pub fn read_header(&mut self) -> Result<&LspkHeader> {
        self.reader.seek(SeekFrom::Start(0))?;

        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic)?;

        if magic != MAGIC {
            return Err(Error::InvalidPakMagic);
        }

        let mut version_bytes = [0u8; 4];
        self.reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);

        if !(MIN_VERSION..=MAX_VERSION).contains(&version) {
            return Err(Error::ConversionError(format!(
                "Unsupported PAK version: {version} (supported: {MIN_VERSION}-{MAX_VERSION})"
            )));
        }

        let mut offset_bytes = [0u8; 8];
        self.reader.read_exact(&mut offset_bytes)?;
        let footer_offset = u64::from_le_bytes(offset_bytes);

        self.header = Some(LspkHeader {
            magic,
            version,
            footer_offset,
        });

        Ok(self.header.as_ref().unwrap())
    }

    /// Read and parse the PAK file footer
    ///
    /// # Errors
    /// Returns an error if the header hasn't been read or reading fails.
    ///
    /// # Panics
    /// This function does not panic under normal conditions.
    pub fn read_footer(&mut self) -> Result<&LspkFooter> {
        let header = self.header.as_ref()
            .ok_or_else(|| Error::ConversionError("Header not read yet".to_string()))?;

        // Footer offset in header is absolute position from start of file
        self.reader.seek(SeekFrom::Start(header.footer_offset))?;

        let mut num_files_bytes = [0u8; 4];
        self.reader.read_exact(&mut num_files_bytes)?;
        let num_files = u32::from_le_bytes(num_files_bytes);

        let mut table_size_bytes = [0u8; 4];
        self.reader.read_exact(&mut table_size_bytes)?;
        let table_size_compressed = u32::from_le_bytes(table_size_bytes);

        self.footer = Some(LspkFooter {
            num_files,
            table_size_compressed,
        });

        Ok(self.footer.as_ref().unwrap())
    }

    /// Read and decompress the file table
    ///
    /// # Errors
    /// Returns an error if the footer hasn't been read or decompression fails.
    ///
    /// # Panics
    /// This function does not panic under normal conditions.
    pub fn read_file_table(&mut self) -> Result<&[FileTableEntry]> {
        let footer = self.footer.as_ref()
            .ok_or_else(|| Error::ConversionError("Footer not read yet".to_string()))?;
        let header = self.header.as_ref().unwrap();

        let num_files = footer.num_files as usize;
        let table_size_compressed = footer.table_size_compressed as usize;
        let table_size_decompressed = num_files * TABLE_ENTRY_SIZE;

        // Read compressed table data
        let mut compressed_table = vec![0u8; table_size_compressed];
        self.reader.read_exact(&mut compressed_table)?;

        // Decompress the table using LZ4
        let decompressed_table = lz4_flex::block::decompress(&compressed_table, table_size_decompressed)
            .map_err(|e| Error::DecompressionError(format!("Failed to decompress file table: {e}")))?;

        // Parse file entries
        self.file_table.clear();
        self.file_table.reserve(num_files);

        for i in 0..num_files {
            let entry_start = i * TABLE_ENTRY_SIZE;
            let entry_bytes = &decompressed_table[entry_start..entry_start + TABLE_ENTRY_SIZE];

            let entry = self.parse_file_entry(entry_bytes, header.version)?;
            self.file_table.push(entry);
        }

        Ok(&self.file_table)
    }

    /// Parse a single file table entry
    fn parse_file_entry(&self, bytes: &[u8], _version: u32) -> Result<FileTableEntry> {
        // Path: bytes 0-255 (null-terminated string)
        let path_end = bytes[..PATH_LENGTH]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(PATH_LENGTH);
        // Use lossy UTF-8 conversion for cross-platform compatibility
        let path_str = String::from_utf8_lossy(&bytes[..path_end]);
        let path = PathBuf::from(path_str.as_ref());

        // Offset: bytes 256-261 (6 bytes)
        // The offset is stored as a 6-byte value
        let offset_low = u32::from_le_bytes(bytes[256..260].try_into().unwrap());
        let offset_high = u16::from_le_bytes(bytes[260..262].try_into().unwrap());
        let mut offset = u64::from(offset_low) | (u64::from(offset_high) << 32);

        // Mask out flag bits - offset uses lower 48 bits
        offset &= 0x0000_FFFF_FFFF_FFFF;

        // Archive part: byte 262
        let archive_part = bytes[262];

        // Flags byte at 263 contains compression type in lower nibble
        let flags = bytes[263];
        let compression = CompressionMethod::from_flags(flags);

        // Compressed size: bytes 264-267
        let size_compressed = u32::from_le_bytes(bytes[264..268].try_into().unwrap());

        // Decompressed size: bytes 268-271
        let size_decompressed = u32::from_le_bytes(bytes[268..272].try_into().unwrap());

        Ok(FileTableEntry {
            path,
            offset,
            size_compressed,
            size_decompressed,
            compression,
            flags,
            archive_part,
        })
    }

    /// Decompress a single file from the PAK
    ///
    /// # Errors
    /// Returns an error if reading or decompression fails.
    pub fn decompress_file(&mut self, entry: &FileTableEntry) -> Result<Vec<u8>> {
        // Get the appropriate reader for this archive part
        let reader = self.get_part_reader(entry.archive_part)?;

        // Seek to the file data
        reader.seek(SeekFrom::Start(entry.offset))?;

        // Read compressed data
        let mut compressed = vec![0u8; entry.size_compressed as usize];
        reader.read_exact(&mut compressed)?;

        // If no compression or zero size, return as-is
        if entry.compression == CompressionMethod::None || entry.size_decompressed == 0 {
            return Ok(compressed);
        }

        // Decompress based on method
        match entry.compression {
            CompressionMethod::None => Ok(compressed),

            CompressionMethod::Lz4 => {
                self.decompress_lz4(&compressed, entry.size_decompressed as usize, &entry.path)
            }

            CompressionMethod::Zlib => {
                self.decompress_zlib(&compressed, entry.size_decompressed as usize, &entry.path)
            }

            // CompressionMethod::Zstd => {
            //     self.decompress_zstd(&compressed, entry.size_decompressed as usize, &entry.path)
            // }
        }
    }

    /// Decompress LZ4 data with multiple fallback strategies
    fn decompress_lz4(&self, compressed: &[u8], expected_size: usize, path: &Path) -> Result<Vec<u8>> {
        // Try standard block decompression first
        if let Ok(data) = lz4_flex::block::decompress(compressed, expected_size) {
            return Ok(data);
        }

        // Try with a larger buffer (metadata might be wrong)
        let larger_size = expected_size.saturating_mul(2).max(65536);
        if let Ok(data) = lz4_flex::block::decompress(compressed, larger_size) {
            return Ok(data);
        }

        // Try decompressing without size hint using uncompressed size detection
        if let Ok(data) = lz4_flex::decompress_size_prepended(compressed) {
            return Ok(data);
        }

        // If all else fails, try treating it as a frame
        let mut decoder = lz4_flex::frame::FrameDecoder::new(compressed);
        let mut decompressed = Vec::with_capacity(expected_size);
        if decoder.read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
            return Ok(decompressed);
        }

        Err(Error::DecompressionError(format!(
            "Failed to decompress LZ4 data for {}: all methods failed (compressed: {} bytes, expected: {} bytes)",
            path.display(),
            compressed.len(),
            expected_size
        )))
    }

    /// Decompress Zlib data
    fn decompress_zlib(&self, compressed: &[u8], expected_size: usize, path: &Path) -> Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;

        let mut decoder = ZlibDecoder::new(compressed);
        let mut decompressed = Vec::with_capacity(expected_size);

        decoder.read_to_end(&mut decompressed)
            .map_err(|e| Error::DecompressionError(format!(
                "Failed to decompress Zlib data for {}: {}",
                path.display(),
                e
            )))?;

        Ok(decompressed)
    }

    // Decompress Zstd data
    // fn decompress_zstd(&self, compressed: &[u8], expected_size: usize, path: &PathBuf) -> Result<Vec<u8>> {
    //     zstd::decode_all(compressed)
    //         .map_err(|e| Error::DecompressionError(format!(
    //             "Failed to decompress Zstd data for {}: {}",
    //             path.display(),
    //             e
    //         )))
    // }

    /// Read the entire PAK file with progress callbacks and error recovery
    ///
    /// # Errors
    /// Returns an error if reading or decompression fails.
    ///
    /// # Panics
    /// This function does not panic under normal conditions.
    pub fn read_all(&mut self, progress: Option<ProgressCallback>) -> Result<PakContents> {
        let progress = progress.unwrap_or(&|_| {});

        // Read header
        progress(&PakProgress {
            phase: PakPhase::ReadingHeader,
            current: 0,
            total: 1,
            current_file: None,
        });
        self.read_header()?;

        // Read footer
        self.read_footer()?;

        // Read file table
        progress(&PakProgress {
            phase: PakPhase::ReadingTable,
            current: 0,
            total: 1,
            current_file: None,
        });
        self.read_file_table()?;

        let version = self.header.as_ref().unwrap().version;
        let mut contents = PakContents::new(version);
        let total_files = self.file_table.len();

        // Clone file table to avoid borrow issues
        let entries: Vec<_> = self.file_table.clone();

        // Decompress each file
        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.path.file_name().map_or_else(|| entry.path.to_string_lossy().to_string(), |n| n.to_string_lossy().to_string());

            progress(&PakProgress {
                phase: PakPhase::DecompressingFiles,
                current: i + 1,
                total: total_files,
                current_file: Some(file_name),
            });

            match self.decompress_file(entry) {
                Ok(data) => {
                    contents.files.push(PakFile {
                        path: entry.path.clone(),
                        data,
                    });
                }
                Err(e) => {
                    // Record error but continue with other files
                    contents.errors.push((entry.path.clone(), e.to_string()));
                }
            }
        }

        progress(&PakProgress {
            phase: PakPhase::Complete,
            current: total_files,
            total: total_files,
            current_file: None,
        });

        Ok(contents)
    }

    /// List files in the PAK without decompressing them
    ///
    /// # Errors
    /// Returns an error if reading the file table fails.
    pub fn list_files(&mut self) -> Result<Vec<FileTableEntry>> {
        if self.header.is_none() {
            self.read_header()?;
        }
        if self.footer.is_none() {
            self.read_footer()?;
        }
        if self.file_table.is_empty() {
            self.read_file_table()?;
        }

        Ok(self.file_table.clone())
    }

    /// Get the PAK version
    pub fn version(&self) -> Option<u32> {
        self.header.as_ref().map(|h| h.version)
    }

    /// Get the number of files in the PAK
    pub fn file_count(&self) -> Option<u32> {
        self.footer.as_ref().map(|f| f.num_files)
    }
}