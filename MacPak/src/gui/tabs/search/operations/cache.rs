//! Index cache operations

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

use crate::gui::state::{IndexStatus, SearchState};

use super::progress::INDEX_AUTO_LOADED;

/// Get the standard cache directory for the search index
pub fn get_index_cache_path() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("MacPak").join("search_index"))
}

/// Attempt to auto-load a cached index on first visit to Search tab.
/// This runs silently in the background without showing any dialogs.
pub fn auto_load_cached_index(state: SearchState) {
    // Only attempt once per session
    if INDEX_AUTO_LOADED.swap(true, Ordering::SeqCst) {
        return;
    }

    let cache_path = match get_index_cache_path() {
        Some(p) => p,
        None => return,
    };

    // Check if cached index exists
    if !cache_path.join("metadata.json").exists() {
        return;
    }

    let index = state.index.clone();
    let index_status = state.index_status;

    // Set a loading status
    index_status.set(IndexStatus::Building {
        progress: "Loading cached index...".to_string(),
    });

    // Load in background thread
    let send = create_ext_action(
        Scope::new(),
        move |result: Result<(usize, usize), String>| {
            match result {
                Ok((file_count, pak_count)) => {
                    index_status.set(IndexStatus::Ready {
                        file_count,
                        pak_count,
                    });
                    tracing::info!(
                        "Auto-loaded cached index: {} files from {} PAKs",
                        file_count,
                        pak_count
                    );
                }
                Err(e) => {
                    // Silently fail - user can manually rebuild
                    tracing::warn!("Failed to auto-load cached index: {}", e);
                    index_status.set(IndexStatus::NotBuilt);
                }
            }
        },
    );

    std::thread::spawn(move || {
        let result = index
            .write()
            .map_err(|e| e.to_string())
            .and_then(|mut idx| {
                idx.import_index(&cache_path).map_err(|e| e.to_string())?;
                Ok((idx.file_count(), idx.pak_count()))
            });
        send(result);
    });
}

/// Save the index to the cache directory (called automatically after building)
pub fn auto_save_index(index: Arc<RwLock<crate::search::SearchIndex>>) {
    let cache_path = match get_index_cache_path() {
        Some(p) => p,
        None => return,
    };

    std::thread::spawn(move || {
        // Ensure cache directory exists
        if let Err(e) = std::fs::create_dir_all(&cache_path) {
            tracing::warn!("Failed to create index cache directory: {}", e);
            return;
        }

        // Export index silently
        match index.read() {
            Ok(idx) => {
                if let Err(e) = idx.export_index(&cache_path) {
                    tracing::warn!("Failed to auto-save index: {}", e);
                } else {
                    tracing::info!("Auto-saved index to cache");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to acquire read lock for auto-save: {}", e);
            }
        }
    });
}
