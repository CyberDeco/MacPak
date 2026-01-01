//! Universal Editor Tab
//!
//! Multi-tab text editor for LSX, LSJ, and LSF files using Floem's text_editor
//! (the same component that powers Lapce).

mod components;
mod formatting;
mod operations;
mod search;
mod syntax;

use floem::event::{Event, EventListener};
use floem::prelude::*;
use std::path::Path;

use crate::state::{AppState, EditorTab, EditorTabsState};
use crate::utils::meta_dialog::meta_dialog;
use components::{editor_content, editor_status_bar, editor_toolbar, search_panel};

// Re-export for external use
pub use operations::load_file_in_tab;
pub use operations::open_file_dialog;
pub use operations::save_file;

/// File extensions that can be opened in the editor
fn is_editable_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "lsx" | "lsf" | "lsj" | "xml" | "json" | "txt" | "lua" | "md" | "cfg" | "ini" | "yaml" | "yml" | "toml"
            )
        })
        .unwrap_or(false)
}

pub fn editor_tab(_app_state: AppState, tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_toolbar = tabs_state.clone();
    let tabs_state_content = tabs_state.clone();
    let tabs_state_status = tabs_state.clone();
    let tabs_state_drop = tabs_state.clone();
    let tabs_state_dialog = tabs_state.clone();
    let _tabs_state_keyboard = tabs_state.clone();
    let show_line_numbers = tabs_state.show_line_numbers;

    // Callback for meta dialog - creates a new tab with the generated content
    let on_meta_create = move |content: String| {
        let tab = tabs_state_dialog.new_tab();
        tab.content.set(content);
        tab.file_format.set("LSX".to_string());
        tab.modified.set(true);
    };

    // Use v_stack with position: Relative so absolutely positioned dialogs work correctly
    v_stack((
        file_tab_bar(tabs_state.clone()),
        editor_toolbar(tabs_state_toolbar),
        dyn_container(
            {
                let tabs_state = tabs_state_content.clone();
                move || tabs_state.active_tab()
            },
            move |maybe_tab| {
                if let Some(tab) = maybe_tab {
                    v_stack((
                        search_panel(tab.clone()),
                        editor_content(tab.clone(), show_line_numbers),
                    ))
                    .style(|s| s.width_full().flex_grow(1.0).flex_basis(0.0).min_height(0.0))
                    .into_any()
                } else {
                    // Empty state with drop hint
                    v_stack((
                        label(|| "📄").style(|s| s.font_size(48.0)),
                        label(|| "Drop files here to open").style(|s| {
                            s.font_size(14.0).color(Color::rgb8(150, 150, 150))
                        }),
                    ))
                    .style(|s| {
                        s.flex_grow(1.0)
                            .width_full()
                            .items_center()
                            .justify_center()
                            .gap(8.0)
                    })
                    .into_any()
                }
            },
        )
        .style(|s| s.width_full().flex_grow(1.0).flex_basis(0.0).min_height(0.0)),
        editor_status_bar(tabs_state_status),
        // Dialog overlay - uses shared meta_dialog from utils
        meta_dialog(tabs_state.show_meta_dialog, None, on_meta_create, Some(tabs_state.status_message)),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .position(floem::style::Position::Relative)
    })
    .on_event_cont(EventListener::DroppedFile, move |e| {
        if let Event::DroppedFile(drop_event) = e {
            let path = &drop_event.path;
            if path.is_file() && is_editable_file(path) {
                load_file_in_tab(path, tabs_state_drop.clone());
            }
        }
    })
}

