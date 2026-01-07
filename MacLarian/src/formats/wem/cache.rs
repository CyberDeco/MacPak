//! Audio cache for storing decoded WEM audio
//!
//! Similar to FlagCache, this provides O(1) lookups for decoded audio data.
//! Audio is decoded on first access and cached for subsequent playback.
//!
//! ## Usage Pattern
//!
//! ```ignore
//! // 1. Create cache during state initialization
//! let audio_cache = Arc::new(RwLock::new(AudioCache::new()));
//!
//! // 2. Configure during PAK loading (in background thread)
//! cache.configure(voice_path);
//!
//! // 3. Use during playback (on-demand loading with caching)
//! let audio = cache.get_or_load(text_handle, wem_filename)?;
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::decoder::{DecodedAudio, WemError};
use crate::pak::PakOperations;

/// Maximum number of audio entries to cache (prevents unbounded memory growth)
const DEFAULT_MAX_ENTRIES: usize = 100;

/// Cache for decoded audio, keyed by text handle
///
/// Uses on-demand loading - audio is decoded when first requested,
/// then cached for subsequent playback. Uses LRU-style eviction
/// when the cache exceeds max_entries.
///
/// Similar to FlagCache but optimized for large binary data that's
/// loaded on-demand rather than all at once.
///
/// ## File Index
///
/// During `build_index()`, the cache scans the voice directory and builds
/// a HashMap of WEM filename → full path. This makes file lookups O(1)
/// instead of requiring recursive directory searches.
#[derive(Debug)]
pub struct AudioCache {
    /// Cached decoded audio, keyed by normalized text handle
    entries: HashMap<String, CachedAudio>,
    /// Order of access for LRU eviction (most recent at end)
    access_order: Vec<String>,
    /// Maximum number of entries to cache
    max_entries: usize,
    /// Base path to voice files directory or PAK file (set during configure)
    voice_path: Option<PathBuf>,
    /// Pre-built index of WEM filename → full path (for extracted files, O(1) lookups)
    file_index: HashMap<String, PathBuf>,
    /// Pre-built index of WEM filename → internal PAK path (for PAK files, O(1) lookups)
    pak_index: HashMap<String, String>,
    /// Whether voice_path is a PAK file (vs extracted directory)
    is_pak: bool,
    /// Whether the cache has been configured
    configured: bool,
    /// Whether the file index has been built
    indexed: bool,
    /// Cache statistics
    stats: CacheStats,
}

/// A cached audio entry
#[derive(Debug, Clone)]
pub struct CachedAudio {
    /// The decoded audio data
    pub audio: DecodedAudio,
    /// Size in bytes (for memory tracking)
    pub size_bytes: usize,
    /// Source file path
    pub source_path: PathBuf,
}

/// Cache statistics for debugging
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub total_bytes_cached: usize,
}

