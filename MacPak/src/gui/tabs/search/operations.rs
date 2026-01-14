//! Search operations and background processing

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use floem::action::exec_after;
use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem::text::Weight;
use floem_reactive::{create_effect, Scope};
use rayon::prelude::*;
use MacLarian::formats::common::extract_value;
use MacLarian::formats::lsf::parse_lsf_bytes;
use MacLarian::pak::{PakOperations, PakReaderCache};
use MacLarian::search::{FileType, IndexedFile};

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
    /// Custom thread pool with limited parallelism (75% of cores) to avoid overwhelming the CPU
    static ref SEARCH_POOL: rayon::ThreadPool = {
        let cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        rayon::ThreadPoolBuilder::new()
            .num_threads((cores * 3 / 4).max(1))
            .build()
            .expect("Failed to create search thread pool")
    };
    /// Timing enabled via MACPAK_TIMING environment variable
    static ref TIMING_ENABLED: bool = std::env::var("MACPAK_TIMING").is_ok();
    /// Accumulated timing stats for parallel operations (in milliseconds)
    static ref TIMING_STATS: SearchTimingStats = SearchTimingStats::default();
}

/// Thread-safe timing statistics for search operations
#[derive(Default)]
struct SearchTimingStats {
    pak_read_ms: AtomicU64,
    lsf_parse_ms: AtomicU64,
    content_search_ms: AtomicU64,
    file_count: AtomicUsize,
}

impl SearchTimingStats {
    fn add_pak_read(&self, duration: Duration) {
        self.pak_read_ms.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }

    fn add_lsf_parse(&self, duration: Duration) {
        self.lsf_parse_ms.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }

    fn add_content_search(&self, duration: Duration) {
        self.content_search_ms.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }

    fn increment_files(&self) {
        self.file_count.fetch_add(1, Ordering::Relaxed);
    }

    fn reset(&self) {
        self.pak_read_ms.store(0, Ordering::SeqCst);
        self.lsf_parse_ms.store(0, Ordering::SeqCst);
        self.content_search_ms.store(0, Ordering::SeqCst);
        self.file_count.store(0, Ordering::SeqCst);
    }

    fn report(&self) -> String {
        let files = self.file_count.load(Ordering::SeqCst);
        let pak_ms = self.pak_read_ms.load(Ordering::SeqCst);
        let lsf_ms = self.lsf_parse_ms.load(Ordering::SeqCst);
        let search_ms = self.content_search_ms.load(Ordering::SeqCst);
        format!(
            "Files: {}, PAK read: {}ms, LSF parse: {}ms, Search: {}ms",
            files, pak_ms, lsf_ms, search_ms
        )
    }
}

/// Maximum results for quick search (filename matching)
const MAX_QUICK_RESULTS: usize = 5000;

/// Maximum results for deep search (content matching)
const MAX_DEEP_RESULTS: usize = 50000;

