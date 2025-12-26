//! LSPK PAK file reader with progress callbacks and error recovery

use std::ffi::OsStr;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

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
}

impl<R: Read + Seek> LspkReader<R> {
    /// Create a new reader from a Read + Seek source
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            header: None,
            footer: None,
            file_table: Vec::new(),
        }
    }

    /// Read and parse the PAK file header
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

        if version < MIN_VERSION || version > MAX_VERSION {
            return Err(Error::ConversionError(format!(
                "Unsupported PAK version: {} (supported: {}-{})",
                version, MIN_VERSION, MAX_VERSION
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
            .map_err(|e| Error::DecompressionError(format!("Failed to decompress file table: {}", e)))?;

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
    fn parse_file_entry(&self, bytes: &[u8], version: u32) -> Result<FileTableEntry> {
        // Path: bytes 0-255 (null-terminated string)
        let path_end = bytes[..PATH_LENGTH]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(PATH_LENGTH);
        let path = PathBuf::from(OsStr::from_bytes(&bytes[..path_end]));

        // Offset: bytes 256-263 (complex encoding)
        // The offset is stored as a 6-byte value with some flags mixed in
        let offset_low = u32::from_le_bytes(bytes[256..260].try_into().unwrap());
        let offset_high = u16::from_le_bytes(bytes[260..262].try_into().unwrap());
        let mut offset = u64::from(offset_low) | (u64::from(offset_high) << 32);

        // Mask out flag bits - offset uses lower 52 bits
        offset &= 0x000F_FFFF_FFFF_FFFF;

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
        })
    }

    /// Decompress a single file from the PAK
    pub fn decompress_file(&mut self, entry: &FileTableEntry) -> Result<Vec<u8>> {
        // Seek to the file data
        self.reader.seek(SeekFrom::Start(entry.offset))?;

        // Read compressed data
        let mut compressed = vec![0u8; entry.size_compressed as usize];
        self.reader.read_exact(&mut compressed)?;

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

            CompressionMethod::Zstd => {
                self.decompress_zstd(&compressed, entry.size_decompressed as usize, &entry.path)
            }
        }
    }

    /// Decompress LZ4 data with multiple fallback strategies
    fn decompress_lz4(&self, compressed: &[u8], expected_size: usize, path: &PathBuf) -> Result<Vec<u8>> {
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
    fn decompress_zlib(&self, compressed: &[u8], expected_size: usize, path: &PathBuf) -> Result<Vec<u8>> {
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

    /// Decompress Zstd data
    fn decompress_zstd(&self, compressed: &[u8], expected_size: usize, path: &PathBuf) -> Result<Vec<u8>> {
        zstd::decode_all(compressed)
            .map_err(|e| Error::DecompressionError(format!(
                "Failed to decompress Zstd data for {}: {}",
                path.display(),
                e
            )))
    }

    /// Read the entire PAK file with progress callbacks and error recovery
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
        let entries: Vec<_> = self.file_table.iter().cloned().collect();

        // Decompress each file
        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.path.to_string_lossy().to_string());

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