impl AudioCache {
    /// Create a new empty cache with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            max_entries: DEFAULT_MAX_ENTRIES,
            voice_path: None,
            file_index: HashMap::new(),
            pak_index: HashMap::new(),
            is_pak: false,
            configured: false,
            indexed: false,
            stats: CacheStats::default(),
        }
    }

    /// Create a cache with custom max entries
    #[must_use]
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            max_entries,
            voice_path: None,
            file_index: HashMap::new(),
            pak_index: HashMap::new(),
            is_pak: false,
            configured: false,
            indexed: false,
            stats: CacheStats::default(),
        }
    }

    /// Configure the cache with the voice files path (directory or PAK file)
    ///
    /// Call this during PAK loading to set up the cache.
    /// Accepts either an extracted Voice directory or a Voice.pak file.
    pub fn configure<P: AsRef<Path>>(&mut self, voice_path: P) {
        let path = voice_path.as_ref().to_path_buf();
        self.is_pak = path.extension().is_some_and(|e| e == "pak");
        self.voice_path = Some(path);
        self.configured = true;
        tracing::debug!(
            "AudioCache configured with voice path: {:?} (is_pak={})",
            self.voice_path,
            self.is_pak
        );
    }

    /// Configure from game data directory (finds voice path automatically)
    ///
    /// Looks for extracted Voice folder first (faster), then falls back to Voice.pak.
    pub fn configure_from_game_data<P: AsRef<Path>>(&mut self, data_path: P) {
        let data_path = data_path.as_ref();

        // Try to find voice files path
        let localization_dir = data_path.join("Localization");

        // Check for extracted Voice folder first (faster to index and load)
        let voice_dir = localization_dir.join("Voice");
        if voice_dir.exists() {
            self.configure(&voice_dir);
            return;
        }

        // Check for Voice.pak (supports on-demand extraction)
        let voice_pak = localization_dir.join("Voice.pak");
        if voice_pak.exists() {
            self.configure(&voice_pak);
            return;
        }

        tracing::warn!("No voice files found in {:?}", data_path);
    }

    /// Check if the cache has been configured
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    /// Get the configured voice path
    #[must_use]
    pub fn voice_path(&self) -> Option<&Path> {
        self.voice_path.as_deref()
    }

    /// Check if the file index has been built
    #[must_use]
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Build the file index by scanning the voice directory or PAK file
    ///
    /// For extracted directories: builds HashMap of WEM filename → full path
    /// For PAK files: builds HashMap of WEM filename → internal PAK path
    ///
    /// Call this during initialization to avoid slow searches later.
    /// Returns the number of files indexed.
    pub fn build_index(&mut self) -> usize {
        let Some(voice_path) = self.voice_path.clone() else {
            tracing::warn!("Cannot build index: voice path not configured");
            return 0;
        };

        // Don't re-index if already done
        if self.indexed {
            return if self.is_pak {
                self.pak_index.len()
            } else {
                self.file_index.len()
            };
        }

        let start = std::time::Instant::now();

        let count = if self.is_pak {
            self.build_pak_index(&voice_path)
        } else {
            self.build_directory_index(&voice_path)
        };

        let elapsed = start.elapsed();
        tracing::info!(
            "Indexed {} WEM files in {:.2}s (is_pak={})",
            count,
            elapsed.as_secs_f64(),
            self.is_pak
        );

        self.indexed = true;
        count
    }

    /// Build index from an extracted directory
    fn build_directory_index(&mut self, voice_path: &Path) -> usize {
        tracing::info!("Building WEM file index from directory {:?}...", voice_path);
        self.file_index.clear();
        self.index_directory_recursive(voice_path);
        self.file_index.len()
    }

    /// Build index from a PAK file
    fn build_pak_index(&mut self, pak_path: &Path) -> usize {
        tracing::info!("Building WEM file index from PAK {:?}...", pak_path);
        self.pak_index.clear();

        // List all files in the PAK
        let entries = match PakOperations::list(pak_path) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("Failed to list PAK contents: {}", e);
                return 0;
            }
        };

        // Index all .wem files
        for internal_path in entries {
            if internal_path.to_lowercase().ends_with(".wem") {
                // Extract filename from internal path
                if let Some(filename) = internal_path.rsplit('/').next() {
                    self.pak_index.insert(filename.to_string(), internal_path);
                }
            }
        }

        self.pak_index.len()
    }

    /// Recursively scan a directory and add all .wem files to the index
    fn index_directory_recursive(&mut self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.index_directory_recursive(&path);
            } else if let Some(ext) = path.extension() {
                if ext == "wem" {
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy().to_string();
                        self.file_index.insert(filename_str, path);
                    }
                }
            }
        }
    }

    /// Look up a WEM file path from the index (O(1))
    #[must_use]
    pub fn get_wem_path(&self, wem_filename: &str) -> Option<&PathBuf> {
        self.file_index.get(wem_filename)
    }

    /// Check if a text handle is cached
    #[must_use]
    pub fn contains(&self, text_handle: &str) -> bool {
        let normalized = normalize_handle(text_handle);
        self.entries.contains_key(&normalized)
    }

    /// Get cached audio if available (updates access order)
    pub fn get(&mut self, text_handle: &str) -> Option<&CachedAudio> {
        let normalized = normalize_handle(text_handle);

        if self.entries.contains_key(&normalized) {
            self.stats.hits += 1;
            // Update access order (move to end = most recently used)
            self.access_order.retain(|h| h != &normalized);
            self.access_order.push(normalized.clone());
            self.entries.get(&normalized)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Get or load audio for a text handle
    ///
    /// If cached, returns the cached version. Otherwise loads from disk or PAK,
    /// caches it, and returns the result.
    ///
    /// # Arguments
    /// * `text_handle` - The localization handle (e.g., "h35f3e7db-fbae-48ba-bc38-bdd1005fe3f5")
    /// * `wem_filename` - The WEM filename from VoiceMeta (e.g., "v518fab8f..._h35f3e7db....wem")
    #[cfg(feature = "audio")]
    pub fn get_or_load(
        &mut self,
        text_handle: &str,
        wem_filename: &str,
    ) -> Result<&CachedAudio, AudioCacheError> {
        let normalized = normalize_handle(text_handle);

        // Check cache first
        if self.entries.contains_key(&normalized) {
            self.stats.hits += 1;
            // Update access order
            self.access_order.retain(|h| h != &normalized);
            self.access_order.push(normalized.clone());
            return Ok(self.entries.get(&normalized).unwrap());
        }

        self.stats.misses += 1;

        // Load audio - method depends on whether we're using PAK or directory
        let (audio, source_path) = if self.is_pak {
            self.load_from_pak(wem_filename)?
        } else {
            self.load_from_directory(wem_filename)?
        };

        let size_bytes = audio.samples.len() * std::mem::size_of::<i16>();

        // Evict if needed
        while self.entries.len() >= self.max_entries && !self.access_order.is_empty() {
            self.evict_oldest();
        }

        // Cache the result
        let cached = CachedAudio {
            audio,
            size_bytes,
            source_path,
        };

        self.stats.total_bytes_cached += size_bytes;
        self.entries.insert(normalized.clone(), cached);
        self.access_order.push(normalized.clone());

        Ok(self.entries.get(&normalized).unwrap())
    }

    /// Load audio from an extracted directory
    #[cfg(feature = "audio")]
    fn load_from_directory(&self, wem_filename: &str) -> Result<(DecodedAudio, PathBuf), AudioCacheError> {
        let wem_path = if self.indexed {
            // O(1) lookup from pre-built index
            self.file_index.get(wem_filename)
                .cloned()
                .ok_or_else(|| AudioCacheError::FileNotFound(format!(
                    "WEM file '{}' not in index", wem_filename
                )))?
        } else {
            // Fallback to slow recursive search (only if index wasn't built)
            let voice_path = self.voice_path.as_ref()
                .ok_or(AudioCacheError::VoicePathNotSet)?;
            find_wem_file(voice_path, wem_filename)?
        };

        let audio = super::decoder::load_wem_file_vgmstream(&wem_path)
            .map_err(AudioCacheError::DecodeError)?;

        Ok((audio, wem_path))
    }

    /// Load audio from a PAK file (extracts to temp file, decodes, cleans up)
    #[cfg(feature = "audio")]
    fn load_from_pak(&self, wem_filename: &str) -> Result<(DecodedAudio, PathBuf), AudioCacheError> {
        let pak_path = self.voice_path.as_ref()
            .ok_or(AudioCacheError::VoicePathNotSet)?;

        // Look up internal path in PAK index
        let internal_path = if self.indexed {
            self.pak_index.get(wem_filename)
                .ok_or_else(|| AudioCacheError::FileNotFound(format!(
                    "WEM file '{}' not in PAK index", wem_filename
                )))?
        } else {
            return Err(AudioCacheError::FileNotFound(
                "PAK index not built - call build_index() first".to_string()
            ));
        };

        // Extract WEM data from PAK
        let wem_data = PakOperations::read_file_bytes(pak_path, internal_path)
            .map_err(|e| AudioCacheError::FileNotFound(format!(
                "Failed to extract '{}' from PAK: {}", internal_path, e
            )))?;

        // Write to temp file for vgmstream
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("macpak_wem_{}.wem", std::process::id()));

        std::fs::write(&temp_path, &wem_data)
            .map_err(|e| AudioCacheError::FileNotFound(format!(
                "Failed to write temp file: {}", e
            )))?;

        // Decode with vgmstream
        let result = super::decoder::load_wem_file_vgmstream(&temp_path);

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        let audio = result.map_err(AudioCacheError::DecodeError)?;

        // Return PAK path as the source (for debugging/display)
        Ok((audio, pak_path.clone()))
    }

    /// Insert pre-decoded audio into the cache
    pub fn insert(&mut self, text_handle: &str, audio: DecodedAudio, source_path: PathBuf) {
        let normalized = normalize_handle(text_handle);
        let size_bytes = audio.samples.len() * std::mem::size_of::<i16>();

        // Evict if needed
        while self.entries.len() >= self.max_entries && !self.access_order.is_empty() {
            self.evict_oldest();
        }

        let cached = CachedAudio {
            audio,
            size_bytes,
            source_path,
        };

        self.stats.total_bytes_cached += size_bytes;
        self.access_order.retain(|h| h != &normalized);
        self.access_order.push(normalized.clone());
        self.entries.insert(normalized, cached);
    }

    /// Evict the oldest (least recently used) entry
    fn evict_oldest(&mut self) {
        if let Some(oldest) = self.access_order.first().cloned() {
            if let Some(entry) = self.entries.remove(&oldest) {
                self.stats.total_bytes_cached = self.stats.total_bytes_cached.saturating_sub(entry.size_bytes);
                self.stats.evictions += 1;
            }
            self.access_order.remove(0);
        }
    }

    /// Clear all cached audio
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
    pub fn stats(&self) -> &CacheStats {
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

impl Default for AudioCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize a text handle for cache lookup
/// Converts "h35f3e7db-fbae-48ba-bc38-bdd1005fe3f5" to "h35f3e7dbgfbaeg48bagbc38gbdd1005fe3f5"
fn normalize_handle(handle: &str) -> String {
    handle.replace('-', "g")
}

/// Find the .wem file for a given filename in the voice directory
/// This is a fallback when the index wasn't built - prefer using the index for O(1) lookups.
#[cfg(feature = "audio")]
fn find_wem_file(voice_path: &Path, wem_filename: &str) -> Result<PathBuf, AudioCacheError> {
    // Common search paths for voice files
    let search_dirs = [
        voice_path.join("Mods/Gustav/Localization/English/Soundbanks"),
        voice_path.join("Mods/GustavDev/Localization/English/Soundbanks"),
        voice_path.join("Mods/Shared/Localization/English/Soundbanks"),
        voice_path.to_path_buf(),
    ];

    for dir in &search_dirs {
        let wem_path = dir.join(wem_filename);
        if wem_path.exists() {
            return Ok(wem_path);
        }
    }

    // Recursive search as fallback
    if let Some(found) = find_file_recursive(voice_path, wem_filename) {
        return Ok(found);
    }

    Err(AudioCacheError::FileNotFound(format!(
        "Could not find WEM file '{}' in {:?}",
        wem_filename, voice_path
    )))
}

/// Recursively search for a file
#[cfg(feature = "audio")]
fn find_file_recursive(dir: &Path, filename: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_recursive(&path, filename) {
                return Some(found);
            }
        } else if path.file_name().is_some_and(|n| n == filename) {
            return Some(path);
        }
    }

    None
}

