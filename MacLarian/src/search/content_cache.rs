//! Content cache for on-demand file loading with LRU eviction
//!
//! Provides lazy loading of file contents from PAK archives with automatic
//! LSF→LSX conversion for searchable text content.
//!
//! Modeled after the `AudioCache` pattern.

#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::converter::lsf_lsx_lsj::to_lsx;
use crate::error::{Error, Result};
use crate::formats::lsf::parse_lsf_bytes;
use crate::pak::PakOperations;

use super::{IndexedFile, FileType};

/// Default maximum number of cached content entries
const DEFAULT_MAX_ENTRIES: usize = 50;

/// A cached content entry
#[derive(Debug, Clone)]
pub struct CachedContent {
    /// The text content (LSX XML for LSF files, raw text for others)
    pub text: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Original file type (before conversion)
    pub original_type: FileType,
    /// Source PAK file
    pub source_pak: PathBuf,
    /// Internal path within PAK
    pub internal_path: String,
}

/// Cache statistics for debugging
#[derive(Debug, Default, Clone)]
pub struct ContentCacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub conversions: usize,
    pub total_bytes_cached: usize,
}

/// Content match from a search
#[derive(Debug, Clone)]
pub struct ContentMatch {
    /// The file entry that matched
    pub entry: IndexedFile,
    /// Matching line numbers (1-indexed)
    pub line_numbers: Vec<usize>,
    /// Context snippets around matches
    pub snippets: Vec<String>,
}

/// Content cache with LRU eviction
///
/// Loads file contents on demand from PAK archives, converting LSF to LSX
/// automatically. Uses LRU eviction to bound memory usage.
#[derive(Debug)]
pub struct ContentCache {
    /// Cached content, keyed by "`pak_path:internal_path`"
    entries: HashMap<String, CachedContent>,
    /// Access order for LRU eviction (most recent at end)
    access_order: Vec<String>,
    /// Maximum number of entries to cache
    max_entries: usize,
    /// Cache statistics
    stats: ContentCacheStats,
}

