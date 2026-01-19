//! "Show All Matches" dialog for expanded context view

use std::panic;

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem::text::Weight;
use floem_reactive::{create_effect, Scope};
use maclarian::converter::lsf_lsx_lsj::to_lsx;
use maclarian::formats::lsf::parse_lsf_bytes;
use maclarian::pak::PakOperations;

use crate::gui::state::{SearchResult, SearchState};

/// A single match with surrounding context
#[derive(Clone, Debug)]
struct MatchWithContext {
    /// Line number (1-indexed)
    line_number: usize,
    /// Lines before the match
    context_before: Vec<String>,
    /// The matched line
    matched_line: String,
    /// Lines after the match
    context_after: Vec<String>,
}

/// Dialog showing all matches in a file with expanded context
pub fn all_matches_dialog(state: SearchState) -> impl IntoView {
    let show = state.show_all_matches;
    let file = state.all_matches_file;
    let query = state.query;

    // Signals for loaded matches
    let matches: RwSignal<Vec<MatchWithContext>> = RwSignal::new(Vec::new());
    let is_loading = RwSignal::new(false);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    // Track which file path we've already loaded to prevent re-loading
    let loaded_path: RwSignal<Option<String>> = RwSignal::new(None);

    // Load matches when dialog opens with a new file
    let file_for_effect = file;
    let query_for_effect = query;
    let show_for_effect = show;

    create_effect(move |_| {
        let visible = show_for_effect.get();
        let current_file = file_for_effect.get();
        let current_query = query_for_effect.get();

        if !visible {
            // Dialog closed - reset loaded path so we reload on next open
            loaded_path.set(None);
            return;
        }

        if let Some(ref file) = current_file {
            let file_path = file.path.clone();
            let already_loaded = loaded_path.get().as_ref() == Some(&file_path);

            // Only load if we haven't loaded this file yet and not currently loading
            if !already_loaded && !is_loading.get_untracked() {
                loaded_path.set(Some(file_path));
                is_loading.set(true);
                error_msg.set(None);
                matches.set(Vec::new());

                load_all_matches(file.clone(), current_query, matches, is_loading, error_msg);
            }
        }
    });

    dyn_container(
        move || show.get(),
        move |visible| {
            if !visible {
                return empty().into_any();
            }

            let current_file = file.get();
            let file_name = current_file
                .as_ref()
                .map(|f| f.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let file_path = current_file
                .as_ref()
                .map(|f| f.path.clone())
                .unwrap_or_default();

            container(
                v_stack((
                    // Header
                    h_stack((
                        v_stack((
                            label(move || format!("All Matches: {}", file_name.clone()))
                                .style(|s| s.font_size(16.0).font_weight(Weight::BOLD)),
                            label(move || file_path.clone())
                                .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                        )),
                        empty().style(|s| s.flex_grow(1.0)),
                        button("Close")
                            .style(|s| {
                                s.padding_horiz(16.0)
                                    .padding_vert(6.0)
                                    .background(Color::rgb8(100, 100, 100))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                            })
                            .action(move || {
                                show.set(false);
                                file.set(None);
                                matches.set(Vec::new());
                            }),
                    ))
                    .style(|s| s.width_full().margin_bottom(16.0)),

                    // Content area
                    dyn_container(
                        move || (is_loading.get(), error_msg.get(), matches.get().len()),
                        move |(loading, error, match_count)| {
                            if loading {
                                label(|| "Loading matches...")
                                    .style(|s| {
                                        s.padding(40.0)
                                            .color(Color::rgb8(100, 100, 100))
                                    })
                                    .into_any()
                            } else if let Some(err) = error {
                                label(move || format!("Error: {}", err))
                                    .style(|s| {
                                        s.padding(20.0)
                                            .color(Color::rgb8(200, 50, 50))
                                    })
                                    .into_any()
                            } else if match_count == 0 {
                                label(|| "No matches found in file content")
                                    .style(|s| {
                                        s.padding(40.0)
                                            .color(Color::rgb8(100, 100, 100))
                                    })
                                    .into_any()
                            } else {
                                // Show matches
                                let current_matches = matches.get();
                                scroll(
                                    v_stack_from_iter(
                                        current_matches.into_iter().enumerate().map(|(i, m)| {
                                            match_row(m, i)
                                        })
                                    )
                                    .style(|s| s.width_full().gap(12.0))
                                )
                                .scroll_style(|s| s.handle_thickness(6.0))
                                .style(|s| {
                                    s.width_full()
                                        .max_height(400.0)
                                        .flex_grow(1.0)
                                })
                                .into_any()
                            }
                        },
                    ),

                    // Match count footer
                    dyn_container(
                        move || matches.get().len(),
                        move |count| {
                            if count > 0 {
                                label(move || format!("{} matches found", count))
                                    .style(|s| {
                                        s.margin_top(12.0)
                                            .font_size(12.0)
                                            .color(Color::rgb8(100, 100, 100))
                                    })
                                    .into_any()
                            } else {
                                empty().into_any()
                            }
                        },
                    ),
                ))
                .style(|s| {
                    s.padding(24.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .width(900.0)
                        .max_height(550.0)
                        .box_shadow_blur(20.0)
                        .box_shadow_color(Color::rgba8(0, 0, 0, 50))
                }),
            )
            .into_any()
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

/// Display a single match with context
fn match_row(m: MatchWithContext, index: usize) -> impl IntoView {
    let line_num = m.line_number;
    let context_before = m.context_before.clone();
    let matched_line = m.matched_line.clone();
    let context_after = m.context_after.clone();

    v_stack((
        // Match header
        h_stack((
            label(move || format!("Match #{}", index + 1))
                .style(|s| s.font_weight(Weight::SEMIBOLD).font_size(12.0)),
            label(move || format!("Line {}", line_num))
                .style(|s| {
                    s.font_size(11.0)
                        .color(Color::rgb8(100, 100, 100))
                        .margin_left(8.0)
                }),
        )),

        // Context display with horizontal scroll for long lines
        scroll(
            v_stack((
                // Lines before
                v_stack_from_iter(
                    context_before.into_iter().enumerate().map(move |(i, line)| {
                        let ln = line_num.saturating_sub(3 - i);
                        context_line(ln, line, false)
                    })
                ),
                // Matched line (highlighted)
                context_line(line_num, matched_line, true),
                // Lines after
                v_stack_from_iter(
                    context_after.into_iter().enumerate().map(move |(i, line)| {
                        let ln = line_num + 1 + i;
                        context_line(ln, line, false)
                    })
                ),
            ))
            .style(|s| s.padding(8.0))
        )
        .scroll_style(|s| s.handle_thickness(4.0))
        .style(|s| {
            s.width_full()
                .max_height(200.0)
                .background(Color::rgb8(248, 248, 248))
                .border_radius(4.0)
                .margin_top(4.0)
        }),
    ))
    .style(|s| {
        s.width_full()
            .padding(8.0)
            .border(1.0)
            .border_color(Color::rgb8(230, 230, 230))
            .border_radius(4.0)
    })
}

/// Display a single context line with line number
fn context_line(line_num: usize, text: String, is_match: bool) -> impl IntoView {
    h_stack((
        // Line number
        label(move || format!("{:4}", line_num))
            .style(|s| {
                s.font_size(11.0)
                    .font_family("monospace".to_string())
                    .color(Color::rgb8(150, 150, 150))
                    .width(40.0)
                    .flex_shrink(0.0)
            }),
        // Line content - no truncation, allows horizontal scroll
        label(move || text.clone())
            .style(move |s| {
                let s = s
                    .font_size(12.0)
                    .font_family("monospace".to_string())
                    .flex_shrink(0.0);
                if is_match {
                    s.background(Color::rgb8(255, 255, 200))
                        .font_weight(Weight::MEDIUM)
                } else {
                    s.color(Color::rgb8(80, 80, 80))
                }
            }),
    ))
}

/// Load all matches from a file
fn load_all_matches(
    result: SearchResult,
    query: String,
    matches: RwSignal<Vec<MatchWithContext>>,
    is_loading: RwSignal<bool>,
    error_msg: RwSignal<Option<String>>,
) {
    let pak_path = result.pak_path.clone();
    let file_path = result.path.clone();

    let send = create_ext_action(Scope::new(), move |found: Result<Vec<MatchWithContext>, String>| {
        is_loading.set(false);
        match found {
            Ok(results) => {
                matches.set(results);
            }
            Err(e) => {
                error_msg.set(Some(e));
            }
        }
    });

    std::thread::spawn(move || {
        // Wrap in catch_unwind to handle panics gracefully
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            // Normalize path separators (PAK files use forward slashes internally)
            let normalized_path = file_path.replace('\\', "/");

            // Read file content from PAK
            let content = match PakOperations::read_file_bytes(&pak_path, &normalized_path) {
                Ok(bytes) => {
                    // Try to interpret as UTF-8, or convert LSF to LSX for proper line numbers
                    match String::from_utf8(bytes.clone()) {
                        Ok(text) => text,
                        Err(_) => {
                            // Convert LSF binary to LSX XML for accurate line numbers
                            match parse_lsf_bytes(&bytes) {
                                Ok(doc) => to_lsx(&doc).unwrap_or_default(),
                                Err(_) => String::new(),
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to read file '{}' from PAK '{}': {}",
                        normalized_path, pak_path.display(), e));
                }
            };

            if content.is_empty() {
                return Ok(Vec::new());
            }

            // Find all matches with context (3 lines before/after)
            let lines: Vec<&str> = content.lines().collect();
            let query_lower = query.to_lowercase();
            let mut found = Vec::new();

            for (i, line) in lines.iter().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    let start = i.saturating_sub(3);
                    let end = (i + 4).min(lines.len());

                    found.push(MatchWithContext {
                        line_number: i + 1,
                        context_before: lines[start..i].iter().map(|s| decode_xml_entities(s)).collect(),
                        matched_line: decode_xml_entities(line),
                        context_after: lines.get(i + 1..end).unwrap_or(&[]).iter().map(|s| decode_xml_entities(s)).collect(),
                    });
                }
            }

            Ok(found)
        }));

        match result {
            Ok(inner_result) => send(inner_result),
            Err(_) => send(Err("Internal error: thread panicked while loading matches".to_string())),
        }
    });
}

/// Decode XML entities to their character equivalents
fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}