/// Errors that can occur with the audio cache
#[derive(Debug, thiserror::Error)]
pub enum AudioCacheError {
    #[error("Voice files path not configured")]
    VoicePathNotSet,
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Decode error: {0}")]
    DecodeError(#[from] WemError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_handle() {
        assert_eq!(
            normalize_handle("h35f3e7db-fbae-48ba-bc38-bdd1005fe3f5"),
            "h35f3e7dbgfbaeg48bagbc38gbdd1005fe3f5"
        );
    }

    #[test]
    fn test_cache_basic() {
        let mut cache = AudioCache::with_max_entries(2);

        let audio = DecodedAudio {
            samples: vec![0i16; 1000],
            channels: 1,
            sample_rate: 44100,
        };

        cache.insert("handle1", audio.clone(), PathBuf::from("/test1.wem"));
        assert_eq!(cache.len(), 1);
        assert!(cache.contains("handle1"));

        cache.insert("handle2", audio.clone(), PathBuf::from("/test2.wem"));
        assert_eq!(cache.len(), 2);

        // This should evict handle1 (oldest)
        cache.insert("handle3", audio, PathBuf::from("/test3.wem"));
        assert_eq!(cache.len(), 2);
        assert!(!cache.contains("handle1"));
        assert!(cache.contains("handle2"));
        assert!(cache.contains("handle3"));
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = AudioCache::with_max_entries(2);

        let audio = DecodedAudio {
            samples: vec![0i16; 100],
            channels: 1,
            sample_rate: 44100,
        };

        cache.insert("handle1", audio.clone(), PathBuf::from("/test1.wem"));
        cache.insert("handle2", audio.clone(), PathBuf::from("/test2.wem"));

        // Access handle1 to make it more recent
        let _ = cache.get("handle1");

        // Insert handle3, should evict handle2 (now oldest)
        cache.insert("handle3", audio, PathBuf::from("/test3.wem"));

        assert!(cache.contains("handle1"));
        assert!(!cache.contains("handle2"));
        assert!(cache.contains("handle3"));
    }
}
