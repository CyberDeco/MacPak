//! UI components for the editor

use floem::action::exec_after;
use floem::prelude::*;
use floem::text::Weight;
use floem::views::text_editor;
use std::time::Duration;

use crate::state::{EditorTab, EditorTabsState};
use crate::tabs::tools::meta_generator;
use super::operations::{
    convert_file, format_content, open_file_dialog, save_file, save_file_as_dialog,
    validate_content,
};
use super::search::{find_next, find_previous, perform_search, replace_all, replace_current};
use super::syntax::SyntaxStyling;

pub fn editor_toolbar(tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_open = tabs_state.clone();
    let tabs_state_save_check = tabs_state.clone();
    let tabs_state_save_action = tabs_state.clone();
    let tabs_state_save_as = tabs_state.clone();
    let tabs_state_validate = tabs_state.clone();
    let tabs_state_format = tabs_state.clone();
    let tabs_state_find = tabs_state.clone();
    let tabs_state_lsx = tabs_state.clone();
    let tabs_state_lsj = tabs_state.clone();
    let tabs_state_lsf = tabs_state.clone();
    let tabs_state_meta = tabs_state.clone();

    h_stack((
        // File operations group
        h_stack((
            button("📂 Open").action(move || {
                open_file_dialog(tabs_state_open.clone());
            }),
            button("Generate meta.lsx").action(move || {
                tabs_state_meta.show_meta_dialog.set(true);
            }),
            button("💾 Save")
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
            button("Save As...").action(move || {
                if let Some(tab) = tabs_state_save_as.active_tab() {
                    save_file_as_dialog(tab);
                }
            }),
        ))
        .style(|s| s.gap(8.0)),
        separator(),
        // Edit tools group
        h_stack((
            button("🔍 Find").action({
                move || {
                    if let Some(tab) = tabs_state_find.active_tab() {
                        let visible = tab.search_visible.get();
                        tab.search_visible.set(!visible);
                    }
                }
            }),
            line_number_toggle(tabs_state.show_line_numbers),
            button("📐 Format").action(move || {
                if let Some(tab) = tabs_state_format.active_tab() {
                    format_content(tab);
                }
            }),
            button("✓ Validate").action(move || {
                if let Some(tab) = tabs_state_validate.active_tab() {
                    validate_content(tab, tabs_state.status_message);
                }
            }),
        ))
        .style(|s| s.gap(8.0)),
        separator(),
        // Convert section group
        h_stack((
            label(|| "Convert:").style(|s| s.font_weight(Weight::BOLD).margin_right(8.0)),
            convert_button("LSX", tabs_state_lsx),
            convert_button("LSJ", tabs_state_lsj),
            convert_button("LSF", tabs_state_lsf),
        ))
        .style(|s| s.gap(8.0)),
        // Spacer
        empty().style(|s| s.flex_grow(1.0)),
        // Format badge
        format_badge(tabs_state.clone()),
        // Status message
        status_message(tabs_state.clone()),
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

fn status_message(tabs_state: EditorTabsState) -> impl IntoView {
    dyn_container(
        move || {
            let global_msg = tabs_state.status_message.get();
            let tab_info = tabs_state.active_tab().map(|tab| {
                (tab.converted_from_lsf.get(), global_msg.clone())
            });
            tab_info
        },
        move |maybe_info| {
            if let Some((is_converted, msg)) = maybe_info {
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
            } else {
                empty().into_any()
            }
        },
    )
}

fn convert_button(format: &'static str, tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_check = tabs_state.clone();
    let tabs_state_action = tabs_state.clone();

    button(format)
        .disabled(move || {
            tabs_state_check.active_tab().map_or(true, |tab| {
                let f = tab.file_format.get().to_uppercase();
                let empty = tab.content.get().is_empty();
                f == format || empty
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

pub fn editor_content(tab: EditorTab, show_line_numbers: RwSignal<bool>) -> impl IntoView {
    let content = tab.content;
    let modified = tab.modified;
    let file_format = tab.file_format;

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

/// Meta.lsx Generator Dialog
pub fn meta_lsx_dialog(tabs_state: EditorTabsState) -> impl IntoView {
    let show_dialog = tabs_state.show_meta_dialog;
    let mod_name = tabs_state.meta_mod_name;
    let folder = tabs_state.meta_folder;
    let author = tabs_state.meta_author;
    let description = tabs_state.meta_description;
    let uuid = tabs_state.meta_uuid;
    let version_major = tabs_state.meta_version_major;
    let version_minor = tabs_state.meta_version_minor;
    let version_patch = tabs_state.meta_version_patch;
    let version_build = tabs_state.meta_version_build;

    let tabs_state_create = tabs_state.clone();
    let tabs_state_cancel = tabs_state.clone();

    dyn_container(
        move || show_dialog.get(),
        move |is_visible| {
            if !is_visible {
                return empty().into_any();
            }

            let tabs_state_create = tabs_state_create.clone();
            let tabs_state_cancel = tabs_state_cancel.clone();

            // Dialog box
            v_stack((
                // Header
                label(|| "New Meta.lsx")
                    .style(|s| s.font_size(18.0).font_weight(Weight::BOLD).margin_bottom(16.0)),

                // Form fields
                v_stack((
                    // Row 1: Mod Name and Folder
                    h_stack((
                        meta_text_field("Mod Name", mod_name, "My Awesome Mod"),
                        meta_text_field("Folder", folder, "MyAwesomeMod"),
                    ))
                    .style(|s| s.width_full().gap(12.0)),

                    // Row 2: Author and Description
                    h_stack((
                        meta_text_field("Author", author, "Your Name"),
                        meta_text_field("Description", description, "A short description..."),
                    ))
                    .style(|s| s.width_full().gap(12.0)),

                    // Row 3: UUID with generate button
                    h_stack((
                        v_stack((
                            label(|| "UUID").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                            h_stack((
                                text_input(uuid)
                                    .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx")
                                    .style(|s| {
                                        s.flex_grow(1.0)
                                            .flex_basis(0.0)
                                            .width_full()
                                            .padding(8.0)
                                            .border(1.0)
                                            .border_color(Color::rgb8(200, 200, 200))
                                            .border_radius(4.0)
                                            .font_family("monospace".to_string())
                                    }),
                                button("Generate")
                                    .style(|s| s.margin_left(4.0))
                                    .action(move || {
                                        let new_uuid = uuid::Uuid::new_v4().to_string();
                                        uuid.set(new_uuid);
                                    }),
                            ))
                            .style(|s| s.width_full()),
                        ))
                        .style(|s| s.flex_grow(1.0).flex_basis(0.0).gap(4.0)),
                    ))
                    .style(|s| s.width_full()),

                    // Row 4: Version
                    h_stack((
                        v_stack((
                            label(|| "Version").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                            h_stack((
                                meta_version_field(version_major, "Major"),
                                label(|| ".").style(|s| s.font_size(16.0).margin_horiz(2.0)),
                                meta_version_field(version_minor, "Minor"),
                                label(|| ".").style(|s| s.font_size(16.0).margin_horiz(2.0)),
                                meta_version_field(version_patch, "Patch"),
                                label(|| ".").style(|s| s.font_size(16.0).margin_horiz(2.0)),
                                meta_version_field(version_build, "Build"),
                            ))
                            .style(|s| s.items_center()),
                        ))
                        .style(|s| s.gap(4.0)),
                    ))
                    .style(|s| s.width_full()),
                ))
                .style(|s| s.width_full().gap(12.0)),

                // Action buttons
                h_stack((
                    empty().style(|s| s.flex_grow(1.0)),
                    button("Cancel")
                        .style(|s| {
                            s.padding_horiz(16.0)
                                .padding_vert(8.0)
                                .border_radius(4.0)
                        })
                        .action(move || {
                            tabs_state_cancel.show_meta_dialog.set(false);
                        }),
                    button("Create")
                        .style(|s| {
                            s.padding_horiz(16.0)
                                .padding_vert(8.0)
                                .background(Color::rgb8(33, 150, 243))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .font_weight(Weight::SEMIBOLD)
                        })
                        .action(move || {
                            // Generate meta.lsx content
                            let content = meta_generator::generate_meta_lsx(
                                &tabs_state_create.meta_mod_name.get(),
                                &tabs_state_create.meta_folder.get(),
                                &tabs_state_create.meta_author.get(),
                                &tabs_state_create.meta_description.get(),
                                &tabs_state_create.meta_uuid.get(),
                                tabs_state_create.meta_version_major.get(),
                                tabs_state_create.meta_version_minor.get(),
                                tabs_state_create.meta_version_patch.get(),
                                tabs_state_create.meta_version_build.get(),
                            );

                            // Create new tab with content
                            let tab = tabs_state_create.new_tab();
                            tab.content.set(content);
                            tab.file_format.set("LSX".to_string());
                            tab.modified.set(true);

                            // Close dialog
                            tabs_state_create.show_meta_dialog.set(false);
                            tabs_state_create.status_message.set("Created new meta.lsx".to_string());

                            // Clear status after 3 seconds
                            let status_msg = tabs_state_create.status_message;
                            exec_after(Duration::from_secs(3), move |_| {
                                if status_msg.get() == "Created new meta.lsx" {
                                    status_msg.set(String::new());
                                }
                            });
                        }),
                ))
                .style(|s| s.width_full().gap(8.0).margin_top(16.0)),
            ))
            .style(|s| {
                s.width(500.0)
                    .padding(24.0)
                    .background(Color::WHITE)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(8.0)
                    .box_shadow_blur(20.0)
                    .box_shadow_color(Color::rgba8(0, 0, 0, 50))
            })
            .into_any()
        },
    )
    .style(move |s| {
        if show_dialog.get() {
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

fn meta_text_field(
    label_text: &'static str,
    signal: RwSignal<String>,
    placeholder: &'static str,
) -> impl IntoView {
    v_stack((
        label(move || label_text)
            .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
        text_input(signal)
            .placeholder(placeholder)
            .style(|s| {
                s.width_full()
                    .padding(8.0)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
    ))
    .style(|s| s.flex_grow(1.0).gap(4.0))
}

fn meta_version_field(signal: RwSignal<u32>, placeholder: &'static str) -> impl IntoView {
    let text_signal = RwSignal::new(signal.get().to_string());

    text_input(text_signal)
        .placeholder(placeholder)
        .style(|s| {
            s.width(50.0)
                .padding(6.0)
                .border(1.0)
                .border_color(Color::rgb8(200, 200, 200))
                .border_radius(4.0)
        })
        .on_event_stop(floem::event::EventListener::KeyUp, move |_| {
            if let Ok(val) = text_signal.get().parse::<u32>() {
                signal.set(val);
            }
        })
}
