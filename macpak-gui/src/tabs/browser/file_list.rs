//! File list widget with sortable columns and keyboard navigation

use floem::event::EventPropagation;
use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem::text::Weight;

use crate::state::{BrowserState, EditorTabsState, FileEntry, SortColumn};
use super::context_menu::show_file_context_menu;
use super::operations::{open_file_or_folder_filtered, perform_rename, select_file, sort_files};

pub fn file_list(
    state: BrowserState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let files = state.files;
    let selected = state.selected_index;
    let state_select = state.clone();
    let sort_column = state.sort_column;
    let sort_ascending = state.sort_ascending;
    let current_path = state.current_path;

    let state_name = state.clone();
    let state_type = state.clone();
    let state_size = state.clone();
    let state_modified = state.clone();
    let state_keyboard_down = state.clone();
    let state_keyboard_up = state.clone();
    let state_keyboard_enter = state.clone();
    let editor_keyboard = editor_tabs_state.clone();

    dyn_container(
        move || current_path.get().is_some(),
        move |has_folder| {
            if !has_folder {
                // Placeholder when no folder is opened
                v_stack((
                    label(|| "📁").style(|s| s.font_size(64.0)),
                    label(|| "Select a folder to browse")
                        .style(|s| s.font_size(16.0).color(Color::rgb8(120, 120, 120))),
                    label(|| "Click \"Browse\" to open a folder")
                        .style(|s| s.font_size(13.0).color(Color::rgb8(160, 160, 160))),
                ))
                .style(|s| {
                    s.width_full()
                        .height_full()
                        .items_center()
                        .justify_center()
                        .gap(8.0)
                        .background(Color::WHITE)
                })
                .into_any()
            } else {
                file_list_content(
                    files,
                    selected,
                    sort_column,
                    sort_ascending,
                    state_select.clone(),
                    state_name.clone(),
                    state_type.clone(),
                    state_size.clone(),
                    state_modified.clone(),
                    state_keyboard_down.clone(),
                    state_keyboard_up.clone(),
                    state_keyboard_enter.clone(),
                    editor_tabs_state.clone(),
                    editor_keyboard.clone(),
                    active_tab,
                )
                .into_any()
            }
        },
    )
    .style(|s| {
        s.width_pct(60.0)
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)
            .background(Color::WHITE)
            .border_right(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn file_list_content(
    files: RwSignal<Vec<FileEntry>>,
    selected: RwSignal<Option<usize>>,
    sort_column: RwSignal<SortColumn>,
    sort_ascending: RwSignal<bool>,
    state_select: BrowserState,
    state_name: BrowserState,
    state_type: BrowserState,
    state_size: BrowserState,
    state_modified: BrowserState,
    state_keyboard_down: BrowserState,
    state_keyboard_up: BrowserState,
    state_keyboard_enter: BrowserState,
    editor_tabs_state: EditorTabsState,
    editor_keyboard: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    v_stack((
        // Column headers
        h_stack((
            sortable_header("Name", SortColumn::Name, sort_column, sort_ascending, state_name),
            sortable_header("Type", SortColumn::Type, sort_column, sort_ascending, state_type)
                .style(|s| s.width(60.0)),
            sortable_header("Size", SortColumn::Size, sort_column, sort_ascending, state_size)
                .style(|s| s.width(80.0)),
            sortable_header("Modified", SortColumn::Modified, sort_column, sort_ascending, state_modified)
                .style(|s| s.width(120.0)),
        ))
        .style(|s| {
            s.width_full()
                .padding(8.0)
                .gap(8.0)
                .background(Color::rgb8(240, 240, 240))
                .border_bottom(1.0)
                .border_color(Color::rgb8(200, 200, 200))
        }),
        // File rows with scroll - use min_height(0) to allow shrinking
        scroll(
            dyn_stack(
                move || files.get(),
                |file| file.path.clone(),
                move |file| {
                    let state_row = state_select.clone();
                    let state_dbl = state_select.clone();
                    let state_ctx = state_select.clone();
                    let editor_for_open = editor_tabs_state.clone();
                    let editor_for_ctx = editor_tabs_state.clone();
                    let file_path = file.path.clone();
                    let file_for_select = file.clone();
                    let file_for_open = file.clone();
                    let file_for_ctx = file.clone();
                    let idx = files.get().iter().position(|f| f.path == file_path);

                    file_row(file, selected, idx, state_row.clone())
                        .on_click_stop(move |_| {
                            // Cancel any ongoing rename when clicking elsewhere
                            state_row.renaming_path.set(None);
                            if let Some(i) = idx {
                                state_row.selected_index.set(Some(i));
                                select_file(&file_for_select, state_row.clone());
                            }
                        })
                        .on_double_click(move |_| {
                            // Only open text files in editor on double-click
                            open_file_or_folder_filtered(
                                &file_for_open,
                                state_dbl.clone(),
                                editor_for_open.clone(),
                                active_tab,
                            );
                            EventPropagation::Stop
                        })
                        .on_secondary_click(move |_| {
                            // Select the file first
                            if let Some(i) = idx {
                                state_ctx.selected_index.set(Some(i));
                                select_file(&file_for_ctx, state_ctx.clone());
                            }
                            // Show context menu
                            show_file_context_menu(
                                &file_for_ctx,
                                state_ctx.clone(),
                                editor_for_ctx.clone(),
                                active_tab,
                            );
                            EventPropagation::Stop
                        })
                },
            )
            .style(|s| s.width_full().flex_col()),
        )
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)  // Allow scroll to shrink - critical for scroll to work
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .min_height(0.0)
            .background(Color::WHITE)
    })
    .keyboard_navigable()
    .on_key_down(
        Key::Named(NamedKey::ArrowDown),
        |_| true,
        move |_| {
            let files_list = state_keyboard_down.files.get();
            let current = state_keyboard_down.selected_index.get();
            let new_idx = match current {
                Some(i) if i + 1 < files_list.len() => Some(i + 1),
                None if !files_list.is_empty() => Some(0),
                _ => current,
            };
            if new_idx != current {
                state_keyboard_down.selected_index.set(new_idx);
                if let Some(i) = new_idx {
                    if let Some(file) = files_list.get(i) {
                        select_file(file, state_keyboard_down.clone());
                    }
                }
            }
        },
    )
    .on_key_down(
        Key::Named(NamedKey::ArrowUp),
        |_| true,
        move |_| {
            let files_list = state_keyboard_up.files.get();
            let current = state_keyboard_up.selected_index.get();
            let new_idx = match current {
                Some(i) if i > 0 => Some(i - 1),
                None if !files_list.is_empty() => Some(0),
                _ => current,
            };
            if new_idx != current {
                state_keyboard_up.selected_index.set(new_idx);
                if let Some(i) = new_idx {
                    if let Some(file) = files_list.get(i) {
                        select_file(file, state_keyboard_up.clone());
                    }
                }
            }
        },
    )
    .on_key_down(
        Key::Named(NamedKey::Enter),
        |_| true,
        move |_| {
            let files_list = state_keyboard_enter.files.get();
            if let Some(i) = state_keyboard_enter.selected_index.get() {
                if let Some(file) = files_list.get(i) {
                    open_file_or_folder_filtered(
                        file,
                        state_keyboard_enter.clone(),
                        editor_keyboard.clone(),
                        active_tab,
                    );
                }
            }
        },
    )
}

fn file_row(file: FileEntry, selected: RwSignal<Option<usize>>, idx: Option<usize>, state: BrowserState) -> impl IntoView {
    let is_selected = move || selected.get() == idx;
    let icon = file.icon.clone();
    let name = file.name.clone();
    let file_type = file.file_type.clone();
    let size = file.size_formatted.clone();
    let modified = file.modified.clone();
    let file_path = file.path.clone();
    let file_path_for_rename = file.path.clone();

    let renaming_path = state.renaming_path;
    let rename_text = state.rename_text;

    h_stack((
        // Icon + Name (with inline rename support)
        h_stack((
            label(move || icon.clone()).style(|s| s.width(24.0)),
            dyn_container(
                move || {
                    let is_renaming = renaming_path.get().as_ref() == Some(&file_path);
                    is_renaming
                },
                {
                    let name = name.clone();
                    move |is_renaming| {
                        let file_path_inner = file_path_for_rename.clone();
                        let state_inner = state.clone();
                        let name_inner = name.clone();
                        if is_renaming {
                            let state_esc = state_inner.clone();
                            // Show text input for renaming
                            text_input(rename_text)
                                .style(|s| {
                                    s.width_full()
                                        .min_width(50.0)
                                        .padding(2.0)
                                        .border(1.0)
                                        .border_color(Color::rgb8(33, 150, 243))
                                        .border_radius(2.0)
                                        .background(Color::WHITE)
                                })
                                .on_key_down(
                                    Key::Named(NamedKey::Enter),
                                    |_| true,
                                    move |_| {
                                        // Confirm rename
                                        let new_name = state_inner.rename_text.get();
                                        if !new_name.is_empty() {
                                            perform_rename(&file_path_inner, &new_name, state_inner.clone());
                                        }
                                        state_inner.renaming_path.set(None);
                                    },
                                )
                                .on_key_down(
                                    Key::Named(NamedKey::Escape),
                                    |_| true,
                                    move |_| {
                                        // Cancel rename
                                        state_esc.renaming_path.set(None);
                                    },
                                )
                                .into_any()
                        } else {
                            // Show label
                            label(move || name_inner.clone())
                                .style(|s| s.flex_grow(1.0).text_ellipsis())
                                .into_any()
                        }
                    }
                },
            )
            .style(|s| s.flex_grow(1.0).min_width(0.0)),
        ))
        .style(|s| s.flex_grow(1.0).gap(4.0).min_width(0.0)),
        // Type
        label(move || file_type.clone()).style(|s| {
            s.width(60.0)
                .font_size(12.0)
                .color(Color::rgb8(100, 100, 100))
        }),
        // Size
        label(move || size.clone()).style(|s| {
            s.width(80.0)
                .font_size(12.0)
                .color(Color::rgb8(100, 100, 100))
        }),
        // Modified
        label(move || modified.clone()).style(|s| {
            s.width(120.0)
                .font_size(12.0)
                .color(Color::rgb8(100, 100, 100))
        }),
    ))
    .style(move |s| {
        let s = s
            .width_full()
            .padding(8.0)
            .gap(8.0)
            .items_center()
            .border_bottom(1.0)
            .border_color(Color::rgb8(245, 245, 245));

        if is_selected() {
            s.background(Color::rgb8(227, 242, 253))
        } else {
            s.background(Color::WHITE)
                .hover(|s| s.background(Color::rgb8(250, 250, 250)))
        }
    })
}

fn sortable_header(
    name: &'static str,
    column: SortColumn,
    sort_column: RwSignal<SortColumn>,
    sort_ascending: RwSignal<bool>,
    state: BrowserState,
) -> impl IntoView {
    h_stack((
        label(move || {
            let current = sort_column.get();
            let asc = sort_ascending.get();
            if current == column {
                if asc {
                    format!("{} ▲", name)
                } else {
                    format!("{} ▼", name)
                }
            } else {
                name.to_string()
            }
        })
        .style(|s| s.font_weight(Weight::BOLD)),
    ))
    .style(move |s| {
        s.cursor(floem::style::CursorStyle::Pointer)
            .hover(|s| s.background(Color::rgb8(230, 230, 230)))
            .padding_vert(2.0)
            .padding_horiz(4.0)
            .border_radius(4.0)
            .flex_grow(if column == SortColumn::Name { 1.0 } else { 0.0 })
    })
    .on_click_stop(move |_| {
        let current = sort_column.get();
        if current == column {
            sort_ascending.set(!sort_ascending.get());
        } else {
            sort_column.set(column);
            sort_ascending.set(true);
        }
        sort_files(state.clone());
    })
}
