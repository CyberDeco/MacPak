//! Voice metadata operations - thin wrapper around maclarian's voice_meta module
//!
//! The heavy lifting (parallel loading, parsing) is done in maclarian.
//! This module provides cache management for the GUI.

use std::path::Path;
use std::sync::{Arc, RwLock};
use maclarian::formats::voice_meta::{VoiceMetaCache, find_voice_meta_path};

// Re-export for convenience
pub use maclarian::formats::voice_meta::find_voice_files_path;

/// Load voice metadata from VoiceMeta.pak or extracted Soundbanks folder
/// Populates the cache with text_handle -> VoiceMetaEntry mappings
pub fn load_voice_meta(
    cache: &Arc<RwLock<VoiceMetaCache>>,
    data_path: &Path,
) -> usize {
    // Check if already loaded
    {
        let Ok(cache_read) = cache.read() else {
            tracing::error!("Failed to acquire voice meta cache read lock");
            return 0;
        };
        if !cache_read.is_empty() {
            return cache_read.len();
        }
    }

    // Find the voice meta source
    let Some(voice_meta_path) = find_voice_meta_path(data_path) else {
        tracing::warn!("No VoiceMeta.pak or folder found");
        return 0;
    };

    // Load using maclarian's parallel loader
    let result = if voice_meta_path.is_file() {
        maclarian::formats::voice_meta::load_voice_meta_from_pak(&voice_meta_path)
    } else {
        maclarian::formats::voice_meta::load_voice_meta_from_folder(&voice_meta_path)
    };

    match result {
        Ok(entries) => {
            let count = entries.len();
            let Ok(mut cache_write) = cache.write() else {
                tracing::error!("Failed to acquire voice meta cache write lock");
                return 0;
            };
            *cache_write = entries;
            count
        }
        Err(e) => {
            tracing::error!("Failed to load voice meta: {}", e);
            0
        }
    }
}
