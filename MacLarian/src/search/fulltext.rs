//! Full-text search using Tantivy
//!
//! Provides instant deep search by pre-indexing file content during "Build Index".

use std::path::{Path, PathBuf};

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, Value, STORED, STRING, TEXT};
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument};

use crate::error::{Error, Result};

/// Full-text search index using Tantivy with in-memory storage.
///
/// Built during "Build Index" by extracting text from LSF/LSX/LSJ files.
/// Supports phrase queries, fuzzy matching, boolean operators, and BM25 ranking.
pub struct FullTextIndex {
    index: Index,
    reader: IndexReader,
    path_field: Field,
    name_field: Field,
    content_field: Field,
    pak_field: Field,
    file_type_field: Field,
}

/// A search result from the full-text index
#[derive(Debug, Clone)]
pub struct FullTextResult {
    /// Full path within PAK
    pub path: String,
    /// Filename only
    pub name: String,
    /// Source PAK file
    pub pak_file: PathBuf,
    /// File type (e.g., "lsf", "lsx")
    pub file_type: String,
    /// Relevance score (higher = more relevant)
    pub score: f32,
    /// Snippet showing match context
    pub snippet: Option<String>,
    /// Number of matches in the file (capped at 99)
    pub match_count: usize,
}

impl FullTextIndex {
    /// Create a new in-memory full-text index
    ///
    /// # Errors
    /// Returns an error if index creation fails.
    pub fn new() -> Result<Self> {
        Self::create_internal(None)
    }

    /// Create a new full-text index in a directory (for persistence)
    ///
    /// # Errors
    /// Returns an error if index creation fails.
    pub fn create_in_dir(dir: &Path) -> Result<Self> {
        Self::create_internal(Some(dir))
    }

    /// Open an existing full-text index from a directory
    ///
    /// # Errors
    /// Returns an error if the index cannot be opened.
    pub fn open_from_dir(dir: &Path) -> Result<Self> {
        let index = Index::open_in_dir(dir)
            .map_err(|e| Error::SearchError(format!("Failed to open index: {e}")))?;

        let schema = index.schema();

        // Extract field handles from schema
        let path_field = schema
            .get_field("path")
            .map_err(|_| Error::SearchError("Missing path field in index".to_string()))?;
        let name_field = schema
            .get_field("name")
            .map_err(|_| Error::SearchError("Missing name field in index".to_string()))?;
        let content_field = schema
            .get_field("content")
            .map_err(|_| Error::SearchError("Missing content field in index".to_string()))?;
        let pak_field = schema
            .get_field("pak")
            .map_err(|_| Error::SearchError("Missing pak field in index".to_string()))?;
        let file_type_field = schema
            .get_field("file_type")
            .map_err(|_| Error::SearchError("Missing file_type field in index".to_string()))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e| Error::SearchError(format!("Failed to create reader: {e}")))?;

