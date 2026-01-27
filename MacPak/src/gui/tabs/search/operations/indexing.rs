//! Index building operations

use std::path::PathBuf;

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

use crate::gui::state::{IndexStatus, SearchState};

use super::cache::auto_save_index;
use super::progress::SEARCH_PROGRESS;

/// Messages from background indexing thread
pub enum IndexMessage {
    Complete { file_count: usize, pak_count: usize },
    Error(String),
}

/// Find PAK files in a directory
pub fn find_pak_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut paks = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "pak") {
                paks.push(path);
            }
        }
    }

    // Sort by name for consistent ordering
    paks.sort();
    paks
}

/// Build the search index in a background thread
pub fn build_index(state: SearchState) {
    let pak_paths = state.pak_paths.get();
    if pak_paths.is_empty() {
        return;
    }

    let index = state.index.clone();
    let index_status = state.index_status;
    let show_progress = state.show_progress;
    let pak_count_display = pak_paths.len();

    // Set building status
    index_status.set(IndexStatus::Building {
        progress: format!("Indexing {} PAK files...", pak_count_display),
    });

    // Show progress dialog for content indexing
    show_progress.set(true);
    SEARCH_PROGRESS.reset();
    SEARCH_PROGRESS.set_active(true);

    // Create action for sending result back to UI thread
    let send = create_ext_action(Scope::new(), move |msg: IndexMessage| {
        SEARCH_PROGRESS.set_active(false);
        match msg {
            IndexMessage::Complete {
                file_count,
                pak_count,
            } => {
                show_progress.set(false);
                index_status.set(IndexStatus::Ready {
                    file_count,
                    pak_count,
                });
            }
            IndexMessage::Error(msg) => {
                show_progress.set(false);
                index_status.set(IndexStatus::Error(msg));
            }
        }
    });

    // Clone index for auto-save after build completes
    let index_for_save = index.clone();

    // Spawn background thread
    std::thread::spawn(move || {
        let build_succeeded;
        match index.write() {
            Ok(mut idx) => {
                // Phase 1: Build metadata index (fast)
                SEARCH_PROGRESS.set(0, 1, "Building file index...".to_string());

                match idx.build_index(&pak_paths) {
                    Ok(file_count) => {
                        let pak_count = idx.pak_count();

                        // Phase 2: Build fulltext index (slower, extracts content)
                        // Progress is reported via SEARCH_PROGRESS in the callback
                        let progress_callback = |progress: &maclarian::search::SearchProgress| {
                            let name = progress
                                .current_file
                                .as_deref()
                                .unwrap_or(progress.phase.as_str());
                            SEARCH_PROGRESS.set(progress.current, progress.total, name.to_string());
                        };

                        match idx.build_fulltext_index(&progress_callback) {
                            Ok(indexed) => {
                                tracing::info!("Fulltext index built for {} files", indexed);
                            }
                            Err(e) => {
                                tracing::warn!("Fulltext index failed: {}", e);
                                // Continue anyway - deep search will use fallback
                            }
                        }

                        send(IndexMessage::Complete {
                            file_count,
                            pak_count,
                        });
                        build_succeeded = true;
                    }
                    Err(e) => {
                        send(IndexMessage::Error(format!("Index build failed: {}", e)));
                        build_succeeded = false;
                    }
                }
            }
            Err(e) => {
                send(IndexMessage::Error(format!("Failed to acquire lock: {}", e)));
                build_succeeded = false;
            }
        }

        // Auto-save index to cache after successful build
        if build_succeeded {
            auto_save_index(index_for_save);
        }
    });
}
