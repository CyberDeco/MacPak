//! Universal Editor Tab
//!
//! Text editor for LSX, LSJ, and LSF files using Floem's text_editor
//! (the same component that powers Lapce).

use floem::prelude::*;
use floem::views::text_editor;
use floem::views::editor::text::SimpleStyling;
use floem::text::Weight;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use regex::RegexBuilder;

use crate::state::{AppState, EditorState};

/// Pretty-print XML content with proper indentation
fn format_xml(content: &str) -> String {
    let mut result = String::new();
    let mut indent_level: i32 = 0;
    let indent_str = "    "; // 4 spaces

    let mut chars = content.chars().peekable();
    let mut in_tag = false;
    let mut current_tag = String::new();
    let mut text_content = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                let trimmed = text_content.trim();
                if !trimmed.is_empty() {
                    result.push_str(trimmed);
                }
                text_content.clear();

                in_tag = true;
                current_tag.clear();
                current_tag.push(ch);
            }
            '>' => {
                current_tag.push(ch);
                in_tag = false;

                let tag = current_tag.trim();

                if tag.starts_with("<?") || tag.starts_with("<!") {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push_str(tag);
                    result.push('\n');
                } else if tag.starts_with("</") {
                    indent_level = (indent_level - 1).max(0);
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                } else if tag.ends_with("/>") {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                } else {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                    indent_level += 1;
                }

                current_tag.clear();
            }
            _ => {
                if in_tag {
                    current_tag.push(ch);
                } else {
                    text_content.push(ch);
                }
            }
        }
    }

    if !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

/// Pretty-print JSON content with proper indentation
fn format_json(content: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| content.to_string()),
        Err(_) => content.to_string(),
    }
}

/// Search state for Find & Replace
struct SearchState {
    matches: Vec<(usize, usize)>,
    current_index: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            matches: Vec::new(),
            current_index: 0,
        }
    }
}

lazy_static::lazy_static! {
    static ref SEARCH_STATE: Arc<Mutex<SearchState>> = Arc::new(Mutex::new(SearchState::default()));
}

pub fn editor_tab(_app_state: AppState, editor_state: EditorState) -> impl IntoView {
    v_stack((
        editor_toolbar(editor_state.clone()),
        search_panel(editor_state.clone()),
        editor_content(editor_state.clone()),
        editor_status_bar(editor_state),
    ))
    .style(|s| s.width_full().height_full())
}

