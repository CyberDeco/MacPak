//! Index export and import functionality for `SearchIndex`

use std::collections::HashMap;
use std::path::Path;

use rayon::prelude::*;

use maclarian::error::Result;

use super::SearchIndex;
use super::fulltext::FullTextIndex;
use super::types::{
    IndexMetadata, IndexedFile, SearchPhase, SearchProgress, SearchProgressCallback,
};

impl SearchIndex {
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
    pub fn export_index_with_progress(
        &self,
        dir: &Path,
        progress: SearchProgressCallback,
    ) -> Result<()> {
        use tantivy::TantivyDocument;

        // Checks for fulltext index
        if self.fulltext.is_none() {
            return Err(maclarian::error::Error::SearchError(
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
        let entries_json = serde_json::to_string(&self.entries).map_err(|e| {
            maclarian::error::Error::SearchError(format!("Failed to serialize entries: {e}"))
        })?;
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
        let meta_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
            maclarian::error::Error::SearchError(format!("Failed to serialize metadata: {e}"))
        })?;
        std::fs::write(&meta_path, meta_json)?;

        // Create a new index in the directory with larger heap for faster writes
        let ft = self.fulltext.as_ref().unwrap();
        let tantivy_dir = dir.join("tantivy");

        // Remove existing tantivy index if present (for rebuild/overwrite)
        if tantivy_dir.exists() {
            std::fs::remove_dir_all(&tantivy_dir)?;
        }

        let dest_index = FullTextIndex::create_in_dir(&tantivy_dir)?;
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
            writer.add_document(doc).map_err(|e| {
                maclarian::error::Error::SearchError(format!("Failed to copy doc: {e}"))
            })?;
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
        writer.commit().map_err(|e| {
            maclarian::error::Error::SearchError(format!("Export commit failed: {e}"))
        })?;

        progress(&SearchProgress::new(
            SearchPhase::Complete,
            total_docs,
            total_docs,
        ));
        tracing::info!(
            "Exported index to {} ({} docs)",
            dir.display(),
            metadata.fulltext_doc_count
        );
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
        let metadata: IndexMetadata = serde_json::from_str(&meta_json).map_err(|e| {
            maclarian::error::Error::SearchError(format!("Failed to parse metadata: {e}"))
        })?;

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
            serde_json::from_str(&entries_json).map_err(|e| {
                maclarian::error::Error::SearchError(format!("Failed to parse entries: {e}"))
            })?
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

        // Reader must be reloaded to see segments from disk with Manual reload policy
        fulltext.reload()?;

        // Update state
        self.entries = entries;
        self.filename_index = filename_index;
        self.fulltext = Some(fulltext);
        self.file_count = metadata.file_count;
        self.indexed_paks = metadata.indexed_paks;
        self.indexed = true;

        progress(&SearchProgress::new(SearchPhase::Complete, 4, 4));

        tracing::info!(
            "Imported index from {} ({} files from {} paks)",
            dir.display(),
            self.entries.len(),
            metadata.pak_count
        );

        Ok(())
    }
}
