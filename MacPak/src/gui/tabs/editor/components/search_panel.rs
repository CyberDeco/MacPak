//! Search and replace panel component

use floem::event::{Event, EventListener};
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::prelude::*;

use crate::gui::state::EditorTab;

use super::super::search::{find_next, find_previous, perform_search, replace_all, replace_current};

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
                    .style(|s| {
                        s.width(80.0)
                            .font_size(12.0)
                            .color(Color::rgb8(100, 100, 100))
                    }),
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