fn editor_toolbar(state: EditorState) -> impl IntoView {
    let state_open = state.clone();
    let state_save = state.clone();
    let state_save_as = state.clone();
    let state_validate = state.clone();
    let state_format = state.clone();

    h_stack((
        // File operations
        button("📂 Open").action(move || {
            open_file_dialog(state_open.clone());
        }),
        button("💾 Save")
            .disabled(move || !state_save.modified.get() || state_save.converted_from_lsf.get())
            .action({
                let state = state_save.clone();
                move || save_file(state.clone())
            }),
        button("Save As...").action(move || {
            save_file_as_dialog(state_save_as.clone());
        }),

        separator(),

        // Edit tools
        button("🔍 Find").action({
            let state = state.clone();
            move || {
                let visible = state.search_visible.get();
                state.search_visible.set(!visible);
            }
        }),
        line_number_toggle(state.show_line_numbers),
        button("📐 Format").action(move || {
            format_content(state_format.clone());
        }),
        button("✓ Validate").action(move || {
            validate_content(state_validate.clone());
        }),

        separator(),

        // Convert section
        label(|| "Convert:").style(|s| s.font_weight(Weight::BOLD).margin_right(8.0)),
        convert_button("LSX", state.clone()),
        convert_button("LSJ", state.clone()),
        convert_button("LSF", state.clone()),

        // Spacer
        empty().style(|s| s.flex_grow(1.0)),

        // Format badge
        format_badge(state.file_format),

        // Status message
        status_message(state.clone()),
    ))
    .style(|s| {
        s.width_full()
            .height(50.0)
            .padding(10.0)
            .gap(8.0)
            .items_center()
            .background(Color::rgb8(245, 245, 245))
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn status_message(state: EditorState) -> impl IntoView {
    dyn_container(
        move || (state.converted_from_lsf.get(), state.status_message.get()),
        move |(is_converted, msg)| {
            if !msg.is_empty() {
                label(move || msg.clone())
                    .style(|s| {
                        s.color(Color::rgb8(76, 175, 80))
                            .font_size(12.0)
                            .margin_left(8.0)
                    })
                    .into_any()
            } else if is_converted {
                label(|| "Converted from LSF - use Save As")
                    .style(|s| {
                        s.color(Color::rgb8(100, 100, 100))
                            .font_size(12.0)
                            .margin_left(8.0)
                    })
                    .into_any()
            } else {
                empty().into_any()
            }
        },
    )
}

fn convert_button(format: &'static str, state: EditorState) -> impl IntoView {
    let current_format = state.file_format;
    let has_content = state.content;
    let state_convert = state.clone();

    button(format)
        .disabled(move || {
            let f = current_format.get().to_uppercase();
            let empty = has_content.get().is_empty();
            f == format || empty
        })
        .action(move || {
            convert_file(state_convert.clone(), format);
        })
}

fn separator() -> impl IntoView {
    empty().style(|s| {
        s.width(1.0)
            .height(30.0)
            .background(Color::rgb8(200, 200, 200))
            .margin_horiz(4.0)
    })
}

fn line_number_toggle(show_line_numbers: RwSignal<bool>) -> impl IntoView {
    let label_text = move || {
        if show_line_numbers.get() {
            "# On"
        } else {
            "# Off"
        }
    };

    label(label_text)
        .style(move |s| {
            let is_active = show_line_numbers.get();
            let s = s
                .padding_horiz(8.0)
                .padding_vert(4.0)
                .border_radius(4.0)
                .font_size(12.0)
                .cursor(floem::style::CursorStyle::Pointer);

            if is_active {
                s.background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
            } else {
                s.background(Color::rgb8(230, 230, 230))
                    .color(Color::rgb8(80, 80, 80))
                    .hover(|s| s.background(Color::rgb8(210, 210, 210)))
            }
        })
        .on_click_stop(move |_| {
            show_line_numbers.set(!show_line_numbers.get());
        })
}

fn search_panel(state: EditorState) -> impl IntoView {
    let visible = state.search_visible;
    let search_text = state.search_text;
    let replace_text = state.replace_text;
    let case_sensitive = state.case_sensitive;
    let whole_words = state.whole_words;
    let use_regex = state.use_regex;
    let match_count = state.match_count;
    let current_match = state.current_match;
    let search_status = state.search_status;
    let content = state.content;

    // State clones for button actions
    let state_find_next = state.clone();
    let state_find_prev = state.clone();
    let state_replace = state.clone();
    let state_replace_all = state.clone();
    let state_close = state.clone();

    dyn_container(
        move || visible.get(),
        move |is_visible| {
            if !is_visible {
                return empty().into_any();
            }

            let state_find_next = state_find_next.clone();
            let state_find_prev = state_find_prev.clone();
            let state_replace = state_replace.clone();
            let state_replace_all = state_replace_all.clone();
            let state_close = state_close.clone();

            v_stack((
                // Find row
                h_stack((
                    label(|| "Find:").style(|s| s.width(60.0)),
                    text_input(search_text)
                        .placeholder("Search...")
                        .style(|s| {
                            s.width(250.0)
                                .padding(6.0)
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0)
                        })
                        .on_event_stop(floem::event::EventListener::KeyUp, move |_| {
                            perform_search(
                                content.get(),
                                search_text.get(),
                                case_sensitive.get(),
                                whole_words.get(),
                                use_regex.get(),
                                match_count,
                                current_match,
                                search_status,
                            );
                        }),
                    button("▲ Prev").action({
                        let state = state_find_prev.clone();
                        move || find_previous(state.clone())
                    }),
                    button("▼ Next").action({
                        let state = state_find_next.clone();
                        move || find_next(state.clone())
                    }),
                    // Match count display
                    label(move || {
                        let count = match_count.get();
                        let current = current_match.get();
                        if count == 0 {
                            "No matches".to_string()
                        } else {
                            format!("{} / {}", current + 1, count)
                        }
                    })
                    .style(|s| s.width(80.0).font_size(12.0).color(Color::rgb8(100, 100, 100))),
                    button("✕").action({
                        let state = state_close.clone();
                        move || {
                            state.search_visible.set(false);
                            state.match_count.set(0);
                            state.current_match.set(0);
                            state.search_status.set(String::new());
                        }
                    }),
                ))
                .style(|s| s.width_full().gap(8.0).items_center()),

                // Replace row
                h_stack((
                    label(|| "Replace:").style(|s| s.width(60.0)),
                    text_input(replace_text)
                        .placeholder("Replace with...")
                        .style(|s| {
                            s.width(250.0)
                                .padding(6.0)
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0)
                        }),
                    button("Replace").action({
                        let state = state_replace.clone();
                        move || replace_current(state.clone())
                    }),
                    button("Replace All").action({
                        let state = state_replace_all.clone();
                        move || replace_all(state.clone())
                    }),
                ))
                .style(|s| s.width_full().gap(8.0).items_center()),

                // Options row
                h_stack((
                    search_option_toggle("Aa", "Case sensitive", case_sensitive),
                    search_option_toggle("W", "Whole words", whole_words),
                    search_option_toggle(".*", "Use regex", use_regex),
                    empty().style(|s| s.flex_grow(1.0)),
                    label(move || search_status.get())
                        .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                ))
                .style(|s| s.width_full().gap(8.0).items_center()),
            ))
            .style(|s| {
                s.width_full()
                    .padding(12.0)
                    .gap(8.0)
                    .background(Color::rgb8(250, 250, 250))
                    .border_bottom(1.0)
                    .border_color(Color::rgb8(220, 220, 220))
            })
            .into_any()
        },
    )
}

fn search_option_toggle(
    label_text: &'static str,
    _tooltip: &'static str,
    signal: RwSignal<bool>,
) -> impl IntoView {
    button(label_text)
        .style(move |s| {
            let is_active = signal.get();
            let s = s
                .padding_horiz(8.0)
                .padding_vert(4.0)
                .border_radius(4.0)
                .font_size(12.0)
                .font_family("monospace".to_string());

            if is_active {
                s.background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
            } else {
                s.background(Color::rgb8(230, 230, 230))
                    .color(Color::rgb8(80, 80, 80))
                    .hover(|s| s.background(Color::rgb8(210, 210, 210)))
            }
        })
        .action(move || {
            signal.set(!signal.get());
        })
}

// ============================================================================
// Search Functions
// ============================================================================

fn perform_search(
    content: String,
    search_text: String,
    case_sensitive: bool,
    whole_words: bool,
    use_regex: bool,
    match_count: RwSignal<usize>,
    current_match: RwSignal<usize>,
    search_status: RwSignal<String>,
) {
    if search_text.is_empty() {
        match_count.set(0);
        current_match.set(0);
        search_status.set(String::new());
        if let Ok(mut state) = SEARCH_STATE.lock() {
            state.matches.clear();
            state.current_index = 0;
        }
        return;
    }

    // Build the regex pattern
    let pattern = if use_regex {
        search_text.clone()
    } else {
        regex::escape(&search_text)
    };

    let pattern = if whole_words {
        format!(r"\b{}\b", pattern)
    } else {
        pattern
    };

    match RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive)
        .build()
    {
        Ok(regex) => {
            let matches: Vec<(usize, usize)> = regex
                .find_iter(&content)
                .map(|m| (m.start(), m.end()))
                .collect();

            let count = matches.len();

            if let Ok(mut state) = SEARCH_STATE.lock() {
                state.matches = matches;
                state.current_index = 0;
            }

            match_count.set(count);
            current_match.set(0);
            search_status.set(String::new());
        }
        Err(e) => {
            match_count.set(0);
            current_match.set(0);
            search_status.set(format!("Invalid regex: {}", e));
            if let Ok(mut state) = SEARCH_STATE.lock() {
                state.matches.clear();
            }
        }
    }
}