/// Tab bar showing all open file tabs
fn file_tab_bar(tabs_state: EditorTabsState) -> impl IntoView {
    let tabs = tabs_state.tabs;
    let active_index = tabs_state.active_tab_index;
    let tabs_state_for_new = tabs_state.clone();
    let tabs_state_inner = tabs_state.clone();

    h_stack((
        // Dynamic tabs using scroll for horizontal layout
        scroll(
            dyn_stack(
                move || {
                    let tab_list = tabs.get();
                    // Just return tabs with their index - active state checked reactively
                    tab_list.into_iter().enumerate().collect::<Vec<_>>()
                },
                |(idx, tab)| (tab.id, *idx),
                move |(index, tab)| {
                    file_tab_button(tab, index, active_index, tabs_state_inner.clone())
                },
            )
            .style(|s| s.flex_row().gap(2.0)),
        )
        .style(|s| s.max_width_pct(80.0)),
        // New tab button
        button("+")
            .style(|s| {
                s.padding_horiz(10.0)
                    .padding_vert(6.0)
                    .font_size(14.0)
                    .border_radius(4.0)
                    .background(Color::TRANSPARENT)
                    .color(Color::rgb8(100, 100, 100))
                    .hover(|s| s.background(Color::rgb8(230, 230, 230)))
            })
            .action(move || {
                tabs_state_for_new.new_tab();
            }),
        // Spacer
        empty().style(|s| s.flex_grow(1.0)),
    ))
    .style(|s| {
        s.width_full()
            .height(36.0)
            .padding_horiz(8.0)
            .padding_vert(4.0)
            .gap(4.0)
            .items_center()
            .background(Color::rgb8(250, 250, 250))
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

/// Individual file tab button
fn file_tab_button(
    tab: EditorTab,
    index: usize,
    active_index: RwSignal<usize>,
    tabs_state: EditorTabsState,
) -> impl IntoView {
    let tab_for_name = tab.clone();
    let modified = tab.modified;
    let file_path = tab.file_path;
    let tabs_state_switch = tabs_state.clone();
    let tabs_state_close = tabs_state.clone();
    let tabs_state_ctx = tabs_state.clone();

    h_stack((
        // Tab label with modified indicator
        label(move || {
            let name = tab_for_name.display_name();
            if modified.get() {
                format!("● {}", name)
            } else {
                name
            }
        })
        .style(move |s| {
            let is_active = active_index.get() == index;
            let s = s.font_size(12.0).max_width(150.0).text_ellipsis();
            if modified.get() {
                s.color(if is_active { Color::rgb8(255, 220, 150) } else { Color::rgb8(255, 152, 0) })
            } else if is_active {
                s.color(Color::WHITE)
            } else {
                s.color(Color::rgb8(60, 60, 60))
            }
        }),
        // Close button
        label(|| "×")
            .style(move |s| {
                let is_active = active_index.get() == index;
                s.font_size(14.0)
                    .margin_left(6.0)
                    .padding(2.0)
                    .border_radius(3.0)
                    .color(if is_active { Color::rgba8(255, 255, 255, 180) } else { Color::rgb8(150, 150, 150) })
                    .hover(|s| {
                        s.background(if is_active { Color::rgba8(255, 255, 255, 50) } else { Color::rgb8(200, 200, 200) })
                            .color(if is_active { Color::WHITE } else { Color::rgb8(80, 80, 80) })
                    })
                    .cursor(floem::style::CursorStyle::Pointer)
            })
            .on_click_stop(move |_| {
                tabs_state_close.close_tab(index);
            }),
    ))
    .style(move |s| {
        let is_active = active_index.get() == index;
        let s = s
            .padding_horiz(10.0)
            .padding_vert(6.0)
            .gap(4.0)
            .items_center()
            .border_radius(4.0)
            .cursor(floem::style::CursorStyle::Pointer);

        if is_active {
            s.background(Color::rgb8(33, 150, 243))
        } else {
            s.background(Color::rgb8(240, 240, 240))
                .hover(|s| s.background(Color::rgb8(225, 225, 225)))
        }
    })
    .on_click_stop(move |_| {
        tabs_state_switch.active_tab_index.set(index);
    })
    .on_secondary_click(move |_| {
        // Show context menu for rename
        show_tab_context_menu(index, file_path, tabs_state_ctx.clone());
        floem::event::EventPropagation::Stop
    })
}

/// Show context menu for a tab
fn show_tab_context_menu(
    index: usize,
    file_path: RwSignal<Option<String>>,
    tabs_state: EditorTabsState,
) {
    use floem::action::show_context_menu;
    use floem::menu::{Menu, MenuItem};
    use std::path::Path;

    let tabs_state_close = tabs_state.clone();
    let tabs_state_close_others = tabs_state.clone();
    let tabs_state_close_all = tabs_state.clone();

    let mut menu = Menu::new("");

    // Rename file (if file has a path)
    if let Some(current_path) = file_path.get() {
        let path_for_rename = current_path.clone();
        let file_path_signal = file_path;
        menu = menu.entry(
            MenuItem::new("Rename...")
                .action(move || {
                    let old_path = Path::new(&path_for_rename);
                    if let Some(old_name) = old_path.file_name().and_then(|n| n.to_str()) {
                        // Use a save dialog to get the new name
                        let dialog = rfd::FileDialog::new()
                            .set_title("Rename File")
                            .set_file_name(old_name);

                        // Set the directory to the file's parent
                        let dialog = if let Some(parent) = old_path.parent() {
                            dialog.set_directory(parent)
                        } else {
                            dialog
                        };

                        if let Some(new_path) = dialog.save_file() {
                            let new_path_str = new_path.to_string_lossy().to_string();
                            // Rename the file on disk
                            if std::fs::rename(&path_for_rename, &new_path).is_ok() {
                                // Update the tab's file path
                                file_path_signal.set(Some(new_path_str));
                            }
                        }
                    }
                })
        );

        // Copy path
        let path_for_copy = current_path.clone();
        menu = menu.entry(
            MenuItem::new("Copy Path")
                .action(move || {
                    use std::io::Write;
                    use std::process::Command;
                    if let Ok(mut child) = Command::new("pbcopy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                    {
                        if let Some(stdin) = child.stdin.as_mut() {
                            let _ = stdin.write_all(path_for_copy.as_bytes());
                        }
                    }
                })
        );
        menu = menu.separator();
    }

    // Close this tab
    menu = menu.entry(
        MenuItem::new("Close Tab")
            .action(move || {
                tabs_state_close.close_tab(index);
            })
    );

    // Close other tabs
    menu = menu.entry(
        MenuItem::new("Close Other Tabs")
            .action(move || {
                tabs_state_close_others.close_others(index);
            })
    );

    // Close all tabs
    menu = menu.entry(
        MenuItem::new("Close All Tabs")
            .action(move || {
                tabs_state_close_all.close_all();
            })
    );

    show_context_menu(menu, None);
}
