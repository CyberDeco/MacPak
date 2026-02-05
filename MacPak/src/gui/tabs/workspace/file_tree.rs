//! Collapsible file tree showing the project directory on disk.
//!
//! Expanded/collapsed state for each directory is persisted in
//! `WorkspaceState::file_tree_expanded` so it survives tab switches
//! (the tab container recreates views on every switch).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use floem::action::show_context_menu;
use floem::event::EventPropagation;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;

use crate::gui::shared::{ThemeColors, theme_signal};
use crate::gui::state::{EditorTabsState, WorkspaceState};
use crate::gui::tabs::load_file_in_tab;

/// A node in the flat file tree list.
#[derive(Clone)]
struct FileNode {
    name: String,
    full_path: PathBuf,
    is_dir: bool,
    depth: usize,
    has_children: bool,
    is_expanded: RwSignal<bool>,
    is_visible: RwSignal<bool>,
    /// Index of the parent directory in the flat list (None for root entries).
    parent_idx: Option<usize>,
}

/// File tree card for the workspace sidebar.
pub fn file_tree_card(
    state: WorkspaceState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let ws_signal = state.workspace;
    let expanded_map = state.file_tree_expanded;

    v_stack((
        label(|| "Files").style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(15.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(colors.text_primary)
                .margin_bottom(8.0)
        }),
        scroll(
            dyn_container(
                move || ws_signal.get().map(|w| w.project_dir.clone()),
                move |maybe_dir| {
                    let project_dir = match maybe_dir {
                        Some(d) => d,
                        None => {
                            return label(|| "No project open")
                                .style(move |s| {
                                    let colors = theme_signal()
                                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                                        .unwrap_or_else(ThemeColors::dark);
                                    s.color(colors.text_muted)
                                })
                                .into_any()
                        }
                    };

                    // Build the flat tree, restoring persisted expanded state
                    let persisted = expanded_map.get_untracked();
                    let mut nodes = Vec::new();
                    scan_directory(&project_dir, 0, &mut nodes, None, &persisted);
                    let nodes_signal = RwSignal::new(nodes.clone());

                    v_stack_from_iter(nodes.into_iter().enumerate().map(|(idx, node)| {
                        let is_visible = node.is_visible;
                        let is_expanded = node.is_expanded;
                        let is_dir = node.is_dir;
                        let has_children = node.has_children;
                        let depth = node.depth;
                        let name = node.name.clone();
                        let full_path = node.full_path.clone();
                        let path_for_persist = node.full_path.clone();
                        let path_for_click = node.full_path.clone();
                        let editor_tabs = editor_tabs_state.clone();

                        h_stack((
                            // Indentation
                            empty().style(move |s| s.width((depth * 16) as f32)),
                            // Disclosure triangle (directories with children)
                            {
                                if is_dir && has_children {
                                    let path_for_tri = path_for_persist.clone();
                                    label(move || {
                                        if is_expanded.get() {
                                            "▼"
                                        } else {
                                            "▶"
                                        }
                                    })
                                    .style(move |s| {
                                        let colors = theme_signal()
                                            .map(|t| ThemeColors::for_theme(t.get().effective()))
                                            .unwrap_or_else(ThemeColors::dark);
                                        s.font_size(10.0)
                                            .width(14.0)
                                            .color(colors.text_muted)
                                            .cursor(floem::style::CursorStyle::Pointer)
                                    })
                                    .on_click_stop(move |_| {
                                        let new_val = !is_expanded.get_untracked();
                                        is_expanded.set(new_val);
                                        update_descendant_visibility(idx, new_val, nodes_signal);
                                        // Persist expanded state
                                        expanded_map.update(|map| {
                                            map.insert(path_for_tri.clone(), new_val);
                                        });
                                    })
                                    .into_any()
                                } else {
                                    empty().style(|s| s.width(14.0)).into_any()
                                }
                            },
                            // File/dir name
                            label(move || name.clone()).style(move |s| {
                                let colors = theme_signal()
                                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                                    .unwrap_or_else(ThemeColors::dark);
                                let s = s.font_size(12.0);
                                if is_dir {
                                    s.color(colors.text_primary)
                                        .font_weight(floem::text::Weight::MEDIUM)
                                } else {
                                    s.color(colors.text_secondary)
                                }
                            }),
                        ))
                        .style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            let base = s
                                .width_full()
                                .padding_vert(2.0)
                                .padding_horiz(4.0)
                                .items_center()
                                .gap(2.0)
                                .border_radius(3.0)
                                .hover(|s| s.background(colors.bg_hover));
                            let base = if is_dir {
                                base.cursor(floem::style::CursorStyle::Pointer)
                            } else {
                                base
                            };
                            if is_visible.get() {
                                base
                            } else {
                                base.display(floem::style::Display::None)
                            }
                        })
                        .on_click_stop(move |_| {
                            if is_dir {
                                let new_val = !is_expanded.get_untracked();
                                is_expanded.set(new_val);
                                update_descendant_visibility(idx, new_val, nodes_signal);
                                expanded_map.update(|map| {
                                    map.insert(path_for_click.clone(), new_val);
                                });
                            }
                        })
                        .on_secondary_click(move |_| {
                            if full_path.exists() {
                                let path_for_editor = full_path.clone();
                                let path_for_finder = full_path.clone();
                                let editor = editor_tabs.clone();

                                let mut menu = Menu::new("");

                                if !is_dir {
                                    menu = menu.entry(
                                        MenuItem::new("Open in Editor").action(move || {
                                            load_file_in_tab(&path_for_editor, editor.clone());
                                            active_tab.set(1);
                                        }),
                                    );
                                }

                                menu = menu.entry(
                                    MenuItem::new("Show in Finder").action(move || {
                                        let _ = std::process::Command::new("open")
                                            .arg("-R")
                                            .arg(&path_for_finder)
                                            .spawn();
                                    }),
                                );

                                show_context_menu(menu, None);
                            }
                            EventPropagation::Stop
                        })
                    }))
                    .style(|s| s.width_full())
                    .into_any()
                },
            ),
        )
        .style(|s| s.width_full().flex_grow(1.0).flex_basis(0.0).min_height(0.0))
        .scroll_style(|s| s.handle_thickness(6.0)),
    ))
    .style(move |s| {
        let colors = theme_signal()
            .map(|t| ThemeColors::for_theme(t.get().effective()))
            .unwrap_or_else(ThemeColors::dark);
        s.width_full()
            .min_height(0.0)
            .flex_grow(1.0)
            .flex_basis(0.0)
            .padding(16.0)
            .background(colors.bg_surface)
            .border(1.0)
            .border_color(colors.border)
            .border_radius(6.0)
    })
}

