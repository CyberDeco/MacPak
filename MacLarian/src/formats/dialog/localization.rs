//! Localization cache for dialog text lookup
//!
//! Loads localization strings from BG3's language .pak files on demand
//! and caches them for efficient lookup.

use crate::formats::loca::{read_loca, parse_loca_bytes, LocaResource};
use crate::pak::PakOperations;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A cached localized string entry
#[derive(Debug, Clone)]
pub struct LocalizedEntry {
    /// The localized text content
    pub text: String,
    /// Version of the localization
    pub version: u16,
}

/// Cache for localization strings
///
/// Loads strings from language .pak files on demand and caches them
/// for efficient lookup during dialog display.
#[derive(Debug, Clone, Default)]
pub struct LocalizationCache {
    /// Cached strings indexed by handle
    strings: HashMap<String, LocalizedEntry>,
    /// Paths to loaded PAK files (to avoid re-loading)
    loaded_sources: Vec<PathBuf>,
    /// Current language code
    language: String,
}

impl LocalizationCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            loaded_sources: Vec::new(),
            language: "English".to_string(),
        }
    }

    /// Create a cache with a specific language
    pub fn with_language(language: &str) -> Self {
        Self {
            strings: HashMap::new(),
            loaded_sources: Vec::new(),
            language: language.to_string(),
        }
    }

    /// Get the current language
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Set the language (clears the cache)
    pub fn set_language(&mut self, language: &str) {
        if self.language != language {
            self.clear();
            self.language = language.to_string();
        }
    }

    /// Get the number of cached strings
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Clear all cached data
    pub fn clear(&mut self) {
        self.strings.clear();
        self.loaded_sources.clear();
    }

    /// Load localization from a .loca file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, LocalizationError> {
        let path = path.as_ref();

        // Check if already loaded
        if self.loaded_sources.iter().any(|p| p == path) {
            return Ok(0);
        }

        let resource = read_loca(path)
            .map_err(|e| LocalizationError::IoError(e.to_string()))?;

        let count = self.add_entries(&resource);
        self.loaded_sources.push(path.to_path_buf());

        Ok(count)
    }

    /// Load localization from a .loca file inside a PAK archive
    pub fn load_from_pak<P: AsRef<Path>>(&mut self, pak_path: P, internal_path: &str) -> Result<usize, LocalizationError> {
        let pak_path = pak_path.as_ref();
        let source_key = pak_path.join(internal_path);

        // Check if already loaded
        if self.loaded_sources.iter().any(|p| p == &source_key) {
            return Ok(0);
        }

        // Read the .loca file from the PAK
        let data = PakOperations::read_file_bytes(pak_path, internal_path)
            .map_err(|e| LocalizationError::PakError(e.to_string()))?;

        let resource = parse_loca_bytes(&data)
            .map_err(|e| LocalizationError::IoError(e.to_string()))?;

        let count = self.add_entries(&resource);
        self.loaded_sources.push(source_key);

        Ok(count)
    }

    /// Load all .loca files from a language PAK
    ///
    /// For BG3, the language PAK is typically at:
    /// `<GameData>/Localization/<Language>.pak`
    pub fn load_language_pak<P: AsRef<Path>>(&mut self, game_data_path: P) -> Result<usize, LocalizationError> {
        let game_data = game_data_path.as_ref();
        let pak_path = game_data
            .join("Localization")
            .join(format!("{}.pak", self.language));

        if !pak_path.exists() {
            return Err(LocalizationError::LanguageNotFound(self.language.clone()));
        }

        // List all .loca files in the PAK
        let entries = PakOperations::list(&pak_path)
            .map_err(|e| LocalizationError::PakError(e.to_string()))?;

        let loca_files: Vec<_> = entries
            .iter()
            .filter(|e| e.to_lowercase().ends_with(".loca"))
            .cloned()
            .collect();

        let mut total_count = 0;

        for loca_path in loca_files {
            match self.load_from_pak(&pak_path, &loca_path) {
                Ok(count) => total_count += count,
                Err(e) => {
                    tracing::warn!("Failed to load {}: {}", loca_path, e);
                }
            }
        }

        Ok(total_count)
    }

    /// Add entries from a LocaResource
    fn add_entries(&mut self, resource: &LocaResource) -> usize {
        let mut count = 0;
        for entry in &resource.entries {
            // Use the handle format that BG3 uses (without the 'h' prefix sometimes)
            let key = entry.key.clone();

            self.strings.insert(key.clone(), LocalizedEntry {
                text: entry.text.clone(),
                version: entry.version,
            });

            // Also insert with 'h' prefix if not present, or without if present
            // to handle different handle formats
            if key.starts_with('h') {
                self.strings.insert(key[1..].to_string(), LocalizedEntry {
                    text: entry.text.clone(),
                    version: entry.version,
                });
            } else {
                self.strings.insert(format!("h{}", key), LocalizedEntry {
                    text: entry.text.clone(),
                    version: entry.version,
                });
            }

            count += 1;
        }
        count
    }

    /// Look up a localized string by handle
    pub fn get(&self, handle: &str) -> Option<&LocalizedEntry> {
        self.strings.get(handle)
            .or_else(|| {
                // Try with/without 'h' prefix
                if handle.starts_with('h') {
                    self.strings.get(&handle[1..])
                } else {
                    self.strings.get(&format!("h{}", handle))
                }
            })
    }

    /// Get text for a handle, returning a placeholder if not found
    pub fn get_text(&self, handle: &str) -> String {
        self.get(handle)
            .map(|e| e.text.clone())
            .unwrap_or_else(|| format!("[{}]", handle))
    }

    /// Get text or None if not found
    pub fn get_text_opt(&self, handle: &str) -> Option<String> {
        self.get(handle).map(|e| e.text.clone())
    }

    /// Check if a handle exists in the cache
    pub fn contains(&self, handle: &str) -> bool {
        self.get(handle).is_some()
    }

    /// Manually insert a localization entry
    pub fn insert(&mut self, handle: String, text: String, version: u16) {
        self.strings.insert(handle, LocalizedEntry { text, version });
    }
}

