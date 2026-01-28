//! Index building functionality for `SearchIndex`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::error::Result;
use crate::pak::PakReaderCache;
use crate::pak::lspk::LspkReader;

use super::SearchIndex;
use super::extract;
use super::fulltext::FullTextIndex;
use super::types::{FileType, IndexedFile, SearchPhase, SearchProgress, SearchProgressCallback};

impl SearchIndex {
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
    pub(super) fn index_single_pak(pak_path: &Path) -> Result<Vec<IndexedFile>> {
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
                let name = e
                    .path
                    .file_name()
                    .map_or_else(|| path_str.clone(), |n| n.to_string_lossy().to_string());

                let ext = e
                    .path
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
            by_pak.entry(file.pak_file.clone()).or_default().push(file);
        }

        // Get a writer with 500MB heap (larger = fewer internal commits)
        let mut writer = fulltext.writer(500_000_000)?;
        let mut indexed_count = 0;

        // Process each PAK using bulk reading (sorted by offset, parallel decompress)
        for (pak_path, files) in &by_pak {
            let pak_name = pak_path.file_name().map_or_else(
                || "Unknown".to_string(),
                |n| n.to_string_lossy().to_string(),
            );

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
            let bulk_bytes = cache
                .read_files_bulk(pak_path, &file_paths)
                .unwrap_or_default();

            // Build list of (file, bytes) pairs
            let file_bytes: Vec<(&IndexedFile, &Vec<u8>)> = files
                .iter()
                .filter_map(|file| bulk_bytes.get(&file.path).map(|bytes| (*file, bytes)))
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

        progress(&SearchProgress::new(
            SearchPhase::Complete,
            total_files,
            total_files,
        ));

        self.fulltext = Some(fulltext);
        Ok(indexed_count)
    }
}
