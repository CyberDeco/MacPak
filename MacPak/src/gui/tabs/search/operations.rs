//! Search operations and background processing

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use floem::action::exec_after;
use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem::text::Weight;
use floem_reactive::{create_effect, Scope};
use crate::gui::state::{IndexStatus, SearchResult, SearchState};

/// Shared progress state for search operations (thread-safe)
#[derive(Default)]
struct SharedSearchProgress {
    current: AtomicUsize,
    total: AtomicUsize,
    message: Mutex<String>,
    active: AtomicBool,
}

impl SharedSearchProgress {
    fn set(&self, current: usize, total: usize, message: String) {
        self.current.store(current, Ordering::SeqCst);
        self.total.store(total, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            *msg = message;
        }
    }

    fn get(&self) -> (usize, usize, String) {
        let msg = self.message.lock().map(|m| m.clone()).unwrap_or_default();
        (
            self.current.load(Ordering::SeqCst),
            self.total.load(Ordering::SeqCst),
            msg,
        )
    }

    fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    fn reset(&self) {
        self.current.store(0, Ordering::SeqCst);
        self.total.store(0, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            *msg = String::new();
        }
    }
}

lazy_static::lazy_static! {
    static ref SEARCH_PROGRESS: Arc<SharedSearchProgress> = Arc::new(SharedSearchProgress::default());
}

/// Maximum results for fulltext search
const MAX_RESULTS: usize = 50000;

/// Messages from background indexing thread
enum IndexMessage {
    Complete { file_count: usize, pak_count: usize },
    Error(String),
}

/// Messages from background search thread
enum SearchMessage {
    Results(Vec<SearchResult>),
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
            IndexMessage::Complete { file_count, pak_count } => {
                show_progress.set(false);
                index_status.set(IndexStatus::Ready { file_count, pak_count });
            }
            IndexMessage::Error(msg) => {
                show_progress.set(false);
                index_status.set(IndexStatus::Error(msg));
            }
        }
    });

    // Spawn background thread
    std::thread::spawn(move || {
        match index.write() {
            Ok(mut idx) => {
                // Phase 1: Build metadata index (fast)
                SEARCH_PROGRESS.set(0, 1, "Building file index...".to_string());

                match idx.build_index(&pak_paths) {
                    Ok(file_count) => {
                        let pak_count = idx.pak_count();

                        // Phase 2: Build fulltext index (slower, extracts content)
                        // Progress is reported via SEARCH_PROGRESS in the callback
                        let progress_callback = |current: usize, total: usize, name: &str| {
                            SEARCH_PROGRESS.set(current, total, name.to_string());
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

                        send(IndexMessage::Complete { file_count, pak_count });
                    }
                    Err(e) => {
                        send(IndexMessage::Error(format!("Index build failed: {}", e)));
                    }
                }
            }
            Err(e) => {
                send(IndexMessage::Error(format!("Failed to acquire lock: {}", e)));
            }
        }
    });
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
    let send_results = create_ext_action(Scope::new(), move |msg: SearchMessage| {
        match msg {
            SearchMessage::Results(results) => {
                is_searching.set(false);
                results_signal.set(results);
            }
            SearchMessage::Error(msg) => {
                is_searching.set(false);
                tracing::error!("Search error: {}", msg);
            }
        }
    });

    // Spawn background thread
    std::thread::spawn(move || {
        use std::time::Instant;
        let total_start = Instant::now();

        let lock_start = Instant::now();
        let idx = match index.read() {
            Ok(idx) => idx,
            Err(e) => {
                send_results(SearchMessage::Error(format!("Failed to acquire lock: {}", e)));
                return;
            }
        };
        let lock_time = lock_start.elapsed();

        // Fulltext search using Tantivy index with progress reporting
        SEARCH_PROGRESS.set_active(true);
        SEARCH_PROGRESS.set(0, 1, "Searching...".to_string());

        let results: Vec<SearchResult> = if idx.has_fulltext() {
            let search_start = Instant::now();
            let progress_callback = |current: usize, total: usize, name: &str| {
                SEARCH_PROGRESS.set(current, total, name.to_string());
            };
            let ft_results = idx.search_fulltext_with_progress(&query, MAX_RESULTS, progress_callback).unwrap_or_default();
            let search_time = search_start.elapsed();
            let result_count = ft_results.len();

            let transform_start = Instant::now();
            let results: Vec<SearchResult> = ft_results
                .into_iter()
                .filter(|r| active_filter.map_or(true, |ft| {
                    r.file_type.to_lowercase() == ft.display_name().to_lowercase()
                }))
                .map(|r| {
                    // Snippet is already processed by the fulltext module
                    let match_count = if r.match_count > 0 { Some(r.match_count) } else { None };
                    SearchResult {
                        name: r.name,
                        path: r.path,
                        pak_file: r.pak_file.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        file_type: r.file_type,
                        pak_path: r.pak_file,
                        context: r.snippet,
                        match_count,
                    }
                })
                .collect();
            let transform_time = transform_start.elapsed();

            eprintln!(
                "Search timing: lock={:?}, tantivy={:?} ({} results), transform={:?} ({} final)",
                lock_time, search_time, result_count, transform_time, results.len()
            );

            results
        } else {
            eprintln!("Search: no fulltext index available");
            Vec::new()
        };

        SEARCH_PROGRESS.set_active(false);
        eprintln!("Search total: {:?}", total_start.elapsed());
        send_results(SearchMessage::Results(results));
    });
}

