//! Search methods for `SearchIndex`

use super::fulltext::FullTextResult;
use super::types::{FileType, IndexedFile, SearchProgressCallback};
use super::SearchIndex;

impl SearchIndex {
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
    pub fn search_fulltext_with_progress(
        &self,
        query: &str,
        limit: usize,
        progress: SearchProgressCallback,
    ) -> Option<Vec<FullTextResult>> {
        self.fulltext.as_ref().and_then(|ft| ft.search_with_progress(query, limit, progress).ok())
    }

    /// Get number of documents in fulltext index
    #[must_use]
    pub fn fulltext_doc_count(&self) -> u64 {
        self.fulltext.as_ref().map_or(0, super::fulltext::FullTextIndex::num_docs)
    }
}
