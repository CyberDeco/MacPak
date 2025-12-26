//! UI components for the editor

use floem::prelude::*;
use floem::text::Weight;
use floem::views::text_editor;

use crate::state::EditorState;
use super::operations::{
    convert_file, format_content, open_file_dialog, save_file, save_file_as_dialog,
    validate_content,
};
use super::search::{find_next, find_previous, perform_search, replace_all, replace_current};
use super::syntax::SyntaxStyling;

pub fn editor_toolbar(state: EditorState) -> impl IntoView {
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

pub fn search_panel(state: EditorState) -> impl IntoView {
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

pub fn editor_content(state: EditorState) -> impl IntoView {
    let content = state.content;
    let modified = state.modified;
    let show_line_numbers = state.show_line_numbers;
    let file_format = state.file_format;

    dyn_container(
        move || (content.get(), show_line_numbers.get(), file_format.get()),
        move |(text, show_lines, format)| {
            let state_change = modified;
            // Create syntax highlighting based on file format
            let styling = SyntaxStyling::new(&text, &format);

            text_editor(text)
                .styling(styling)
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

pub fn editor_status_bar(state: EditorState) -> impl IntoView {
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
