//! Search index for PAK file contents
//!
//! Provides fast filename/path searching across PAK archives without extraction.
//! Uses a two-phase approach:
//! - Phase 1: Build file metadata index from PAK listings (fast, no extraction)
//! - Phase 2: On-demand content loading with LRU caching (for deep search)
//!
//! ## Usage
//!
//! ```ignore
//! let mut index = SearchIndex::new();
//! index.build_index(&[pak_path1, pak_path2])?;
//!
//! // Fast filename search
//! let results = index.search_filename("Barbarian", None);
//!
//! // Search with filter
//! let lsx_only = index.search_filename("Barbarian", Some(FileType::Lsx));
//! ```

#![allow(clippy::cast_possible_truncation)]

// Submodules
mod builder;
pub mod content_cache;
pub(crate) mod extract;
mod fulltext;
mod persistence;
mod search_methods;
mod types;

use std::collections::HashMap;
use std::path::PathBuf;

// Internal use within search module
use fulltext::FullTextIndex;

// Public exports
pub use content_cache::ContentCache;
pub use fulltext::FullTextResult;
pub use types::{
    FileType, IndexMetadata, IndexedFile, SearchPhase, SearchProgress, SearchProgressCallback,
};

/// Search index for PAK file contents
///
/// Builds an in-memory index of file metadata from PAK archives.
/// Supports fast O(1) filename lookups and filtered searches.
/// Optionally includes a full-text index for instant content search.
#[derive(Default)]
pub struct SearchIndex {
    /// All file entries, keyed by full internal path
    pub(crate) entries: HashMap<String, IndexedFile>,
    /// Reverse index: lowercase filename -> list of full paths
    pub(crate) filename_index: HashMap<String, Vec<String>>,
    /// Source PAK files that have been indexed
    pub(crate) indexed_paks: Vec<PathBuf>,
    /// Whether the index has been built
    pub(crate) indexed: bool,
    /// Total file count
    pub(crate) file_count: usize,
    /// Full-text search index (built separately via `build_fulltext_index`)
    pub(crate) fulltext: Option<FullTextIndex>,
}

impl SearchIndex {
    /// Create a new empty search index
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the index has been built
    #[must_use]
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Get the number of indexed files
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.file_count
    }

    /// Get the number of indexed PAK files
    #[must_use]
    pub fn pak_count(&self) -> usize {
        self.indexed_paks.len()
    }

    /// Get list of indexed PAK files
    #[must_use]
    pub fn indexed_paks(&self) -> &[PathBuf] {
        &self.indexed_paks
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.entries.clear();
        self.filename_index.clear();
        self.indexed_paks.clear();
        self.indexed = false;
        self.file_count = 0;
        self.fulltext = None;
    }

    /// Check if full-text index is available
    #[must_use]
    pub fn has_fulltext(&self) -> bool {
        self.fulltext.is_some()
    }
}