impl ContentCache {
    /// Create a new empty cache with default settings
    #[must_use] 
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            max_entries: DEFAULT_MAX_ENTRIES,
            stats: ContentCacheStats::default(),
        }
    }

    /// Create a cache with custom max entries
    #[must_use] 
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            max_entries,
            stats: ContentCacheStats::default(),
        }
    }

    /// Generate cache key from PAK path and internal path
    fn cache_key(pak_path: &Path, internal_path: &str) -> String {
        format!("{}:{}", pak_path.display(), internal_path)
    }

    /// Check if content is cached
    #[must_use] 
    pub fn contains(&self, pak_path: &Path, internal_path: &str) -> bool {
        let key = Self::cache_key(pak_path, internal_path);
        self.entries.contains_key(&key)
    }

    /// Get cached content if available (updates access order)
    pub fn get(&mut self, pak_path: &Path, internal_path: &str) -> Option<&CachedContent> {
        let key = Self::cache_key(pak_path, internal_path);

        if self.entries.contains_key(&key) {
            self.stats.hits += 1;
            // Update access order (move to end = most recently used)
            self.access_order.retain(|k| k != &key);
            self.access_order.push(key.clone());
            self.entries.get(&key)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Get or load content from PAK
    ///
    /// If cached, returns the cached version. Otherwise loads from PAK,
    /// converts LSF→LSX if needed, caches, and returns.
    ///
    /// # Errors
    /// Returns an error if the PAK cannot be read or conversion fails.
    ///
    /// # Panics
    /// This function does not panic under normal conditions.
    pub fn get_or_load(
        &mut self,
        pak_path: &Path,
        internal_path: &str,
        file_type: FileType,
    ) -> Result<&CachedContent> {
        let key = Self::cache_key(pak_path, internal_path);

        // Check cache first
        if self.entries.contains_key(&key) {
            self.stats.hits += 1;
            self.access_order.retain(|k| k != &key);
            self.access_order.push(key.clone());
            return Ok(self.entries.get(&key).expect("key exists per contains_key check"));
        }

        self.stats.misses += 1;

        // Load from PAK
        let raw_bytes = PakOperations::read_file_bytes(pak_path, internal_path)?;

        // Convert based on file type
        let text = match file_type {
            FileType::Lsf => {
                // Convert LSF binary to LSX XML
                self.stats.conversions += 1;
                let lsf_doc = parse_lsf_bytes(&raw_bytes)?;
                to_lsx(&lsf_doc)?
            }
            FileType::Lsx | FileType::Xml => {
                // Already text, just decode
                String::from_utf8(raw_bytes)
                    .map_err(|e| Error::ConversionError(format!("UTF-8 decode error: {e}")))?
            }
            FileType::Lsj | FileType::Json => {
                // JSON is text
                String::from_utf8(raw_bytes)
                    .map_err(|e| Error::ConversionError(format!("UTF-8 decode error: {e}")))?
            }
            _ => {
                // Non-text format, try anyway but may fail
                String::from_utf8(raw_bytes)
                    .map_err(|e| Error::ConversionError(format!("Not a text file: {e}")))?
            }
        };

        let size_bytes = text.len();

        // Evict if needed
        while self.entries.len() >= self.max_entries && !self.access_order.is_empty() {
            self.evict_oldest();
        }

        // Cache the result
        let cached = CachedContent {
            text,
            size_bytes,
            original_type: file_type,
            source_pak: pak_path.to_path_buf(),
            internal_path: internal_path.to_string(),
        };

        self.stats.total_bytes_cached += size_bytes;
        self.entries.insert(key.clone(), cached);
        self.access_order.push(key.clone());

        Ok(self.entries.get(&key).expect("entry was just inserted"))
    }

    /// Search content for a query string
    ///
    /// Loads content on demand and searches for matches.
    /// Returns matches with line numbers and context snippets.
    ///
    /// # Errors
    /// Returns an error if the content cannot be loaded.
    pub fn search_content(
        &mut self,
        entry: &IndexedFile,
        query: &str,
        case_sensitive: bool,
    ) -> Result<Option<ContentMatch>> {
        // Only search text-based files
        if !entry.file_type.is_searchable_text() {
            return Ok(None);
        }

        // Load content
        let content = self.get_or_load(&entry.pak_file, &entry.path, entry.file_type)?;

        // Search for matches
        let query_normalized = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut line_numbers = Vec::new();
        let mut snippets = Vec::new();

        for (line_num, line) in content.text.lines().enumerate() {
            let line_to_search = if case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };

            if line_to_search.contains(&query_normalized) {
                line_numbers.push(line_num + 1); // 1-indexed

                // Create snippet (truncate long lines)
                let snippet = if line.len() > 200 {
                    format!("{}...", &line[..200])
                } else {
                    line.to_string()
                };
                snippets.push(snippet);
            }
        }

        if line_numbers.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ContentMatch {
                entry: entry.clone(),
                line_numbers,
                snippets,
            }))
        }
    }

    /// Evict the oldest (least recently used) entry
    fn evict_oldest(&mut self) {
        if let Some(oldest) = self.access_order.first().cloned() {
            if let Some(entry) = self.entries.remove(&oldest) {
                self.stats.total_bytes_cached = self.stats
                    .total_bytes_cached
                    .saturating_sub(entry.size_bytes);
                self.stats.evictions += 1;
            }
            self.access_order.remove(0);
        }
    }

    /// Clear all cached content
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.stats.total_bytes_cached = 0;
    }

    /// Get the number of cached entries
    #[must_use] 
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get cache statistics
    #[must_use] 
    pub fn stats(&self) -> &ContentCacheStats {
        &self.stats
    }

    /// Get total cached size in bytes
    #[must_use] 
    pub fn total_size_bytes(&self) -> usize {
        self.stats.total_bytes_cached
    }

    /// Get total cached size in megabytes
    #[must_use] 
    pub fn total_size_mb(&self) -> f32 {
        self.stats.total_bytes_cached as f32 / (1024.0 * 1024.0)
    }
}

impl Default for ContentCache {
    fn default() -> Self {
        Self::new()
    }
}