/// Copy text to system clipboard (macOS)
pub fn copy_to_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Progress overlay shown during long-running search operations
/// Uses polling to read from shared atomic state updated by background thread
pub fn progress_overlay(state: SearchState) -> impl IntoView {
    let show = state.show_progress;

    // Local signals for polled values
    let polled_current = RwSignal::new(0usize);
    let polled_total = RwSignal::new(0usize);
    let polled_msg = RwSignal::new(String::new());
    let polled_pct = RwSignal::new(0u32);
    let timer_active = RwSignal::new(false);

    // Polling function
    fn poll_and_schedule(
        polled_current: RwSignal<usize>,
        polled_total: RwSignal<usize>,
        polled_msg: RwSignal<String>,
        polled_pct: RwSignal<u32>,
        show: RwSignal<bool>,
        timer_active: RwSignal<bool>,
    ) {
        let (current, total, msg) = SEARCH_PROGRESS.get();
        polled_current.set(current);
        polled_total.set(total);
        if !msg.is_empty() {
            polled_msg.set(msg);
        }
        if total > 0 {
            polled_pct.set(((current as f64 / total as f64) * 100.0) as u32);
        }

        // Schedule next poll if still active
        if show.get_untracked() && timer_active.get_untracked() {
            exec_after(Duration::from_millis(50), move |_| {
                if show.get_untracked() && timer_active.get_untracked() {
                    poll_and_schedule(polled_current, polled_total, polled_msg, polled_pct, show, timer_active);
                }
            });
        }
    }

    // Start/stop polling based on visibility
    create_effect(move |_| {
        let visible = show.get();
        if visible {
            SEARCH_PROGRESS.reset();
            polled_current.set(0);
            polled_total.set(0);
            polled_msg.set("Preparing...".to_string());
            polled_pct.set(0);
            timer_active.set(true);

            exec_after(Duration::from_millis(50), move |_| {
                if show.get_untracked() {
                    poll_and_schedule(polled_current, polled_total, polled_msg, polled_pct, show, timer_active);
                }
            });
        } else {
            timer_active.set(false);
        }
    });

    dyn_container(
        move || show.get(),
        move |is_visible| {
            if is_visible {
                container(
                    v_stack((
                        // Title
                        label(|| "Indexing...")
                            .style(|s| {
                                s.font_size(16.0)
                                    .font_weight(Weight::BOLD)
                                    .margin_bottom(12.0)
                            }),
                        // Count display (e.g., "1/5000")
                        label(move || {
                            let t = polled_total.get();
                            let c = polled_current.get();
                            if t > 0 {
                                format!("{}/{}", c, t)
                            } else {
                                String::new()
                            }
                        })
                        .style(|s| {
                            s.font_size(13.0)
                                .color(Color::rgb8(100, 100, 100))
                                .margin_bottom(4.0)
                        }),
                        // Current file being searched
                        label(move || polled_msg.get())
                            .style(|s| {
                                s.font_size(12.0)
                                    .color(Color::rgb8(120, 120, 120))
                                    .margin_bottom(12.0)
                                    .text_ellipsis()
                                    .max_width(450.0)
                            }),
                        // Progress bar
                        container(
                            container(empty())
                                .style(move |s| {
                                    let pct = polled_pct.get();
                                    s.height_full()
                                        .width_pct(pct as f64)
                                        .background(Color::rgb8(33, 150, 243))
                                        .border_radius(4.0)
                                }),
                        )
                        .style(|s| {
                            s.width_full()
                                .height(8.0)
                                .background(Color::rgb8(220, 220, 220))
                                .border_radius(4.0)
                        }),
                        label(move || format!("{}%", polled_pct.get()))
                            .style(|s| s.font_size(12.0).margin_top(8.0).color(Color::rgb8(100, 100, 100))),
                    ))
                    .style(|s| {
                        s.padding(24.0)
                            .background(Color::WHITE)
                            .border(1.0)
                            .border_color(Color::rgb8(200, 200, 200))
                            .border_radius(8.0)
                            .width(500.0)
                    }),
                )
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
}

/// Overlay shown while search is in progress with progress bar
pub fn search_overlay(state: SearchState) -> impl IntoView {
    let is_searching = state.is_searching;

    // Local signals for polled values
    let polled_current = RwSignal::new(0usize);
    let polled_total = RwSignal::new(0usize);
    let polled_msg = RwSignal::new(String::new());
    let polled_pct = RwSignal::new(0u32);
    let timer_active = RwSignal::new(false);

    // Polling function
    fn poll_search_progress(
        polled_current: RwSignal<usize>,
        polled_total: RwSignal<usize>,
        polled_msg: RwSignal<String>,
        polled_pct: RwSignal<u32>,
        is_searching: RwSignal<bool>,
        timer_active: RwSignal<bool>,
    ) {
        let (current, total, msg) = SEARCH_PROGRESS.get();
        polled_current.set(current);
        polled_total.set(total);
        if !msg.is_empty() {
            polled_msg.set(msg);
        }
        if total > 0 {
            polled_pct.set(((current as f64 / total as f64) * 100.0) as u32);
        }

        // Schedule next poll if still active
        if is_searching.get_untracked() && timer_active.get_untracked() {
            exec_after(Duration::from_millis(50), move |_| {
                if is_searching.get_untracked() && timer_active.get_untracked() {
                    poll_search_progress(polled_current, polled_total, polled_msg, polled_pct, is_searching, timer_active);
                }
            });
        }
    }

    // Start/stop polling based on search state
    create_effect(move |_| {
        let searching = is_searching.get();
        if searching {
            polled_current.set(0);
            polled_total.set(0);
            polled_msg.set("Searching...".to_string());
            polled_pct.set(0);
            timer_active.set(true);

            exec_after(Duration::from_millis(50), move |_| {
                if is_searching.get_untracked() {
                    poll_search_progress(polled_current, polled_total, polled_msg, polled_pct, is_searching, timer_active);
                }
            });
        } else {
            timer_active.set(false);
        }
    });

    dyn_container(
        move || is_searching.get(),
        move |searching| {
            if searching {
                container(
                    v_stack((
                        label(|| "Searching...")
                            .style(|s| {
                                s.font_size(16.0)
                                    .font_weight(Weight::BOLD)
                                    .margin_bottom(12.0)
                            }),
                        // Count display
                        label(move || {
                            let t = polled_total.get();
                            let c = polled_current.get();
                            if t > 1 {
                                format!("{}/{}", c, t)
                            } else {
                                String::new()
                            }
                        })
                        .style(|s| {
                            s.font_size(13.0)
                                .color(Color::rgb8(100, 100, 100))
                                .margin_bottom(4.0)
                        }),
                        // Current file
                        label(move || polled_msg.get())
                            .style(|s| {
                                s.font_size(12.0)
                                    .color(Color::rgb8(120, 120, 120))
                                    .margin_bottom(12.0)
                                    .text_ellipsis()
                                    .max_width(450.0)
                            }),
                        // Progress bar
                        container(
                            container(empty())
                                .style(move |s| {
                                    let pct = polled_pct.get();
                                    s.height_full()
                                        .width_pct(pct as f64)
                                        .background(Color::rgb8(33, 150, 243))
                                        .border_radius(4.0)
                                }),
                        )
                        .style(|s| {
                            s.width_full()
                                .height(8.0)
                                .background(Color::rgb8(220, 220, 220))
                                .border_radius(4.0)
                        }),
                        label(move || format!("{}%", polled_pct.get()))
                            .style(|s| s.font_size(12.0).margin_top(8.0).color(Color::rgb8(100, 100, 100))),
                    ))
                    .style(|s| {
                        s.padding(24.0)
                            .background(Color::WHITE)
                            .border(1.0)
                            .border_color(Color::rgb8(200, 200, 200))
                            .border_radius(8.0)
                            .width(500.0)
                            .items_center()
                    }),
                )
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if is_searching.get() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
}

/// Export the index to a user-selected directory
pub fn export_index(state: SearchState) {
    // Get destination folder
    let dest = match rfd::FileDialog::new()
        .set_title("Export Index To...")
        .pick_folder()
    {
        Some(d) => d,
        None => return,
    };

    let index = state.index.clone();
    let show_progress = state.show_progress;

    show_progress.set(true);
    SEARCH_PROGRESS.reset();
    SEARCH_PROGRESS.set(0, 1, "Exporting index...".to_string());

    let dest_for_msg = dest.clone();
    let send = create_ext_action(Scope::new(), move |result: Result<(), String>| {
        show_progress.set(false);
        match result {
            Ok(_) => {
                rfd::MessageDialog::new()
                    .set_title("Export Complete")
                    .set_description(&format!("Index exported to:\n{}", dest_for_msg.display()))
                    .show();
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Export Failed")
                    .set_description(&e)
                    .show();
            }
        }
    });

    std::thread::spawn(move || {
        let result = index.read()
            .map_err(|e| e.to_string())
            .and_then(|idx| {
                idx.export_index_with_progress(&dest, |current, total, msg| {
                    SEARCH_PROGRESS.set(current, total, msg.to_string());
                }).map_err(|e| e.to_string())
            });
        send(result);
    });
}

/// Extract selected search results to a user-selected directory
pub fn extract_selected_results(state: SearchState) {
    use std::collections::HashMap;
    use MacLarian::pak::PakOperations;

    let selected_paths = state.selected_results.get();
    if selected_paths.is_empty() {
        return;
    }

    let all_results = state.results.get();

    // Filter to selected results
    let to_extract: Vec<SearchResult> = all_results
        .into_iter()
        .filter(|r| selected_paths.contains(&r.path))
        .collect();

    if to_extract.is_empty() {
        return;
    }

    // Get destination folder
    let dest = match rfd::FileDialog::new()
        .set_title("Extract Selected Files To...")
        .pick_folder()
    {
        Some(d) => d,
        None => return,
    };

    // Group by PAK file for efficient extraction
    let mut by_pak: HashMap<PathBuf, Vec<String>> = HashMap::new();
    for result in &to_extract {
        by_pak
            .entry(result.pak_path.clone())
            .or_default()
            .push(result.path.clone());
    }

    let total_files = to_extract.len();
    let show_progress = state.show_progress;
    let selected_results = state.selected_results;

    show_progress.set(true);
    SEARCH_PROGRESS.reset();
    SEARCH_PROGRESS.set(0, total_files, "Extracting files...".to_string());

    let send = create_ext_action(Scope::new(), move |result: Result<usize, String>| {
        show_progress.set(false);
        match result {
            Ok(count) => {
                selected_results.set(std::collections::HashSet::new()); // Clear selection
                rfd::MessageDialog::new()
                    .set_title("Extraction Complete")
                    .set_description(&format!("Extracted {} files", count))
                    .show();
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Extraction Failed")
                    .set_description(&e)
                    .show();
            }
        }
    });

    std::thread::spawn(move || {
        let mut total_extracted = 0;
        for (pak_path, file_paths) in by_pak {
            let paths: Vec<&str> = file_paths.iter().map(|s| s.as_str()).collect();
            match PakOperations::extract_files_with_progress(&pak_path, &dest, &paths, &|current, _total, _| {
                SEARCH_PROGRESS.set(total_extracted + current, total_files, format!("Extracting from {}", pak_path.file_name().unwrap_or_default().to_string_lossy()));
            }) {
                Ok(_) => total_extracted += paths.len(),
                Err(e) => {
                    send(Err(e.to_string()));
                    return;
                }
            }
        }
        send(Ok(total_extracted));
    });
}

/// Import an index from a user-selected directory
pub fn import_index(state: SearchState) {
    // Get source folder
    let source = match rfd::FileDialog::new()
        .set_title("Import Index From...")
        .pick_folder()
    {
        Some(d) => d,
        None => return,
    };

    let index = state.index.clone();
    let index_status = state.index_status;
    let show_progress = state.show_progress;

    show_progress.set(true);
    SEARCH_PROGRESS.reset();
    SEARCH_PROGRESS.set(0, 1, "Importing index...".to_string());

    let send = create_ext_action(Scope::new(), move |result: Result<(usize, usize), String>| {
        show_progress.set(false);
        match result {
            Ok((file_count, pak_count)) => {
                index_status.set(IndexStatus::Ready { file_count, pak_count });
                rfd::MessageDialog::new()
                    .set_title("Import Complete")
                    .set_description(&format!("Index loaded: {} files from {} PAKs", file_count, pak_count))
                    .show();
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Import Failed")
                    .set_description(&e)
                    .show();
            }
        }
    });

    std::thread::spawn(move || {
        let result = index.write()
            .map_err(|e| e.to_string())
            .and_then(|mut idx| {
                idx.import_index(&source).map_err(|e| e.to_string())?;
                Ok((idx.file_count(), idx.pak_count()))
            });
        send(result);
    });
}
