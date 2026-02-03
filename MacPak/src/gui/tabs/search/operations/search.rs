//! Search execution operations

use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

use crate::gui::state::{SearchResult, SearchState};

use super::progress::{MAX_RESULTS, SEARCH_PROGRESS};

/// Messages from background search thread
enum SearchMessage {
    Results(Vec<SearchResult>),
    Error(String),
}

/// Perform a search in a background thread
pub fn perform_search(state: SearchState) {
    let query = state.query.get();
    if query.is_empty() {
        return;
    }

    let index = state.index.clone();
    let active_filter = state.active_filter.get();
    let is_searching = state.is_searching;
    let results_signal = state.results;

    // Set searching state
    is_searching.set(true);
    results_signal.set(Vec::new());

    // Create action for sending final results back to UI thread
    let send_results = create_ext_action(Scope::new(), move |msg: SearchMessage| match msg {
        SearchMessage::Results(results) => {
            is_searching.set(false);
            results_signal.set(results);
        }
        SearchMessage::Error(msg) => {
            is_searching.set(false);
            tracing::error!("Search error: {}", msg);
        }
    });

    // Spawn background thread
    std::thread::spawn(move || {
        let idx = match index.read() {
            Ok(idx) => idx,
            Err(e) => {
                send_results(SearchMessage::Error(format!(
                    "Failed to acquire lock: {}",
                    e
                )));
                return;
            }
        };

        // Combined search: fulltext (content matches) + filename/path (all file types)
        SEARCH_PROGRESS.set_active(true);
        SEARCH_PROGRESS.set(0, 1, "Searching...".to_string());

        // 1. Get fulltext results (text files with content matches)
        let fulltext_results: Vec<SearchResult> = if idx.has_fulltext() {
            let progress_callback = |progress: &crate::search::SearchProgress| {
                let name = progress
                    .current_file
                    .as_deref()
                    .unwrap_or(progress.phase.as_str());
                SEARCH_PROGRESS.set(progress.current, progress.total, name.to_string());
            };
            let ft_results = idx
                .search_fulltext_with_progress(&query, MAX_RESULTS, &progress_callback)
                .unwrap_or_default();

            ft_results
                .into_iter()
                .filter(|r| {
                    active_filter.map_or(true, |ft| {
                        r.file_type.to_lowercase() == ft.display_name().to_lowercase()
                    })
                })
                .map(|r| {
                    let match_count = if r.match_count > 0 {
                        Some(r.match_count)
                    } else {
                        None
                    };
                    SearchResult {
                        name: r.name,
                        path: r.path,
                        pak_file: r
                            .pak_file
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        file_type: r.file_type,
                        pak_path: r.pak_file,
                        context: r.snippet,
                        match_count,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        // 2. Get filename/path matches (ALL file types including images, audio, models)
        let filename_results: Vec<SearchResult> = idx
            .search_path(&query, active_filter)
            .into_iter()
            .take(MAX_RESULTS)
            .map(|f| SearchResult::from_indexed_file(f))
            .collect();

        // 3. Merge results with deduplication (fulltext results take priority - they have snippets)
        let mut seen_paths: HashSet<String> = HashSet::new();
        let mut merged: Vec<SearchResult> =
            Vec::with_capacity(fulltext_results.len() + filename_results.len());

        // Add fulltext results first (they have context snippets)
        for result in fulltext_results {
            if seen_paths.insert(result.path.clone()) {
                merged.push(result);
            }
        }

        // Add filename matches that weren't already found via fulltext
        for result in filename_results {
            if seen_paths.insert(result.path.clone()) {
                merged.push(result);
            }
        }

        SEARCH_PROGRESS.set_active(false);
        send_results(SearchMessage::Results(merged));
    });
}

/// Copy text to system clipboard (macOS)
pub fn copy_to_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}