        Ok(Self {
            index,
            reader,
            path_field,
            name_field,
            content_field,
            pak_field,
            file_type_field,
        })
    }

    /// Internal helper to create index with optional directory
    fn create_internal(dir: Option<&Path>) -> Result<Self> {
        let mut schema_builder = Schema::builder();

        // Path is stored exactly (for retrieval) but not tokenized
        let path_field = schema_builder.add_text_field("path", STRING | STORED);
        // Name is tokenized for searching and stored
        let name_field = schema_builder.add_text_field("name", TEXT | STORED);
        // Content is tokenized and stored (needed for snippet generation)
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        // PAK file path stored for retrieval
        let pak_field = schema_builder.add_text_field("pak", STRING | STORED);
        // File type stored for retrieval
        let file_type_field = schema_builder.add_text_field("file_type", STRING | STORED);

        let schema = schema_builder.build();

        // Create index in RAM or on disk
        let index = match dir {
            Some(path) => {
                std::fs::create_dir_all(path)?;
                Index::create_in_dir(path, schema.clone())
                    .map_err(|e| Error::SearchError(format!("Failed to create index in dir: {e}")))?
            }
            None => Index::create_in_ram(schema.clone()),
        };

        // Create reader with automatic reload
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e| Error::SearchError(format!("Failed to create reader: {e}")))?;

        Ok(Self {
            index,
            reader,
            path_field,
            name_field,
            content_field,
            pak_field,
            file_type_field,
        })
    }

    /// Get an index writer for adding documents
    ///
    /// Call `commit()` on the writer when done, then call `reload()` on this index.
    ///
    /// # Errors
    /// Returns an error if the writer cannot be created.
    pub fn writer(&self, heap_size: usize) -> Result<IndexWriter> {
        self.index
            .writer(heap_size)
            .map_err(|e| Error::SearchError(format!("Failed to create writer: {e}")))
    }

    /// Add a document to the index
    ///
    /// Must be called with a writer obtained from `writer()`.
    ///
    /// # Errors
    /// Returns an error if adding the document fails.
    pub fn add_document(
        &self,
        writer: &IndexWriter,
        path: &str,
        name: &str,
        content: &str,
        pak_file: &str,
        file_type: &str,
    ) -> Result<()> {
        let doc = doc!(
            self.path_field => path,
            self.name_field => name,
            self.content_field => content,
            self.pak_field => pak_file,
            self.file_type_field => file_type,
        );

        writer
            .add_document(doc)
            .map_err(|e| Error::SearchError(format!("Failed to add document: {e}")))?;

        Ok(())
    }

    /// Reload the reader after committing writes
    ///
    /// # Errors
    /// Returns an error if the reader cannot be reloaded.
    pub fn reload(&self) -> Result<()> {
        self.reader
            .reload()
            .map_err(|e| Error::SearchError(format!("Failed to reload: {e}")))
    }

    /// Search the index with a query string
    ///
    /// Supports:
    /// - Simple terms: `barbarian`
    /// - Phrases: `"Action_Shove"`
    /// - Fuzzy: `barbrian~1`
    /// - Boolean: `class AND barbarian`, `wizard OR sorcerer`
    ///
    /// # Errors
    /// Returns an error if the search fails.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<FullTextResult>> {
        self.search_with_progress(query, limit, |_, _, _| {})
    }

    /// Search with progress callback: (current, total, filename)
    ///
    /// # Errors
    /// Returns an error if the query is invalid or the search fails.
    pub fn search_with_progress<F>(&self, query: &str, limit: usize, progress: F) -> Result<Vec<FullTextResult>>
    where
        F: Fn(usize, usize, &str),
    {
        let searcher = self.reader.searcher();

        // Create query parser that searches name and content fields
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.name_field, self.content_field]);

        progress(0, 1, "Parsing query...");
        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| Error::SearchError(format!("Invalid query: {e}")))?;

        progress(0, 1, "Searching index...");
        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(limit))
            .map_err(|e| Error::SearchError(format!("Search failed: {e}")))?;

        let total = top_docs.len();
        progress(0, total, "Processing results...");

        // Extract search terms from the query for custom snippet generation
        let search_terms = extract_search_terms(query);

        let mut results = Vec::with_capacity(total);

        for (i, (score, doc_address)) in top_docs.into_iter().enumerate() {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| Error::SearchError(format!("Failed to retrieve doc: {e}")))?;

            let path = doc
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let name = doc
                .get_first(self.name_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let pak = doc
                .get_first(self.pak_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let file_type = doc
                .get_first(self.file_type_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Get content and generate snippet with match count using custom finder
            let content = doc
                .get_first(self.content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let (snippet_text, match_count) = find_first_match_and_count(content, &search_terms, 150);

            // Report progress every 50 docs
            if i % 50 == 0 {
                progress(i, total, &name);
            }

            results.push(FullTextResult {
                path,
                name,
                pak_file: PathBuf::from(pak),
                file_type,
                score,
                snippet: snippet_text,
                match_count,
            });
        }

        progress(total, total, "Complete");
        Ok(results)
    }

    /// Get the number of documents in the index
    #[must_use] 
    pub fn num_docs(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// Get access to the searcher for iteration
    #[must_use] 
    pub fn searcher(&self) -> tantivy::Searcher {
        self.reader.searcher()
    }
}

impl Default for FullTextIndex {
    fn default() -> Self {
        Self::new().expect("Failed to create default FullTextIndex")
    }
}

/// Extract search terms from a query string, skipping boolean operators
fn extract_search_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .filter(|w| !matches!(w.to_uppercase().as_str(), "AND" | "OR" | "NOT"))
        .filter(|w| w.len() >= 2)
        .map(|w| {
            // Trim non-alphanumeric characters from start/end (quotes, parens, etc.)
            w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// Find the first match and count lines containing matches
///
/// Returns (snippet around first match, number of lines with matches capped at 99)
fn find_first_match_and_count(
    content: &str,
    terms: &[String],
    max_snippet_len: usize,
) -> (Option<String>, usize) {
    if terms.is_empty() || content.is_empty() {
        return (None, 0);
    }

    let content_lower = content.to_lowercase();

    // Find the first term that matches
    for term in terms {
        if let Some(first_pos) = content_lower.find(term) {
            // Count lines containing the term (matches "Show All Matches" behavior)
            let count = content_lower
                .lines()
                .filter(|line| line.contains(term.as_str()))
                .count();

            // Extract snippet around first match
            let snippet = extract_snippet_around(content, first_pos, term.len(), max_snippet_len);

            return (Some(snippet), count);
        }
    }

    (None, 0)
}

/// Extract a snippet centered around a match position
fn extract_snippet_around(
    content: &str,
    match_pos: usize,
    match_len: usize,
    max_len: usize,
) -> String {
    // Work with bytes for position calculations, but respect char boundaries
    let bytes = content.as_bytes();
    let content_len = bytes.len();

    if content_len <= max_len {
        // Content fits entirely, just collapse whitespace
        return collapse_to_single_line(content);
    }

    // Calculate context window around the match
    let half_context = (max_len.saturating_sub(match_len)) / 2;

    // Determine start position (expand back from match)
    let mut start = match_pos.saturating_sub(half_context);

    // Determine end position (expand forward from match end)
    let match_end = match_pos + match_len;
    let mut end = (match_end + half_context).min(content_len);

    // Adjust to char boundaries
    while start > 0 && !content.is_char_boundary(start) {
        start -= 1;
    }
    while end < content_len && !content.is_char_boundary(end) {
        end += 1;
    }

    // Try to start at a word boundary (look for whitespace)
    let snippet_start = if start > 0 {
        // Find nearest whitespace before start
        let search_range = &content[start.saturating_sub(20)..start];
        if let Some(ws_pos) = search_range.rfind(|c: char| c.is_whitespace()) {
            start.saturating_sub(20) + ws_pos + 1
        } else {
            start
        }
    } else {
        0
    };

    // Try to end at a word boundary
    let snippet_end = if end < content_len {
        // Find nearest whitespace after end
        let search_end = (end + 20).min(content_len);
        while !content.is_char_boundary(search_end.min(content_len)) && search_end > end {
            // This shouldn't happen often, but be safe
        }
        let search_range = &content[end..search_end.min(content_len)];
        if let Some(ws_pos) = search_range.find(|c: char| c.is_whitespace()) {
            end + ws_pos
        } else {
            end
        }
    } else {
        content_len
    };

    // Build the snippet
    let mut snippet = String::with_capacity(max_len + 10);

    if snippet_start > 0 {
        snippet.push_str("...");
    }

    snippet.push_str(&collapse_to_single_line(&content[snippet_start..snippet_end]));

    if snippet_end < content_len {
        snippet.push_str("...");
    }

    snippet
}

/// Collapse whitespace and newlines to single spaces
fn collapse_to_single_line(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_space = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}