/// Recursively scan a directory and build a flat depth-first node list.
///
/// Uses `persisted` to restore previously saved expanded/collapsed state
/// for each directory. Directories not in the map default to expanded
/// for the first two depth levels.
fn scan_directory(
    dir: &Path,
    depth: usize,
    nodes: &mut Vec<FileNode>,
    parent_idx: Option<usize>,
    persisted: &HashMap<PathBuf, bool>,
) {
    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_dir
            .cmp(&a_dir)
            .then_with(|| a.file_name().cmp(&b.file_name()))
    });

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/dirs
        if name.starts_with('.') {
            continue;
        }

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let full_path = entry.path();

        // Restore persisted expanded state, or default to auto-expand first 2 levels
        let expanded = if is_dir {
            persisted
                .get(&full_path)
                .copied()
                .unwrap_or(depth < 2)
        } else {
            false
        };

        // Visible if parent is visible and expanded
        let parent_visible_and_expanded = parent_idx
            .map(|pi| {
                nodes[pi].is_visible.get_untracked() && nodes[pi].is_expanded.get_untracked()
            })
            .unwrap_or(true);

        let idx = nodes.len();
        nodes.push(FileNode {
            name,
            full_path: full_path.clone(),
            is_dir,
            depth,
            has_children: false, // updated below for dirs
            is_expanded: RwSignal::new(expanded),
            is_visible: RwSignal::new(parent_visible_and_expanded),
            parent_idx,
        });

        if is_dir {
            let before = nodes.len();
            scan_directory(&full_path, depth + 1, nodes, Some(idx), persisted);
            nodes[idx].has_children = nodes.len() > before;
        }
    }
}

/// Update visibility of all descendants when a directory is expanded/collapsed.
fn update_descendant_visibility(
    parent_idx: usize,
    parent_expanded: bool,
    nodes_signal: RwSignal<Vec<FileNode>>,
) {
    nodes_signal.with_untracked(|nodes| {
        let parent_depth = nodes[parent_idx].depth;

        // Walk subsequent nodes that are deeper (descendants in the flat DFS list)
        let mut i = parent_idx + 1;
        while i < nodes.len() && nodes[i].depth > parent_depth {
            let node = &nodes[i];

            if node.depth == parent_depth + 1 {
                // Direct child: visible iff parent is expanded
                node.is_visible.set(parent_expanded);
            } else {
                // Deeper descendant: visible iff its own parent is visible AND expanded
                if let Some(pi) = node.parent_idx {
                    let p = &nodes[pi];
                    node.is_visible
                        .set(p.is_visible.get_untracked() && p.is_expanded.get_untracked());
                }
            }
            i += 1;
        }
    });
}
