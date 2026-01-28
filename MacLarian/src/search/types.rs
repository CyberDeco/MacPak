//! Types for the search index module

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File type classification for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileType {
    Lsx,
    Lsf,
    Lsj,
    Lsbc,
    Xml,
    Json,
    Dds,
    Png,
    Gr2,
    Wem,
    Gts,
    Gtp,
    Other,
}

impl FileType {
    /// Determine file type from extension
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "lsx" => FileType::Lsx,
            "lsf" => FileType::Lsf,
            "lsj" => FileType::Lsj,
            "lsbc" | "lsbs" | "lsbx" => FileType::Lsbc,
            "xml" => FileType::Xml,
            "json" => FileType::Json,
            "dds" => FileType::Dds,
            "png" | "jpg" | "jpeg" | "tga" | "bmp" => FileType::Png,
            "gr2" => FileType::Gr2,
            "wem" | "ogg" | "wav" => FileType::Wem,
            "gts" => FileType::Gts,
            "gtp" => FileType::Gtp,
            _ => FileType::Other,
        }
    }

    /// Check if this is a text-based format that can be content-searched
    #[must_use]
    pub fn is_searchable_text(&self) -> bool {
        matches!(
            self,
            FileType::Lsx | FileType::Lsf | FileType::Lsj | FileType::Xml | FileType::Json
        )
    }

    /// Get display name for UI
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            FileType::Lsx => "LSX",
            FileType::Lsf => "LSF",
            FileType::Lsj => "LSJ",
            FileType::Lsbc => "LSBC",
            FileType::Xml => "XML",
            FileType::Json => "JSON",
            FileType::Dds => "DDS",
            FileType::Png => "Image",
            FileType::Gr2 => "GR2",
            FileType::Wem => "Audio",
            FileType::Gts => "GTS",
            FileType::Gtp => "GTP",
            FileType::Other => "Other",
        }
    }
}

/// Metadata for an indexed file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    /// Filename only (without path)
    pub name: String,
    /// Full internal path within PAK
    pub path: String,
    /// Source PAK file
    pub pak_file: PathBuf,
    /// Detected file type
    pub file_type: FileType,
    /// Decompressed file size in bytes
    pub size: u64,
}

/// Progress callback type for search operations
pub type SearchProgressCallback<'a> = &'a (dyn Fn(&SearchProgress) + Sync + Send);

/// Progress information during search operations
#[derive(Debug, Clone)]
pub struct SearchProgress {
    /// Current operation phase
    pub phase: SearchPhase,
    /// Current item number (1-indexed)
    pub current: usize,
    /// Total number of items
    pub total: usize,
    /// Current file or item being processed (if applicable)
    pub current_file: Option<String>,
}

impl SearchProgress {
    /// Create a new progress update
    #[must_use]
    pub fn new(phase: SearchPhase, current: usize, total: usize) -> Self {
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
        phase: SearchPhase,
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

/// Phase of search operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchPhase {
    /// Scanning PAK files for file listings
    ScanningPaks,
    /// Building file metadata index
    BuildingIndex,
    /// Indexing file content for full-text search
    IndexingContent,
    /// Exporting index to disk
    ExportingIndex,
    /// Importing index from disk
    ImportingIndex,
    /// Searching the index
    Searching,
    /// Operation complete
    Complete,
}

impl SearchPhase {
    /// Get a human-readable description of this phase
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ScanningPaks => "Scanning PAKs",
            Self::BuildingIndex => "Building index",
            Self::IndexingContent => "Indexing content",
            Self::ExportingIndex => "Exporting index",
            Self::ImportingIndex => "Importing index",
            Self::Searching => "Searching",
            Self::Complete => "Complete",
        }
    }
}

/// Metadata saved alongside the exported index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// Number of files in the metadata index
    pub file_count: usize,
    /// Number of PAK files indexed
    pub pak_count: usize,
    /// List of indexed PAK file paths
    pub indexed_paks: Vec<PathBuf>,
    /// Number of documents in the fulltext index
    pub fulltext_doc_count: u64,
}