fn find_next(state: EditorState) {
    let count = state.match_count.get();
    if count == 0 {
        return;
    }

    let current = state.current_match.get();
    let next = if current + 1 >= count { 0 } else { current + 1 };

    state.current_match.set(next);

    if let Ok(mut search_state) = SEARCH_STATE.lock() {
        search_state.current_index = next;
    }
}

fn find_previous(state: EditorState) {
    let count = state.match_count.get();
    if count == 0 {
        return;
    }

    let current = state.current_match.get();
    let prev = if current == 0 { count - 1 } else { current - 1 };

    state.current_match.set(prev);

    if let Ok(mut search_state) = SEARCH_STATE.lock() {
        search_state.current_index = prev;
    }
}

fn replace_current(state: EditorState) {
    let count = state.match_count.get();
    if count == 0 {
        state.search_status.set("No matches to replace".to_string());
        return;
    }

    let replace_with = state.replace_text.get();
    let current_idx = state.current_match.get();

    if let Ok(search_state) = SEARCH_STATE.lock() {
        if let Some(&(start, end)) = search_state.matches.get(current_idx) {
            let content = state.content.get();
            let mut new_content = String::new();
            new_content.push_str(&content[..start]);
            new_content.push_str(&replace_with);
            new_content.push_str(&content[end..]);

            state.content.set(new_content.clone());
            state.modified.set(true);

            // Re-run search to update matches
            drop(search_state);
            perform_search(
                new_content,
                state.search_text.get(),
                state.case_sensitive.get(),
                state.whole_words.get(),
                state.use_regex.get(),
                state.match_count,
                state.current_match,
                state.search_status,
            );

            state.search_status.set("Replaced 1 occurrence".to_string());
        }
    }
}

