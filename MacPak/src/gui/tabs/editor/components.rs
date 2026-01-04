//! UI components for the editor

use std::time::Duration;

use floem::action::exec_after;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::prelude::*;
use floem::text::Weight;
use floem::views::{text_editor_keys, checkbox};
use floem::views::editor::command::CommandExecuted;
use floem::views::editor::keypress::{default_key_handler, key::KeyInput, press::KeyPress};

use crate::gui::state::{EditorTab, EditorTabsState};
use super::operations::{
    convert_file, open_file_dialog, save_file, save_file_as_dialog, validate_content,
};
use super::search::{find_next, find_previous, perform_search, replace_all, replace_current};
use super::syntax::SyntaxStyling;

/// Common toolbar button style for consistent height
fn toolbar_button_style(s: floem::style::Style) -> floem::style::Style {
    s.min_height(0.0)
        .height(22.0)
        .max_height(22.0)
        .padding_horiz(6.0)
        .padding_vert(2.0)
        .items_center()
        .justify_center()
}

pub fn editor_toolbar(tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_open = tabs_state.clone();
    let tabs_state_save_check = tabs_state.clone();
    let tabs_state_save_action = tabs_state.clone();
    let tabs_state_save_as = tabs_state.clone();
    let tabs_state_validate = tabs_state.clone();
    let tabs_state_find = tabs_state.clone();
    let tabs_state_lsx = tabs_state.clone();
    let tabs_state_lsj = tabs_state.clone();
    let tabs_state_lsf = tabs_state.clone();
    let _tabs_state_meta = tabs_state.clone();
    let tabs_state_xml = tabs_state.clone();
    let tabs_state_loca = tabs_state.clone();

    h_stack((
        // File operations group
        h_stack((
            button("📂 Open")
                .style(toolbar_button_style)
                .action(move || {
                    open_file_dialog(tabs_state_open.clone());
                }),
            // button("Generate meta.lsx")
            //     .style(toolbar_button_style)
            //     .action(move || {
            //         tabs_state_meta.show_meta_dialog.set(true);
            //     }),
            button("💾 Save")
                .style(toolbar_button_style)
                .disabled(move || {
                    tabs_state_save_check.active_tab().map_or(true, |tab| {
                        !tab.modified.get() || tab.converted_from_lsf.get()
                    })
                })
                .action(move || {
                    if let Some(tab) = tabs_state_save_action.active_tab() {
                        save_file(tab);
                    }
                }),
            button("💾 Save As...")
                .style(toolbar_button_style)
                .action(move || {
                    if let Some(tab) = tabs_state_save_as.active_tab() {
                        save_file_as_dialog(tab);
                    }
                }),
        ))
        .style(|s| s.gap(8.0).items_center()),
        separator(),
        // Edit tools group
        h_stack((
            button("🔍 Find")
                .style(toolbar_button_style)
                .action({
                    move || {
                        if let Some(tab) = tabs_state_find.active_tab() {
                            let visible = tab.search_visible.get();
                            tab.search_visible.set(!visible);
                        }
                    }
                }),
            button("✓ Validate")
                .style(toolbar_button_style)
                .action(move || {
                    if let Some(tab) = tabs_state_validate.active_tab() {
                        validate_content(tab, tabs_state.status_message);
                    }
                }),
            line_number_toggle(tabs_state.show_line_numbers),
        ))
        .style(|s| s.gap(8.0).items_center()),
        // Spacer
        empty().style(|s| s.flex_grow(1.0)),
        separator(),
        // LSF/LSX/LSJ Convert section (disabled for LOCA files)
        h_stack((
            // label(|| "Convert:").style(|s| s.font_weight(Weight::BOLD).margin_right(8.0)),
            convert_button_lsf_group("LSX", tabs_state_lsx),
            convert_button_lsf_group("LSJ", tabs_state_lsj),
            convert_button_lsf_group("LSF", tabs_state_lsf),
        ))
        .style(|s| s.gap(8.0).items_center()),
        separator(),
        // LOCA/XML Convert section (disabled for LSF/LSX/LSJ files)
        h_stack((
            // label(|| "Loca:").style(|s| s.font_weight(Weight::BOLD).margin_right(8.0)),
            convert_button_loca_group("XML", tabs_state_xml),
            convert_button_loca_group("LOCA", tabs_state_loca),
        ))
        .style(|s| s.gap(8.0).items_center()),
        separator(),
        // Format badge
        format_badge(tabs_state.clone()),
        // Status message
        status_message(tabs_state.status_message),
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

fn status_message(status: RwSignal<String>) -> impl IntoView {
    dyn_container(
        move || status.get(),
        move |msg| {
            if !msg.is_empty() {
                label(move || msg.clone())
                    .style(|s| {
                        s.color(Color::rgb8(76, 175, 80))
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

/// Convert button for LSF/LSX/LSJ group - disabled for LOCA files
fn convert_button_lsf_group(format: &'static str, tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_check = tabs_state.clone();
    let tabs_state_action = tabs_state.clone();

    button(format)
        .style(toolbar_button_style)
        .disabled(move || {
            tabs_state_check.active_tab().map_or(true, |tab| {
                let f = tab.file_format.get().to_uppercase();
                let empty = tab.content.get().is_empty();
                // Disable if: current format matches, empty, OR it's a LOCA-related file
                f == format || empty || matches!(f.as_str(), "LOCA" | "XML")
            })
        })
        .action(move || {
            if let Some(tab) = tabs_state_action.active_tab() {
                convert_file(tab, format);
            }
        })
}

/// Convert button for LOCA/XML group - disabled for LSF/LSX/LSJ files
fn convert_button_loca_group(format: &'static str, tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_check = tabs_state.clone();
    let tabs_state_action = tabs_state.clone();

    button(format)
        .style(toolbar_button_style)
        .disabled(move || {
            tabs_state_check.active_tab().map_or(true, |tab| {
                let f = tab.file_format.get().to_uppercase();
                let empty = tab.content.get().is_empty();
                // Disable if: current format matches, empty, OR it's a LSF-related file
                f == format || empty || matches!(f.as_str(), "LSF" | "LSX" | "LSJ")
            })
        })
        .action(move || {
            if let Some(tab) = tabs_state_action.active_tab() {
                convert_file(tab, format);
            }
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
    h_stack((
        checkbox(move || show_line_numbers.get())
            .on_update(move |checked| {
                show_line_numbers.set(checked);
            })
            .style(move |s| {
                s.margin_right(8.0)                                                                    
            }),
        label(|| "Show Line Numbers")
            .style(|s| {
                s.font_size(12.0)
                    .cursor(floem::style::CursorStyle::Pointer)
            }),
    ))
    .on_click_stop(move |_| {
        show_line_numbers.set(!show_line_numbers.get());
    })
    .style(|s| {
        s.padding_horiz(8.0)
            .padding_vert(4.0)
            .items_center()
    })
}

pub fn search_panel(tab: EditorTab) -> impl IntoView {
    let visible = tab.search_visible;
    let search_text = tab.search_text;
    let replace_text = tab.replace_text;
    let case_sensitive = tab.case_sensitive;
    let whole_words = tab.whole_words;
    let use_regex = tab.use_regex;
    let match_count = tab.match_count;
    let current_match = tab.current_match;
    let search_status = tab.search_status;
    let content = tab.content;

    // State clones for button actions
    let tab_find_next = tab.clone();
    let tab_find_prev = tab.clone();
    let tab_replace = tab.clone();
    let tab_replace_all = tab.clone();
    let tab_close = tab.clone();

    dyn_container(
        move || visible.get(),
        move |is_visible| {
            if !is_visible {
                return empty().into_any();
            }

            let tab_find_next = tab_find_next.clone();
            let tab_find_prev = tab_find_prev.clone();
            let tab_replace = tab_replace.clone();
            let tab_replace_all = tab_replace_all.clone();
            let tab_close = tab_close.clone();

            // Clone tab for Enter key handler
            let tab_find_enter = tab_find_next.clone();

            v_stack((
                // Find row
                h_stack((
                    label(|| "Find:").style(|s| s.width(60.0)),
                    {
                        let input = text_input(search_text)
                            .placeholder("Search...")
                            .style(|s| {
                                s.width(250.0)
                                    .padding(6.0)
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                            });
                        // Auto-focus the search input when panel opens
                        let input_id = input.id();
                        input_id.request_focus();
                        input
                    }
                        .on_event_stop(EventListener::KeyDown, {
                            let tab = tab_find_enter.clone();
                            move |e| {
                                if let Event::KeyDown(key_event) = e {
                                    // CMD+F / Ctrl+F closes the search panel
                                    let is_cmd_or_ctrl = key_event.modifiers.contains(Modifiers::META)
                                        || key_event.modifiers.contains(Modifiers::CONTROL);
                                    let is_f_key = matches!(
                                        &key_event.key.logical_key,
                                        Key::Character(c) if c.as_str().eq_ignore_ascii_case("f")
                                    );
                                    if is_cmd_or_ctrl && is_f_key {
                                        visible.set(false);
                                        return;
                                    }
                                    // Enter key jumps to next match
                                    if key_event.key.logical_key == Key::Named(NamedKey::Enter) {
                                        find_next(tab.clone());
                                    }
                                }
                            }
                        })
                        .on_event_cont(EventListener::KeyUp, move |e| {
                            // Don't re-search on Enter release (it would reset the match index)
                            if let Event::KeyUp(key_event) = e {
                                if key_event.key.logical_key == Key::Named(NamedKey::Enter) {
                                    return;
                                }
                            }
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
                        let tab = tab_find_prev.clone();
                        move || find_previous(tab.clone())
                    }),
                    button("▼ Next").action({
                        let tab = tab_find_next.clone();
                        move || find_next(tab.clone())
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
                        let tab = tab_close.clone();
                        move || {
                            tab.search_visible.set(false);
                            tab.match_count.set(0);
                            tab.current_match.set(0);
                            tab.search_status.set(String::new());
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
                        })
                        .on_event_stop(EventListener::KeyDown, move |e| {
                            // CMD+F / Ctrl+F closes the search panel
                            if let Event::KeyDown(key_event) = e {
                                let is_cmd_or_ctrl = key_event.modifiers.contains(Modifiers::META)
                                    || key_event.modifiers.contains(Modifiers::CONTROL);
                                let is_f_key = matches!(
                                    &key_event.key.logical_key,
                                    Key::Character(c) if c.as_str().eq_ignore_ascii_case("f")
                                );
                                if is_cmd_or_ctrl && is_f_key {
                                    visible.set(false);
                                }
                            }
                        }),
                    button("Replace").action({
                        let tab = tab_replace.clone();
                        move || replace_current(tab.clone())
                    }),
                    button("Replace All").action({
                        let tab = tab_replace_all.clone();
                        move || replace_all(tab.clone())
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
            .on_event_cont(EventListener::KeyDown, move |e| {
                // CMD+F / Ctrl+F closes the search panel
                if let Event::KeyDown(key_event) = e {
                    let is_cmd_or_ctrl = key_event.modifiers.contains(Modifiers::META)
                        || key_event.modifiers.contains(Modifiers::CONTROL);
                    let is_f_key = matches!(
                        &key_event.key.logical_key,
                        Key::Character(c) if c.as_str().eq_ignore_ascii_case("f")
                    );
                    if is_cmd_or_ctrl && is_f_key {
                        visible.set(false);
                    }
                }
            })
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

fn format_badge(tabs_state: EditorTabsState) -> impl IntoView {
    dyn_container(
        move || tabs_state.active_tab().map(|tab| tab.file_format.get()),
        move |maybe_format| {
            let format = maybe_format.unwrap_or_default();
            let format_text = if format.is_empty() {
                "No file".to_string()
            } else {
                format!("Format: {}", format)
            };

            let (bg, border, text_color) = match format.to_uppercase().as_str() {
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
                "LOCA" => (
                    Color::rgb8(232, 245, 233),
                    Color::rgb8(76, 175, 80),
                    Color::rgb8(56, 142, 60),
                ),
                _ => (
                    Color::rgb8(240, 240, 240),
                    Color::rgb8(200, 200, 200),
                    Color::rgb8(100, 100, 100),
                ),
            };

            label(move || format_text.clone())
                .style(move |s| {
                    s.padding_horiz(12.0)
                        .padding_vert(4.0)
                        .background(bg)
                        .border(1.0)
                        .border_color(border)
                        .border_radius(4.0)
                        .color(text_color)
                        .font_weight(Weight::SEMIBOLD)
                })
                .into_any()
        },
    )
}

pub fn editor_content(tab: EditorTab, tabs_state: EditorTabsState, show_line_numbers: RwSignal<bool>) -> impl IntoView {
    let content = tab.content;
    let modified = tab.modified;
    let file_format = tab.file_format;
    let file_path = tab.file_path;
    let goto_offset = tab.goto_offset;
    let search_visible = tab.search_visible;
    let converted_from_lsf = tab.converted_from_lsf;
    let tab_for_save = tab.clone();
    let tabs_state_for_open = tabs_state.clone();

    // Recreate editor when file changes or line number toggle changes
    // Width/resize is handled automatically by floem's reactive viewport tracking
    dyn_container(
        move || (file_path.get(), show_line_numbers.get(), file_format.get()),
        move |(_path, show_lines, format)| {
            let text = content.get();
            let state_change = modified;
            // Create syntax highlighting based on file format
            let styling = SyntaxStyling::new(&text, &format);

            // Clone tab and state for the key handler
            let tab_for_keys = tab_for_save.clone();
            let tabs_state_for_keys = tabs_state_for_open.clone();

            // Custom key handler that intercepts shortcuts before the default handler
            let key_handler = move |editor_sig, keypress: &KeyPress, mods: Modifiers| {
                // Check for CMD/Ctrl modifier
                let is_cmd_or_ctrl = mods.meta() || mods.control();

                if is_cmd_or_ctrl {
                    if let KeyInput::Keyboard(Key::Character(c), _) = &keypress.key {
                        // CMD+F - Find
                        if c.as_str().eq_ignore_ascii_case("f") {
                            search_visible.set(!search_visible.get());
                            return CommandExecuted::Yes;
                        }
                        // CMD+S - Save
                        if c.as_str().eq_ignore_ascii_case("s") {
                            if modified.get() {
                                if converted_from_lsf.get() {
                                    let tab_clone = tab_for_keys.clone();
                                    exec_after(Duration::from_millis(50), move |_| {
                                        save_file_as_dialog(tab_clone);
                                    });
                                } else {
                                    save_file(tab_for_keys.clone());
                                }
                            }
                            return CommandExecuted::Yes;
                        }
                        // CMD+O - Open
                        if c.as_str().eq_ignore_ascii_case("o") {
                            let tabs_clone = tabs_state_for_keys.clone();
                            exec_after(Duration::from_millis(50), move |_| {
                                open_file_dialog(tabs_clone);
                            });
                            return CommandExecuted::Yes;
                        }
                    }
                }

                // Fall through to default handler (handles CMD+Z, CMD+Shift+Z, etc.)
                default_key_handler(editor_sig)(keypress, mods)
            };

            text_editor_keys(text, key_handler)
                .styling(styling)
                .editor_style(move |s| s.hide_gutter(!show_lines))
                .style(move |s| {
                    let s = s.width_full().height_full();
                    if show_lines {
                        s
                    } else {
                        s.padding_left(12.0)
                    }
                })
                .placeholder("Open a file to start editing...")
                .with_editor(move |editor| {
                    let cursor = editor.cursor;

                    // Workaround for floem text_editor not re-wrapping on width expansion.
                    // Watch parent_size (which tracks the container) and update viewport
                    // when it expands. Floem's internal effect will handle the rest.
                    let parent_size = editor.parent_size;
                    let viewport = editor.viewport;
                    floem::reactive::create_effect(move |prev_width: Option<f64>| {
                        let current_width = parent_size.get().width();
                        if let Some(prev) = prev_width {
                            if current_width > prev {
                                // Container expanded - update viewport to match parent
                                let mut vp = viewport.get();
                                vp = vp.with_size((current_width, vp.height()));
                                viewport.set(vp);
                            }
                        }
                        current_width
                    });

                    // Set up reactive effect to jump to offset when goto_offset changes
                    let editor_clone = editor.clone();
                    floem::reactive::create_effect(move |_| {
                        if let Some(offset) = goto_offset.get() {
                            // Move cursor to the match offset
                            cursor.update(|c| c.set_offset(offset, false, false));
                            // Center the view on the new cursor position
                            editor_clone.center_window();
                            // Clear the goto_offset so it doesn't keep triggering
                            goto_offset.set(None);
                        }
                    });

                    // Sync editor content back to tab.content when document changes
                    let doc = editor.doc();
                    let cache_rev = doc.cache_rev();
                    let editor_for_sync = editor.clone();
                    floem::reactive::create_effect(move |prev_rev: Option<u64>| {
                        let current_rev = cache_rev.get();
                        // Only sync if revision changed (actual edit occurred)
                        if prev_rev.is_some() && prev_rev != Some(current_rev) {
                            let new_text = editor_for_sync.doc().text().to_string();
                            content.set(new_text);
                            state_change.set(true);
                        }
                        current_rev
                    });
                })
                .style(|s| s.size_full().flex_grow(1.0))
        },
    )
    .style(|s| s.size_full().flex_grow(1.0))
}

pub fn editor_status_bar(tabs_state: EditorTabsState) -> impl IntoView {
    dyn_container(
        move || tabs_state.active_tab(),
        move |maybe_tab| {
            if let Some(tab) = maybe_tab {
                let file_path = tab.file_path;
                let modified = tab.modified;

                h_stack((
                    // File path
                    label(move || {
                        file_path
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
                        if modified.get() {
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
                .into_any()
            } else {
                h_stack((
                    label(|| "No file loaded".to_string())
                        .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0)),
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
                .into_any()
            }
        },
    )
}

