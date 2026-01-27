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

pub mod content_cache;
pub(crate) mod extract;
mod fulltext;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::pak::lspk::LspkReader;
use crate::pak::PakReaderCache;

// Internal use within search module
use fulltext::FullTextIndex;

// Public exports
pub use fulltext::FullTextResult;
pub use content_cache::ContentCache;

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
        matches!(self, FileType::Lsx | FileType::Lsf | FileType::Lsj | FileType::Xml | FileType::Json)
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
    pub fn with_file(phase: SearchPhase, current: usize, total: usize, file: impl Into<String>) -> Self {
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

/// Search index for PAK file contents
///
/// Builds an in-memory index of file metadata from PAK archives.
/// Supports fast O(1) filename lookups and filtered searches.
/// Optionally includes a full-text index for instant content search.
#[derive(Default)]
pub struct SearchIndex {
    /// All file entries, keyed by full internal path
    entries: HashMap<String, IndexedFile>,
    /// Reverse index: lowercase filename -> list of full paths
    filename_index: HashMap<String, Vec<String>>,
    /// Source PAK files that have been indexed
    indexed_paks: Vec<PathBuf>,
    /// Whether the index has been built
    indexed: bool,
    /// Total file count
    file_count: usize,
    /// Full-text search index (built separately via `build_fulltext_index`)
    fulltext: Option<FullTextIndex>,
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

    /// Build index from multiple PAK files
    ///
    /// Scans each PAK file in parallel to extract file metadata.
    /// Returns the total number of files indexed.
    ///
    /// # Errors
    /// Returns an error if any PAK file cannot be read.
    pub fn build_index(&mut self, pak_paths: &[PathBuf]) -> Result<usize> {
        self.build_index_with_progress(pak_paths, &|_| {})
    }

    /// Build the search index from multiple PAK files with progress callback
    ///
    /// Scans each PAK file in parallel to extract file metadata.
    /// Returns the total number of files indexed.
    ///
    /// # Errors
    /// Returns an error if any PAK file cannot be read.
    pub fn build_index_with_progress(
        &mut self,
        pak_paths: &[PathBuf],
        progress: SearchProgressCallback,
    ) -> Result<usize> {
        self.clear();

        let start = std::time::Instant::now();
        let total = pak_paths.len();

        progress(&SearchProgress::with_file(
            SearchPhase::ScanningPaks,
            0,
            total,
            "Starting PAK scan",
        ));

        // Process each PAK file in parallel
        let pak_entries: Vec<Result<Vec<IndexedFile>>> = pak_paths
            .par_iter()
            .map(|pak_path| Self::index_single_pak(pak_path))
            .collect();

        // Merge results sequentially (to avoid lock contention)
        for (i, (pak_path, result)) in pak_paths.iter().zip(pak_entries).enumerate() {
            progress(&SearchProgress::with_file(
                SearchPhase::BuildingIndex,
                i + 1,
                total,
                pak_path.display().to_string(),
            ));

            match result {
                Ok(entries) => {
                    for entry in entries {
                        let path_key = entry.path.clone();
                        let filename_key = entry.name.to_lowercase();

                        // Add to filename index
                        self.filename_index
                            .entry(filename_key)
                            .or_default()
                            .push(path_key.clone());

                        // Add to main entries
                        self.entries.insert(path_key, entry);
                    }
                    self.indexed_paks.push(pak_path.clone());
                }
                Err(e) => {
                    tracing::warn!("Failed to index {}: {}", pak_path.display(), e);
                }
            }
        }

        self.file_count = self.entries.len();
        self.indexed = true;

        progress(&SearchProgress::new(SearchPhase::Complete, total, total));

        let elapsed = start.elapsed();
        tracing::info!(
            "Indexed {} files from {} PAKs in {:.2}s",
            self.file_count,
            self.indexed_paks.len(),
            elapsed.as_secs_f64()
        );

        Ok(self.file_count)
    }

    /// Index a single PAK file
    fn index_single_pak(pak_path: &Path) -> Result<Vec<IndexedFile>> {
        let file = std::fs::File::open(pak_path)?;
        let mut reader = LspkReader::with_path(file, pak_path);

        let entries = reader.list_files()?;

        let file_entries: Vec<IndexedFile> = entries
            .into_iter()
            .filter(|e| {
                // Skip .DS_Store and other junk
                e.path.file_name() != Some(std::ffi::OsStr::new(".DS_Store"))
            })
            .map(|e| {
                let path_str = e.path.to_string_lossy().to_string();
                let name = e.path
                    .file_name().map_or_else(|| path_str.clone(), |n| n.to_string_lossy().to_string());

                let ext = e.path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();

                IndexedFile {
                    name,
                    path: path_str,
                    pak_file: pak_path.to_path_buf(),
                    file_type: FileType::from_extension(&ext),
                    size: u64::from(e.size_decompressed),
                }
            })
            .collect();

        Ok(file_entries)
    }

    /// Search for files by filename (case-insensitive)
    ///
    /// Returns entries where the filename contains the query string.
    /// Optionally filter by file type.
    #[must_use] 
    pub fn search_filename(&self, query: &str, filter: Option<FileType>) -> Vec<&IndexedFile> {
        let query_lower = query.to_lowercase();

        self.filename_index
            .iter()
            .filter(|(filename, _)| filename.contains(&query_lower))
            .flat_map(|(_, paths)| paths.iter())
            .filter_map(|path| self.entries.get(path))
            .filter(|entry| {
                filter.is_none_or(|f| entry.file_type == f)
            })
            .collect()
    }

    /// Search for files by path (case-insensitive substring match)
    ///
    /// Returns entries where the full path contains the query string.
    #[must_use] 
    pub fn search_path(&self, query: &str, filter: Option<FileType>) -> Vec<&IndexedFile> {
        let query_lower = query.to_lowercase();

        self.entries
            .values()
            .filter(|entry| entry.path.to_lowercase().contains(&query_lower))
            .filter(|entry| filter.is_none_or(|f| entry.file_type == f))
            .collect()
    }

    /// Search for UUIDs in filenames/paths
    ///
    /// Handles various UUID formats (with/without hyphens, with h/g prefix).
    #[must_use] 
    pub fn search_uuid(&self, uuid: &str) -> Vec<&IndexedFile> {
        // Normalize UUID: remove hyphens, convert to lowercase
        let normalized: String = uuid
            .chars()
            .filter(char::is_ascii_hexdigit)
            .collect::<String>()
            .to_lowercase();

        if normalized.len() < 8 {
            return Vec::new(); // Too short to be meaningful
        }

        self.entries
            .values()
            .filter(|entry| {
                let path_normalized: String = entry.path
                    .chars()
                    .filter(char::is_ascii_hexdigit)
                    .collect::<String>()
                    .to_lowercase();
                path_normalized.contains(&normalized)
            })
            .collect()
    }

    /// Get a file entry by its full path
    #[must_use] 
    pub fn get_by_path(&self, path: &str) -> Option<&IndexedFile> {
        self.entries.get(path)
    }

    /// Get all entries (for iteration)
    pub fn all_entries(&self) -> impl Iterator<Item = &IndexedFile> {
        self.entries.values()
    }

    /// Get entries filtered by file type
    #[must_use] 
    pub fn entries_by_type(&self, file_type: FileType) -> Vec<&IndexedFile> {
        self.entries
            .values()
            .filter(|e| e.file_type == file_type)
            .collect()
    }

    /// Build full-text index from file contents
    ///
    /// Extracts text from all searchable files and indexes them for instant search.
    /// This is a potentially long operation - use the progress callback to track progress.
    ///
    /// Must be called after `build_index()` has been run.
    ///
    /// # Errors
    /// Returns an error if file extraction or indexing fails.
    pub fn build_fulltext_index(&mut self, progress: SearchProgressCallback) -> Result<usize> {
        if !self.indexed {
            return Ok(0);
        }

        // Create new fulltext index
        let fulltext = FullTextIndex::new()?;

        // Collect searchable files (skip tiny files < 100 bytes)
        let searchable_files: Vec<&IndexedFile> = self
            .entries
            .values()
            .filter(|f| f.file_type.is_searchable_text())
            .filter(|f| f.size >= 100) // Skip tiny files
            .collect();

        let total_files = searchable_files.len();
        progress(&SearchProgress::with_file(
            SearchPhase::IndexingContent,
            0,
            total_files,
            "Starting content indexing...",
        ));

        // Group files by PAK for efficient reading
        let mut by_pak: HashMap<PathBuf, Vec<&IndexedFile>> = HashMap::new();
        for file in &searchable_files {
            by_pak
                .entry(file.pak_file.clone())
                .or_default()
                .push(file);
        }

        // Get a writer with 500MB heap (larger = fewer internal commits)
        let mut writer = fulltext.writer(500_000_000)?;
        let mut indexed_count = 0;

        // Process each PAK using bulk reading (sorted by offset, parallel decompress)
        for (pak_path, files) in &by_pak {
            let pak_name = pak_path
                .file_name().map_or_else(|| "Unknown".to_string(), |n| n.to_string_lossy().to_string());

            progress(&SearchProgress::with_file(
                SearchPhase::IndexingContent,
                indexed_count,
                total_files,
                &pak_name,
            ));

            // Collect all file paths for bulk reading
            let file_paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();

            // Create cache and do bulk read (sorted by offset, parallel decompress)
            let mut cache = PakReaderCache::new(1);
            let bulk_bytes = cache.read_files_bulk(pak_path, &file_paths).unwrap_or_default();

            // Build list of (file, bytes) pairs
            let file_bytes: Vec<(&IndexedFile, &Vec<u8>)> = files
                .iter()
                .filter_map(|file| {
                    bulk_bytes.get(&file.path).map(|bytes| (*file, bytes))
                })
                .collect();

            // Extract text in parallel (CPU bound)
            let extracted: Vec<(&IndexedFile, String)> = file_bytes
                .par_iter()
                .map(|(file, bytes)| {
                    let text = extract::extract_text(bytes, file.file_type);
                    (*file, text)
                })
                .collect();

            // Add to Tantivy (single-threaded writer)
            for (file, text) in extracted {
                if text.is_empty() {
                    continue;
                }

                indexed_count += 1;
                if indexed_count % 1000 == 0 {
                    progress(&SearchProgress::with_file(
                        SearchPhase::IndexingContent,
                        indexed_count,
                        total_files,
                        &pak_name,
                    ));
                }

                fulltext.add_document(
                    &writer,
                    &file.path,
                    &file.name,
                    &text,
                    &pak_path.to_string_lossy(),
                    file.file_type.display_name(),
                )?;
            }
        }

        // Commit and reload
        writer
            .commit()
            .map_err(|e| crate::error::Error::SearchError(format!("Commit failed: {e}")))?;
        fulltext.reload()?;

        tracing::info!(
            "Built fulltext index for {} files ({} docs)",
            indexed_count,
            fulltext.num_docs()
        );

        progress(&SearchProgress::new(SearchPhase::Complete, total_files, total_files));

        self.fulltext = Some(fulltext);
        Ok(indexed_count)
    }

    /// Search using the full-text index
    ///
    /// Returns results ranked by relevance (BM25).
    /// Supports phrase queries, fuzzy matching, and boolean operators.
    ///
    /// Returns None if fulltext index hasn't been built.
    #[must_use] 
    pub fn search_fulltext(&self, query: &str, limit: usize) -> Option<Vec<FullTextResult>> {
        self.fulltext.as_ref().and_then(|ft| ft.search(query, limit).ok())
    }

    /// Search fulltext index with progress callback
    pub fn search_fulltext_with_progress(&self, query: &str, limit: usize, progress: SearchProgressCallback) -> Option<Vec<FullTextResult>>
    {
        self.fulltext.as_ref().and_then(|ft| ft.search_with_progress(query, limit, progress).ok())
    }

    /// Get number of documents in fulltext index
    #[must_use] 
    pub fn fulltext_doc_count(&self) -> u64 {
        self.fulltext.as_ref().map_or(0, fulltext::FullTextIndex::num_docs)
    }

    /// Export the fulltext index to a directory
    ///
    /// Saves the Tantivy index and metadata (file count, pak list) for later import.
    /// Returns an error if no fulltext index has been built.
    ///
    /// # Errors
    /// Returns an error if writing the index fails.
    pub fn export_index(&self, dir: &Path) -> Result<()> {
        self.export_index_with_progress(dir, &|_| {})
    }

    /// Export the fulltext index with progress callback
    ///
    /// # Errors
    /// Returns an error if no fulltext index exists or writing fails.
    ///
    /// # Panics
    /// This function does not panic under normal conditions.
    pub fn export_index_with_progress(&self, dir: &Path, progress: SearchProgressCallback) -> Result<()>
    {
        use tantivy::TantivyDocument;

        // Checks for fulltext index
        if self.fulltext.is_none() {
            return Err(crate::error::Error::SearchError(
                "No fulltext index to export".to_string(),
            ));
        }

        progress(&SearchProgress::with_file(
            SearchPhase::ExportingIndex,
            0,
            1,
            "Creating export directory...",
        ));

        // Create the export directory
        std::fs::create_dir_all(dir)?;

        // Save file entries (needed for filename/path search)
        progress(&SearchProgress::with_file(
            SearchPhase::ExportingIndex,
            0,
            1,
            "Saving file entries...",
        ));
        let entries_path = dir.join("entries.json");
        let entries_json = serde_json::to_string(&self.entries)
            .map_err(|e| crate::error::Error::SearchError(format!("Failed to serialize entries: {e}")))?;
        std::fs::write(&entries_path, entries_json)?;

        // Save metadata
        progress(&SearchProgress::with_file(
            SearchPhase::ExportingIndex,
            0,
            1,
            "Saving metadata...",
        ));
        let metadata = IndexMetadata {
            file_count: self.file_count,
            pak_count: self.indexed_paks.len(),
            indexed_paks: self.indexed_paks.clone(),
            fulltext_doc_count: self.fulltext_doc_count(),
        };
        let meta_path = dir.join("metadata.json");
        let meta_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| crate::error::Error::SearchError(format!("Failed to serialize metadata: {e}")))?;
        std::fs::write(&meta_path, meta_json)?;

        // Create a new index in the directory with larger heap for faster writes
        let ft = self.fulltext.as_ref().unwrap();
        let dest_index = FullTextIndex::create_in_dir(&dir.join("tantivy"))?;
        let mut writer = dest_index.writer(500_000_000)?; // 500MB heap for faster batching

        // Count total documents for progress
        let total_docs = ft.num_docs() as usize;
        progress(&SearchProgress::with_file(
            SearchPhase::ExportingIndex,
            0,
            total_docs,
            "Reading documents...",
        ));

        // Read all documents from segments in parallel using rayon
        let searcher = ft.searcher();
        let segment_readers: Vec<_> = searcher.segment_readers().iter().collect();

        // Parallel read: each segment is processed independently
        let all_docs: Vec<TantivyDocument> = segment_readers
            .par_iter()
            .flat_map(|segment_reader| {
                let Ok(store_reader) = segment_reader.get_store_reader(16) else {
                    return Vec::new(); // Larger cache
                };
                (0..segment_reader.num_docs())
                    .filter_map(|doc_id| store_reader.get(doc_id).ok())
                    .collect::<Vec<_>>()
            })
            .collect();

        progress(&SearchProgress::with_file(
            SearchPhase::ExportingIndex,
            all_docs.len(),
            total_docs,
            "Writing documents...",
        ));

        // Sequential write (IndexWriter is not thread-safe for concurrent adds)
        for (i, doc) in all_docs.into_iter().enumerate() {
            writer.add_document(doc)
                .map_err(|e| crate::error::Error::SearchError(format!("Failed to copy doc: {e}")))?;
            if i % 5000 == 0 {
                progress(&SearchProgress::with_file(
                    SearchPhase::ExportingIndex,
                    i,
                    total_docs,
                    "Writing documents...",
                ));
            }
        }

        progress(&SearchProgress::with_file(
            SearchPhase::ExportingIndex,
            total_docs,
            total_docs,
            "Committing index...",
        ));
        writer
            .commit()
            .map_err(|e| crate::error::Error::SearchError(format!("Export commit failed: {e}")))?;

        progress(&SearchProgress::new(SearchPhase::Complete, total_docs, total_docs));
        tracing::info!("Exported index to {} ({} docs)", dir.display(), metadata.fulltext_doc_count);
        Ok(())
    }

    /// Import a fulltext index from a directory
    ///
    /// Loads the Tantivy index, file entries, and metadata previously saved with `export_index`.
    ///
    /// # Errors
    /// Returns an error if reading the index fails.
    pub fn import_index(&mut self, dir: &Path) -> Result<()> {
        self.import_index_with_progress(dir, &|_| {})
    }

    /// Import a fulltext index from a directory with progress callback
    ///
    /// Loads the Tantivy index, file entries, and metadata previously saved with `export_index`.
    ///
    /// # Errors
    /// Returns an error if reading the index fails.
    pub fn import_index_with_progress(
        &mut self,
        dir: &Path,
        progress: SearchProgressCallback,
    ) -> Result<()> {
        progress(&SearchProgress::with_file(
            SearchPhase::ImportingIndex,
            0,
            4,
            "Loading metadata",
        ));

        // Load metadata
        let meta_path = dir.join("metadata.json");
        let meta_json = std::fs::read_to_string(&meta_path)?;
        let metadata: IndexMetadata = serde_json::from_str(&meta_json)
            .map_err(|e| crate::error::Error::SearchError(format!("Failed to parse metadata: {e}")))?;

        progress(&SearchProgress::with_file(
            SearchPhase::ImportingIndex,
            1,
            4,
            "Loading file entries",
        ));

        // Load file entries (if available - for backward compatibility)
        let entries_path = dir.join("entries.json");
        let entries: HashMap<String, IndexedFile> = if entries_path.exists() {
            let entries_json = std::fs::read_to_string(&entries_path)?;
            serde_json::from_str(&entries_json)
                .map_err(|e| crate::error::Error::SearchError(format!("Failed to parse entries: {e}")))?
        } else {
            HashMap::new()
        };

        progress(&SearchProgress::with_file(
            SearchPhase::ImportingIndex,
            2,
            4,
            "Rebuilding filename index",
        ));

        // Rebuild filename index from entries
        let mut filename_index: HashMap<String, Vec<String>> = HashMap::new();
        for (path, file) in &entries {
            let filename_lower = file.name.to_lowercase();
            filename_index
                .entry(filename_lower)
                .or_default()
                .push(path.clone());
        }

        progress(&SearchProgress::with_file(
            SearchPhase::ImportingIndex,
            3,
            4,
            "Opening Tantivy index",
        ));

        // Open the Tantivy index
        let tantivy_dir = dir.join("tantivy");
        let fulltext = FullTextIndex::open_from_dir(&tantivy_dir)?;

        // Update state
        self.entries = entries;
        self.filename_index = filename_index;
        self.fulltext = Some(fulltext);
        self.file_count = metadata.file_count;
        self.indexed_paks = metadata.indexed_paks;
        self.indexed = true;

        progress(&SearchProgress::new(SearchPhase::Complete, 4, 4));

        tracing::info!(
            "Imported index from {} ({} files, {} docs from {} paks)",
            dir.display(),
            self.entries.len(),
            metadata.fulltext_doc_count,
            metadata.pak_count
        );

        Ok(())
    }
}

/// Metadata saved alongside the exported index
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexMetadata {
    /// Number of files in the metadata index
    file_count: usize,
    /// Number of PAK files indexed
    pak_count: usize,
    /// List of indexed PAK file paths
    indexed_paks: Vec<PathBuf>,
    /// Number of documents in the fulltext index
    fulltext_doc_count: u64,
}
