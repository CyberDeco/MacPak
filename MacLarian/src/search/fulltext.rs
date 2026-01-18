//! Full-text search using Tantivy
//!
//! Provides instant deep search by pre-indexing file content during "Build Index".

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, Value, STORED, STRING, TEXT};
use tantivy::snippet::SnippetGenerator;
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
    /// Snippet showing match context (with <b> tags around matched terms)
    pub snippet: Option<String>,
}

impl FullTextIndex {
    /// Create a new in-memory full-text index
    pub fn new() -> Result<Self> {
        Self::create_internal(None)
    }

    /// Create a new full-text index in a directory (for persistence)
    pub fn create_in_dir(dir: &Path) -> Result<Self> {
        Self::create_internal(Some(dir))
    }

    /// Open an existing full-text index from a directory
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
    pub fn writer(&self, heap_size: usize) -> Result<IndexWriter> {
        self.index
            .writer(heap_size)
            .map_err(|e| Error::SearchError(format!("Failed to create writer: {e}")))
    }

    /// Add a document to the index
    ///
    /// Must be called with a writer obtained from `writer()`.
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
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<FullTextResult>> {
        self.search_with_progress(query, limit, |_, _, _| {})
    }

    /// Search with progress callback: (current, total, filename)
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
        progress(0, total, "Generating snippets...");

        // Create snippet generator for content field
        let snippet_generator = SnippetGenerator::create(&searcher, &parsed_query, self.content_field)
            .map_err(|e| Error::SearchError(format!("Failed to create snippet generator: {e}")))?;

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

            // Generate snippet with match context
            let snippet = snippet_generator.snippet_from_doc(&doc);
            let snippet_text = if snippet.is_empty() {
                None
            } else {
                // Convert to HTML with <b> tags around matched terms
                Some(snippet.to_html())
            };

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
            });
        }

        progress(total, total, "Complete");
        Ok(results)
    }

    /// Get the number of documents in the index
    pub fn num_docs(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// Get access to the searcher for iteration
    pub fn searcher(&self) -> tantivy::Searcher {
        self.reader.searcher()
    }
}

impl Default for FullTextIndex {
    fn default() -> Self {
        Self::new().expect("Failed to create default FullTextIndex")
    }
}

/// Thread-safe wrapper for FullTextIndex
pub type SharedFullTextIndex = Arc<FullTextIndex>;
