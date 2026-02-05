//! File status checklist for workspace projects

use floem::action::show_context_menu;
use floem::event::EventPropagation;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;

use crate::gui::shared::{ThemeColors, theme_signal};
use crate::gui::state::{EditorTabsState, WorkspaceState};
use crate::gui::tabs::load_file_in_tab;
use crate::workspace::FileStatus;
use crate::workspace::recipe::{FileKind, substitute};

/// File checklist showing all recipe files and their status
pub fn file_checklist(
    state: WorkspaceState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let ws_signal = state.workspace;

    v_stack((
        // Section header
        label(|| "Project Files").style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(15.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(colors.text_primary)
                .margin_bottom(8.0)
        }),
        // Scrollable file list
        scroll(dyn_container(
            move || ws_signal.get(),
            move |maybe_ws| {
                if let Some(w) = maybe_ws {
                    let vars = {
                        let mut vars = std::collections::HashMap::new();
                        vars.insert("mod_name".to_string(), w.manifest.project.folder.clone());
                        vars.insert("uuid".to_string(), w.manifest.project.uuid.clone());
                        vars.insert("author".to_string(), w.manifest.project.author.clone());
                        vars.insert("version".to_string(), w.manifest.project.version.clone());
                        for (key, value) in &w.manifest.variables {
                            vars.insert(key.clone(), value.clone());
                        }
                        vars
                    };

                    v_stack_from_iter(
                        w.recipe
                            .files
                            .iter()
                            .map(|file| {
                                let resolved_path = substitute(&file.path, &vars);
                                let status = w
                                    .file_status
                                    .get(&resolved_path)
                                    .cloned()
                                    .unwrap_or(FileStatus::Missing);
                                let description = file.description.clone();
                                let hint = file.hint.clone();
                                let kind = file.kind.clone();
                                let full_path = w.project_dir.join(&resolved_path);
                                let editor_tabs = editor_tabs_state.clone();

                                file_row(
                                    resolved_path,
                                    status,
                                    kind,
                                    description,
                                    hint,
                                    full_path,
                                    editor_tabs,
                                    active_tab,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                    .style(|s| s.width_full().gap(4.0))
                    .into_any()
                } else {
                    label(|| "No project open")
                        .style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.color(colors.text_muted)
                        })
                        .into_any()
                }
            },
        ))
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
        })
        .scroll_style(|s| s.handle_thickness(6.0)),
    ))
    .style(move |s| {
        let colors = theme_signal()
            .map(|t| ThemeColors::for_theme(t.get().effective()))
            .unwrap_or_else(ThemeColors::dark);
        s.width_full()
            .min_height(0.0)
            .padding(16.0)
            .background(colors.bg_surface)
            .border(1.0)
            .border_color(colors.border)
            .border_radius(6.0)
    })
}

fn file_row(
    path: String,
    status: FileStatus,
    kind: FileKind,
    description: String,
    hint: Option<String>,
    full_path: std::path::PathBuf,
    editor_tabs: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let status_icon = match status {
        FileStatus::Present | FileStatus::Generated => "OK",
        FileStatus::Missing => "MISSING",
        FileStatus::MissingOptional => "optional",
    };
    let status_for_style = status.clone();
    let can_open = full_path.exists();

    let path_display = path.clone();
    let hint_text = hint.unwrap_or_default();
    let has_hint = !hint_text.is_empty();
    let kind_label = match kind {
        FileKind::Generated => "auto",
        FileKind::Manual => "manual",
        FileKind::Optional => "optional",
    };

    h_stack((
        // Status indicator
        label(move || status_icon).style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            let s = s
                .font_size(10.0)
                .padding_horiz(6.0)
                .padding_vert(2.0)
                .border_radius(3.0)
                .min_width(55.0)
                .justify_center();
            match status_for_style {
                FileStatus::Present | FileStatus::Generated => {
                    s.background(colors.success_bg).color(colors.success)
                }
                FileStatus::Missing => s.background(colors.error_bg).color(colors.error),
                FileStatus::MissingOptional => {
                    s.background(colors.bg_hover).color(colors.text_muted)
                }
            }
        }),
        // File info
        v_stack((
            h_stack((
                label(move || path_display.clone()).style(move |s| {
                    let colors = theme_signal()
                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                        .unwrap_or_else(ThemeColors::dark);
                    s.font_size(12.0).color(colors.text_primary)
                }),
                label(move || kind_label).style(move |s| {
                    let colors = theme_signal()
                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                        .unwrap_or_else(ThemeColors::dark);
                    s.font_size(9.0)
                        .padding_horiz(4.0)
                        .padding_vert(1.0)
                        .border_radius(2.0)
                        .background(colors.bg_hover)
                        .color(colors.text_muted)
                }),
            ))
            .style(|s| s.gap(6.0).items_center()),
            label(move || description.clone()).style(move |s| {
                let colors = theme_signal()
                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                    .unwrap_or_else(ThemeColors::dark);
                s.font_size(11.0).color(colors.text_muted)
            }),
            // Hint (shown for manual files)
            {
                if has_hint {
                    label(move || hint_text.clone())
                        .style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.font_size(10.0)
                                .color(colors.text_muted)
                                .padding(4.0)
                                .background(colors.bg_elevated)
                                .border_radius(3.0)
                                .margin_top(2.0)
                        })
                        .into_any()
                } else {
                    empty().into_any()
                }
            },
        ))
        .style(|s| s.flex_grow(1.0).gap(2.0)),
    ))
    .style(move |s| {
        let colors = theme_signal()
            .map(|t| ThemeColors::for_theme(t.get().effective()))
            .unwrap_or_else(ThemeColors::dark);
        let s = s
            .width_full()
            .padding(8.0)
            .gap(8.0)
            .items_start()
            .border_radius(4.0);
        if can_open {
            s.hover(|s| s.background(colors.bg_hover))
        } else {
            s
        }
    })
    .on_secondary_click(move |_| {
        if can_open {
            let path_for_editor = full_path.clone();
            let path_for_finder = full_path.clone();
            let editor = editor_tabs.clone();

            let menu = Menu::new("")
                .entry(MenuItem::new("Open in Editor").action(move || {
                    load_file_in_tab(&path_for_editor, editor.clone());
                    active_tab.set(1);
                }))
                .entry(MenuItem::new("Show in Finder").action(move || {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(&path_for_finder)
                        .spawn();
                }));

            show_context_menu(menu, None);
        }
        EventPropagation::Stop
    })
}