/// Maximum files to scan during deep search
const MAX_DEEP_SCAN_FILES: usize = 100000;

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
    let pak_count_display = pak_paths.len();

    // Set building status
    index_status.set(IndexStatus::Building {
        progress: format!("Indexing {} PAK files...", pak_count_display),
    });

    // Create action for sending result back to UI thread
    let send = create_ext_action(Scope::new(), move |msg: IndexMessage| match msg {
        IndexMessage::Complete { file_count, pak_count } => {
            index_status.set(IndexStatus::Ready { file_count, pak_count });
        }
        IndexMessage::Error(msg) => {
            index_status.set(IndexStatus::Error(msg));
        }
    });

    // Spawn background thread
    std::thread::spawn(move || {
        match index.write() {
            Ok(mut idx) => {
                match idx.build_index(&pak_paths) {
                    Ok(file_count) => {
                        let pak_count = idx.pak_count();
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
    let deep_search = state.deep_search.get();
    let active_filter = state.active_filter.get();
    let is_searching = state.is_searching;
    let results_signal = state.results;
    let show_progress = state.show_progress;

    // Set searching state
    is_searching.set(true);
    results_signal.set(Vec::new());

    // Show progress dialog for deep search
    if deep_search {
        show_progress.set(true);
        SEARCH_PROGRESS.reset();
        SEARCH_PROGRESS.set_active(true);
    }

    // Create action for sending final results back to UI thread
    let send_results = create_ext_action(Scope::new(), move |msg: SearchMessage| {
        SEARCH_PROGRESS.set_active(false);
        match msg {
            SearchMessage::Results(results) => {
                is_searching.set(false);
                show_progress.set(false);
                results_signal.set(results);
            }
            SearchMessage::Error(msg) => {
                is_searching.set(false);
                show_progress.set(false);
                tracing::error!("Search error: {}", msg);
            }
        }
    });

    // Spawn background thread
    std::thread::spawn(move || {
        let idx = match index.read() {
            Ok(idx) => idx,
            Err(e) => {
                send_results(SearchMessage::Error(format!("Failed to acquire lock: {}", e)));
                return;
            }
        };

        // Quick search: filename matching
        let matches = idx.search_filename(&query, active_filter);

        let mut results: Vec<SearchResult> = matches
            .iter()
            .take(MAX_QUICK_RESULTS)
            .map(|f| SearchResult::from_indexed_file(f))
            .collect();

        // Deep search: content matching (if enabled) - PARALLELIZED with PAK batching
        if deep_search {
            let deep_start = if *TIMING_ENABLED { Some(Instant::now()) } else { None };
            TIMING_STATS.reset();

            let searchable_files: Vec<_> = idx
                .all_entries()
                .filter(|f| f.file_type.is_searchable_text())
                .filter(|f| active_filter.map_or(true, |ft| f.file_type == ft))
                .take(MAX_DEEP_SCAN_FILES)
                .cloned()
                .collect();

            let total_files = searchable_files.len();
            SEARCH_PROGRESS.set(0, total_files, "Starting parallel search...".to_string());

            // Group files by PAK for cache efficiency
            // This avoids decompressing the same PAK file table multiple times
            let by_pak: std::collections::HashMap<PathBuf, Vec<IndexedFile>> = searchable_files
                .into_iter()
                .fold(std::collections::HashMap::new(), |mut acc, f| {
                    acc.entry(f.pak_file.clone()).or_default().push(f);
                    acc
                });

            let pak_count = by_pak.len();
            if *TIMING_ENABLED {
                eprintln!("[TIMING] Grouped {} files into {} PAKs", total_files, pak_count);
            }

            // Progress counter for parallel updates
            let progress_counter = AtomicUsize::new(0);

            // Parallel search: process each PAK batch, reusing file table within batch
            // Each parallel task processes one PAK with its own cache
            let content_results: Vec<SearchResult> = SEARCH_POOL.install(|| {
                by_pak
                    .par_iter()
                    .flat_map(|(pak_path, files)| {
                        // Create a cache for this PAK (capacity 1 since we only process one PAK per task)
                        let mut cache = PakReaderCache::new(1);

                        // Process all files in this PAK sequentially (to reuse cache)
                        files
                            .iter()
                            .flat_map(|file| {
                                // Update progress periodically
                                let current = progress_counter.fetch_add(1, Ordering::Relaxed);
                                if current % 100 == 0 {
                                    SEARCH_PROGRESS.set(current, total_files, file.name.clone());
                                }

                                // Search using cached PAK reader
                                search_file_content_cached(&mut cache, pak_path, file, &query)
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            });

            // Add content results (up to limit)
            let remaining = MAX_DEEP_RESULTS.saturating_sub(results.len());
            results.extend(content_results.into_iter().take(remaining));

            // Report timing stats
            if let Some(start) = deep_start {
                let elapsed = start.elapsed();
                eprintln!(
                    "[TIMING] Deep search completed in {:.2}s - {}",
                    elapsed.as_secs_f64(),
                    TIMING_STATS.report()
                );
            }
        }

        send_results(SearchMessage::Results(results));
    });
}

/// Load and search a single file's content (used in parallel search)
/// Returns search results for this file, or empty vec if no matches/error
#[allow(dead_code)]
fn search_file_content(file: &IndexedFile, query: &str) -> Vec<SearchResult> {
    let timing = *TIMING_ENABLED;

    // Load raw bytes from PAK
    let pak_start = if timing { Some(Instant::now()) } else { None };
    let raw_bytes = match PakOperations::read_file_bytes(&file.pak_file, &file.path) {
        Ok(bytes) => bytes,
        Err(_) => return Vec::new(),
    };
    if let Some(start) = pak_start {
        TIMING_STATS.add_pak_read(start.elapsed());
    }

    search_file_content_inner(&raw_bytes, file, query)
}

/// Load and search a single file's content using cached PAK reader
/// This is much faster when processing multiple files from the same PAK,
/// as the PAK file table is only decompressed once.
fn search_file_content_cached(
    cache: &mut PakReaderCache,
    pak_path: &PathBuf,
    file: &IndexedFile,
    query: &str,
) -> Vec<SearchResult> {
    let timing = *TIMING_ENABLED;

    // Load raw bytes from PAK using cache
    let pak_start = if timing { Some(Instant::now()) } else { None };
    let raw_bytes = match cache.read_file_bytes(pak_path, &file.path) {
        Ok(bytes) => bytes,
        Err(_) => return Vec::new(),
    };
    if let Some(start) = pak_start {
        TIMING_STATS.add_pak_read(start.elapsed());
    }

    search_file_content_inner(&raw_bytes, file, query)
}

/// Inner search logic shared by cached and non-cached versions
fn search_file_content_inner(raw_bytes: &[u8], file: &IndexedFile, query: &str) -> Vec<SearchResult> {
    let timing = *TIMING_ENABLED;

    TIMING_STATS.increment_files();
    let query_lower = query.to_lowercase();

    match file.file_type {
        FileType::Lsf => {
            // Search LSF directly without converting to LSX
            let parse_start = if timing { Some(Instant::now()) } else { None };
            let doc = match parse_lsf_bytes(raw_bytes) {
                Ok(d) => d,
                Err(_) => return Vec::new(),
            };
            if let Some(start) = parse_start {
                TIMING_STATS.add_lsf_parse(start.elapsed());
            }

            // Search the names table (node/attribute names)
            for name_list in &doc.names {
                for name in name_list {
                    if name.to_lowercase().contains(&query_lower) {
                        let snippet = if name.len() > 200 {
                            format!("{}...", &name[..200])
                        } else {
                            name.clone()
                        };
                        return vec![SearchResult::from_content_match(file, 0, snippet)];
                    }
                }
            }

            // Search attribute values
            for attr in &doc.attributes {
                let type_id = attr.type_info & 0x3F;
                let value_length = (attr.type_info >> 6) as usize;

                // Only search string-like types (strings, UUIDs, translated strings)
                if matches!(type_id, 20 | 21 | 22 | 23 | 28 | 29 | 30 | 31) {
                    if let Ok(value) = extract_value(&doc.values, attr.offset, value_length, type_id) {
                        if !value.is_empty() && value.to_lowercase().contains(&query_lower) {
                            let snippet = if value.len() > 200 {
                                format!("{}...", &value[..200])
                            } else {
                                value
                            };
                            return vec![SearchResult::from_content_match(file, 0, snippet)];
                        }
                    }
                }
            }

            Vec::new()
        }
        FileType::Lsx | FileType::Xml | FileType::Lsj | FileType::Json => {
            // Already text, just decode and search
            let text = match String::from_utf8(raw_bytes.to_vec()) {
                Ok(s) => s,
                Err(_) => return Vec::new(),
            };

            for (line_num, line) in text.lines().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    let snippet = if line.len() > 200 {
                        format!("{}...", &line[..200])
                    } else {
                        line.to_string()
                    };
                    return vec![SearchResult::from_content_match(file, line_num + 1, snippet)];
                }
            }

            Vec::new()
        }
        _ => Vec::new(),
    }
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
                        label(|| "Searching...")
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
