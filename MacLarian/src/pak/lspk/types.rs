//! Types for LSPK PAK file handling
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0

use std::path::PathBuf;

/// Compression method used for a file in the PAK
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMethod {
    None,
    Zlib,
    Lz4,
}

impl CompressionMethod {
    /// Parse compression method from the flags byte
    #[must_use]
    pub fn from_flags(flags: u8) -> Self {
        match flags & 0x0F {
            0 => CompressionMethod::None,
            1 => CompressionMethod::Zlib,
            2 => CompressionMethod::Lz4,
            _ => CompressionMethod::None, // Unknown, treat as uncompressed
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            CompressionMethod::None => "none",
            CompressionMethod::Zlib => "zlib",
            CompressionMethod::Lz4 => "lz4",
        }
    }

    /// Convert compression method to flags byte for writing
    #[must_use]
    pub fn to_flags(self) -> u8 {
        match self {
            CompressionMethod::None => 0,
            CompressionMethod::Zlib => 1,
            CompressionMethod::Lz4 => 2,
        }
    }
}

/// Header of an LSPK PAK file
#[derive(Debug, Clone)]
pub(crate) struct LspkHeader {
    /// Magic bytes (should be "LSPK")
    ///
    /// Validated during parsing but retained for format completeness.
    #[allow(dead_code)]
    pub magic: [u8; 4],
    /// Version number
    pub version: u32,
    /// Offset to the footer from the start of the file
    pub footer_offset: u64,
}

/// Footer/metadata of an LSPK PAK file
#[derive(Debug, Clone)]
pub(crate) struct LspkFooter {
    /// Number of files in the archive
    pub num_files: u32,
    /// Size of the compressed file table
    pub table_size_compressed: u32,
}

/// Entry in the file table describing a file in the PAK
#[derive(Debug, Clone)]
pub(crate) struct FileTableEntry {
    /// Path of the file within the archive
    pub path: PathBuf,
    /// Offset of the compressed data from the start of the archive part
    pub offset: u64,
    /// Size of the compressed data
    pub size_compressed: u32,
    /// Size of the decompressed data
    pub size_decompressed: u32,
    /// Compression method
    pub compression: CompressionMethod,
    /// Raw flags byte from PAK format (parsed for completeness, useful for debugging)
    #[allow(dead_code)]
    pub flags: u8,
    /// Archive part number (0 = main .pak, 1+ = _1.pak, _2.pak, etc.)
    pub archive_part: u8,
}

/// A decompressed file from the PAK archive
#[derive(Debug, Clone)]
pub struct PakFile {
    /// Path of the file within the archive
    pub path: PathBuf,
    /// Decompressed file contents
    pub data: Vec<u8>,
}

/// Result of reading a PAK file, with support for partial success
#[derive(Debug)]
pub struct PakContents {
    /// Successfully extracted files
    pub files: Vec<PakFile>,
    /// Files that failed to extract (path, error message)
    pub errors: Vec<(PathBuf, String)>,
    /// PAK version
    pub version: u32,
}

impl PakContents {
    #[must_use]
    pub fn new(version: u32) -> Self {
        Self {
            files: Vec::new(),
            errors: Vec::new(),
            version,
        }
    }

    /// Returns true if all files were extracted successfully
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the total number of files (successful + failed)
    #[must_use]
    pub fn total_files(&self) -> usize {
        self.files.len() + self.errors.len()
    }
}

/// Progress information during PAK operations
#[derive(Debug, Clone)]
pub struct PakProgress {
    /// Current operation phase
    pub phase: PakPhase,
    /// Current item number (1-indexed)
    pub current: usize,
    /// Total number of items
    pub total: usize,
    /// Current file being processed (if applicable)
    pub current_file: Option<String>,
}

impl PakProgress {
    /// Create a new progress update
    #[must_use]
    pub fn new(phase: PakPhase, current: usize, total: usize) -> Self {
        Self {
            phase,
            current,
            total,
            current_file: None,
        }
    }

    /// Create a progress update with a file/item name
    #[must_use]
    pub fn with_file(
        phase: PakPhase,
        current: usize,
        total: usize,
        file: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            current,
            total,
            current_file: Some(file.into()),
        }
    }

    /// Get the progress percentage (0.0 - 1.0)
    #[must_use]
    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            self.current as f32 / self.total as f32
        }
    }
}

/// Phase of PAK operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PakPhase {
    /// Reading PAK header
    ReadingHeader,
    /// Reading and decompressing file table
    ReadingTable,
    /// Decompressing individual files
    DecompressingFiles,
    /// Scanning files in a directory (during PAK creation)
    ScanningFiles,
    /// Compressing files (during PAK creation)
    CompressingFiles,
    /// Writing file table to PAK
    WritingTable,
    /// Writing files to disk (during extraction)
    WritingFiles,
    /// Operation complete
    Complete,
}

impl PakPhase {
    /// Get a human-readable description of this phase
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadingHeader => "Reading header",
            Self::ReadingTable => "Reading file table",
            Self::DecompressingFiles => "Decompressing files",
            Self::ScanningFiles => "Scanning files",
            Self::CompressingFiles => "Compressing files",
            Self::WritingTable => "Writing file table",
            Self::WritingFiles => "Writing files",
            Self::Complete => "Complete",
        }
    }
}