/// Error type for localization operations
#[derive(Debug)]
pub enum LocalizationError {
    IoError(String),
    PakError(String),
    LanguageNotFound(String),
    ParseError(String),
}

impl std::fmt::Display for LocalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalizationError::IoError(e) => write!(f, "IO error: {}", e),
            LocalizationError::PakError(e) => write!(f, "PAK error: {}", e),
            LocalizationError::LanguageNotFound(lang) => write!(f, "Language not found: {}", lang),
            LocalizationError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for LocalizationError {}

/// Get available languages from a BG3 data directory
pub fn get_available_languages<P: AsRef<Path>>(game_data_path: P) -> Vec<String> {
    let localization_dir = game_data_path.as_ref().join("Localization");

    let mut languages = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&localization_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "pak").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    // Filter out non-language paks (Voice, VoiceMeta)
                    if !name.contains("Voice") && !name.contains("Meta") {
                        languages.push(name);
                    }
                }
            }
        }
    }

    // Sort alphabetically, but put English first
    languages.sort_by(|a, b| {
        if a == "English" {
            std::cmp::Ordering::Less
        } else if b == "English" {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });

    languages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = LocalizationCache::new();
        assert!(cache.is_empty());

        cache.insert("h12345".to_string(), "Hello, world!".to_string(), 1);
        assert_eq!(cache.len(), 1);

        assert_eq!(cache.get_text("h12345"), "Hello, world!");
        assert_eq!(cache.get_text("unknown"), "[unknown]");
    }

    #[test]
    fn test_handle_prefix_normalization() {
        let mut cache = LocalizationCache::new();

        cache.insert("h12345".to_string(), "Test".to_string(), 1);

        // Should find with or without 'h' prefix
        assert!(cache.contains("h12345"));
        assert!(cache.contains("12345"));
    }

    #[test]
    fn test_language_change_clears_cache() {
        let mut cache = LocalizationCache::with_language("English");
        cache.insert("h12345".to_string(), "Hello".to_string(), 1);

        assert!(!cache.is_empty());

        cache.set_language("German");

        assert!(cache.is_empty());
        assert_eq!(cache.language(), "German");
    }
}