fn replace_all(state: EditorState) {
    let search_text = state.search_text.get();
    let replace_with = state.replace_text.get();
    let case_sensitive = state.case_sensitive.get();
    let whole_words = state.whole_words.get();
    let use_regex = state.use_regex.get();

    if search_text.is_empty() {
        state.search_status.set("Nothing to replace".to_string());
        return;
    }

    // Build the regex pattern
    let pattern = if use_regex {
        search_text.clone()
    } else {
        regex::escape(&search_text)
    };

    let pattern = if whole_words {
        format!(r"\b{}\b", pattern)
    } else {
        pattern
    };

    match RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive)
        .build()
    {
        Ok(regex) => {
            let content = state.content.get();
            let count = regex.find_iter(&content).count();

            if count == 0 {
                state.search_status.set("No matches found".to_string());
                return;
            }

            let new_content = regex.replace_all(&content, replace_with.as_str()).to_string();

            state.content.set(new_content.clone());
            state.modified.set(true);

            // Re-run search
            perform_search(
                new_content,
                state.search_text.get(),
                state.case_sensitive.get(),
                state.whole_words.get(),
                state.use_regex.get(),
                state.match_count,
                state.current_match,
                state.search_status,
            );

            state.search_status.set(format!("Replaced {} occurrences", count));
        }
        Err(e) => {
            state.search_status.set(format!("Invalid regex: {}", e));
        }
    }
}

fn format_badge(format: RwSignal<String>) -> impl IntoView {
    let format_text = move || {
        let f = format.get();
        if f.is_empty() {
            "No file".to_string()
        } else {
            format!("Format: {}", f)
        }
    };

    label(format_text).style(move |s| {
        let f = format.get();
        let (bg, border, text_color) = match f.to_uppercase().as_str() {
            "LSX" => (
                Color::rgb8(227, 242, 253),
                Color::rgb8(33, 150, 243),
                Color::rgb8(25, 118, 210),
            ),
            "LSJ" => (
                Color::rgb8(243, 229, 245),
                Color::rgb8(156, 39, 176),
                Color::rgb8(123, 31, 162),
            ),
            "LSF" => (
                Color::rgb8(255, 243, 224),
                Color::rgb8(255, 152, 0),
                Color::rgb8(245, 124, 0),
            ),
            _ => (
                Color::rgb8(240, 240, 240),
                Color::rgb8(200, 200, 200),
                Color::rgb8(100, 100, 100),
            ),
        };

        s.padding_horiz(12.0)
            .padding_vert(4.0)
            .background(bg)
            .border(1.0)
            .border_color(border)
            .border_radius(4.0)
            .color(text_color)
            .font_weight(Weight::SEMIBOLD)
    })
}

fn editor_content(state: EditorState) -> impl IntoView {
    let content = state.content;
    let modified = state.modified;
    let show_line_numbers = state.show_line_numbers;

    dyn_container(
        move || (content.get(), show_line_numbers.get()),
        move |(text, show_lines)| {
            let state_change = modified;
            text_editor(text)
                .styling(SimpleStyling::new())
                .editor_style(move |s| s.hide_gutter(!show_lines))
                .style(|s| s.width_full().flex_grow(1.0))
                .placeholder("Open a file to start editing...")
                .on_event_stop(floem::event::EventListener::KeyUp, move |_| {
                    state_change.set(true);
                })
        },
    )
    .style(|s| s.width_full().flex_grow(1.0))
}

fn editor_status_bar(state: EditorState) -> impl IntoView {
    h_stack((
        // File path
        label(move || {
            state
                .file_path
                .get()
                .unwrap_or_else(|| "No file loaded".to_string())
        })
        .style(|s| {
            s.color(Color::rgb8(100, 100, 100))
                .font_size(12.0)
                .text_ellipsis()
                .max_width(500.0)
        }),

        empty().style(|s| s.flex_grow(1.0)),

        // Modified indicator
        label(move || {
            if state.modified.get() {
                "● Modified"
            } else {
                ""
            }
            .to_string()
        })
        .style(|s| {
            s.color(Color::rgb8(255, 152, 0))
                .font_size(12.0)
                .margin_right(12.0)
        }),
    ))
    .style(|s| {
        s.width_full()
            .height(32.0)
            .padding_horiz(12.0)
            .items_center()
            .background(Color::rgb8(248, 248, 248))
            .border_top(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

// ============================================================================
// File Operations
// ============================================================================

fn open_file_dialog(state: EditorState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Open File")
        .add_filter("Larian Files", &["lsx", "lsf", "lsj"])
        .add_filter("LSX (XML)", &["lsx"])
        .add_filter("LSF (Binary)", &["lsf"])
        .add_filter("LSJ (JSON)", &["lsj"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.pick_file() {
        load_file(&path, state);
    }
}

pub fn load_file(path: &Path, state: EditorState) {
    let path_str = path.to_string_lossy().to_string();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_uppercase();

    state.file_format.set(ext.clone());
    state.file_path.set(Some(path_str.clone()));
    state.status_message.set("Loading...".to_string());

    match ext.as_str() {
        "LSX" => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Skip formatting for large files (>500KB)
                    let (formatted, was_large) = if content.len() > 500_000 {
                        (content, true)
                    } else {
                        (format_xml(&content), false)
                    };

                    state.content.set(formatted);
                    state.modified.set(false);
                    state.converted_from_lsf.set(false);

                    if was_large {
                        state.status_message.set("Large file - formatting skipped".to_string());
                    } else {
                        state.status_message.set("File loaded".to_string());
                    }
                }
                Err(e) => {
                    state.status_message.set(format!("Error: {}", e));
                }
            }
        }
        "LSJ" => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    let (formatted, was_large) = if content.len() > 500_000 {
                        (content, true)
                    } else {
                        (format_json(&content), false)
                    };

                    state.content.set(formatted);
                    state.modified.set(false);
                    state.converted_from_lsf.set(false);

                    if was_large {
                        state.status_message.set("Large file - formatting skipped".to_string());
                    } else {
                        state.status_message.set("File loaded".to_string());
                    }
                }
                Err(e) => {
                    state.status_message.set(format!("Error: {}", e));
                }
            }
        }
        "LSF" => {
            // Binary format - convert to LSX for display using MacLarian
            match MacLarian::formats::lsf::read_lsf(path) {
                Ok(lsf_doc) => {
                    match MacLarian::converter::to_lsx(&lsf_doc) {
                        Ok(content) => {
                            let (formatted, was_large) = if content.len() > 500_000 {
                                (content, true)
                            } else {
                                (format_xml(&content), false)
                            };

                            state.content.set(formatted);
                            state.modified.set(false);
                            state.converted_from_lsf.set(true);

                            if was_large {
                                state.status_message.set("Converted from LSF (large file)".to_string());
                            } else {
                                state.status_message.set("Converted from LSF - use Save As".to_string());
                            }
                        }
                        Err(e) => {
                            state.status_message.set(format!("Conversion error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    state.status_message.set(format!("Failed to read LSF: {}", e));
                }
            }
        }
        _ => {
            // Unknown format - try to read as text
            match fs::read_to_string(path) {
                Ok(content) => {
                    state.content.set(content);
                    state.modified.set(false);
                    state.converted_from_lsf.set(false);
                    state.status_message.set("File loaded".to_string());
                }
                Err(_) => {
                    state.content.set("[Binary file - cannot display]".to_string());
                    state.status_message.set("Binary file".to_string());
                }
            }
        }
    }
}

fn save_file(state: EditorState) {
    if let Some(path) = state.file_path.get() {
        let content = state.content.get();
        match fs::write(&path, &content) {
            Ok(_) => {
                state.modified.set(false);
                state.status_message.set("Saved".to_string());
            }
            Err(e) => {
                state.status_message.set(format!("Save failed: {}", e));
            }
        }
    } else {
        state.status_message.set("No file loaded".to_string());
    }
}

fn save_file_as_dialog(state: EditorState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Save As")
        .add_filter("LSX (XML)", &["lsx"])
        .add_filter("LSJ (JSON)", &["lsj"])
        .add_filter("All Files", &["*"]);

    if let Some(path) = dialog.save_file() {
        let content = state.content.get();
        match fs::write(&path, &content) {
            Ok(_) => {
                let path_str = path.to_string_lossy().to_string();
                state.file_path.set(Some(path_str));
                state.modified.set(false);
                state.converted_from_lsf.set(false);

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    state.file_format.set(ext.to_uppercase());
                }

                state.status_message.set("Saved".to_string());
            }
            Err(e) => {
                state.status_message.set(format!("Save failed: {}", e));
            }
        }
    }
}

fn format_content(state: EditorState) {
    let content = state.content.get();
    let format = state.file_format.get().to_uppercase();

    if content.is_empty() {
        state.status_message.set("No content to format".to_string());
        return;
    }

    let formatted = match format.as_str() {
        "LSX" | "LSF" => format_xml(&content),
        "LSJ" => format_json(&content),
        _ => content,
    };

    state.content.set(formatted);
    state.modified.set(true);
    state.status_message.set("Content formatted".to_string());
}

fn validate_content(state: EditorState) {
    let content = state.content.get();
    let format = state.file_format.get().to_uppercase();

    if content.is_empty() {
        state.status_message.set("No content to validate".to_string());
        return;
    }

    let result = match format.as_str() {
        "LSX" | "LSF" => {
            match roxmltree::Document::parse(&content) {
                Ok(_) => Ok("Valid XML structure"),
                Err(e) => Err(format!("Invalid XML: {}", e)),
            }
        }
        "LSJ" => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => Ok("Valid JSON structure"),
                Err(e) => Err(format!("Invalid JSON: {}", e)),
            }
        }
        _ => Ok("Unknown format - skipped validation"),
    };

    match result {
        Ok(msg) => state.status_message.set(msg.to_string()),
        Err(msg) => state.status_message.set(msg),
    }
}

fn convert_file(state: EditorState, target_format: &str) {
    let source_path = match state.file_path.get() {
        Some(p) => p,
        None => {
            state.status_message.set("No file loaded".to_string());
            return;
        }
    };

    let current_format = state.file_format.get().to_lowercase();
    let target = target_format.to_lowercase();

    // Show save dialog for converted file
    let dialog = rfd::FileDialog::new()
        .set_title(&format!("Save as {} File", target.to_uppercase()))
        .add_filter(&target.to_uppercase(), &[&target]);

    if let Some(dest_path) = dialog.save_file() {
        let dest = dest_path.to_string_lossy().to_string();

        state.status_message.set("Converting...".to_string());

        // Perform conversion
        let result = match (current_format.as_str(), target.as_str()) {
            ("lsf", "lsx") => MacLarian::converter::lsf_to_lsx(&source_path, &dest),
            ("lsx", "lsf") => MacLarian::converter::lsx_to_lsf(&source_path, &dest),
            ("lsx", "lsj") => MacLarian::converter::lsx_to_lsj(&source_path, &dest),
            ("lsj", "lsx") => MacLarian::converter::lsj_to_lsx(&source_path, &dest),
            ("lsf", "lsj") => MacLarian::converter::lsf_to_lsj(&source_path, &dest),
            ("lsj", "lsf") => MacLarian::converter::lsj_to_lsf(&source_path, &dest),
            _ => {
                state.status_message.set(format!(
                    "Unsupported conversion: {} to {}",
                    current_format, target
                ));
                return;
            }
        };

        match result {
            Ok(_) => {
                state.status_message.set(format!("Converted to {}", target.to_uppercase()));
            }
            Err(e) => {
                state.status_message.set(format!("Conversion failed: {}", e));
            }
        }
    }
}
