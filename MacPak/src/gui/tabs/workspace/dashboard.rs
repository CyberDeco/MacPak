//! Project dashboard shown when a workspace is open

use floem::prelude::*;

use crate::gui::shared::{ThemeColors, theme_signal};
use crate::gui::state::{EditorTabsState, WorkspaceState};

use super::build::build_panel;
use super::file_checklist::file_checklist;

/// Main dashboard view for an open project
pub fn project_dashboard(
    state: WorkspaceState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let ws = state.workspace;
    let state_for_close = state.clone();
    let state_for_finder = state.clone();
    let state_for_refresh = state.clone();

    v_stack((
        // Project info bar
        dyn_container(
            move || ws.get(),
            move |maybe_ws| {
                if let Some(w) = maybe_ws {
                    let folder = w.project_dir.to_string_lossy().to_string();
                    let version = w.manifest.project.version.clone();
                    let author = w.manifest.project.author.clone();

                    h_stack((
                        // Path
                        label(move || folder.clone()).style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.font_size(12.0).color(colors.text_muted)
                        }),
                        empty().style(|s| s.flex_grow(1.0)),
                        // Author
                        label(move || {
                            if author.is_empty() {
                                String::new()
                            } else {
                                format!("by {}", author)
                            }
                        })
                        .style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.font_size(12.0).color(colors.text_muted)
                        }),
                        // Version
                        label(move || format!("v{}", version)).style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.font_size(12.0)
                                .padding_horiz(6.0)
                                .padding_vert(2.0)
                                .border_radius(3.0)
                                .background(colors.bg_hover)
                                .color(colors.text_muted)
                        }),
                    ))
                    .style(move |s| {
                        let colors = theme_signal()
                            .map(|t| ThemeColors::for_theme(t.get().effective()))
                            .unwrap_or_else(ThemeColors::dark);
                        s.width_full()
                            .padding_horiz(24.0)
                            .padding_vert(8.0)
                            .gap(8.0)
                            .items_center()
                            .background(colors.bg_surface)
                            .border_bottom(1.0)
                            .border_color(colors.border)
                    })
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        // Main content: checklist + build panel side by side
        h_stack((
            // File checklist (left, takes most space)
            file_checklist(state.clone(), editor_tabs_state, active_tab)
                .style(|s| s.flex_grow(1.0).flex_basis(0.0).min_width(0.0)),
            // Build panel (right sidebar)
            build_panel(state.clone()).style(|s| s.width(280.0)),
        ))
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
                .padding(24.0)
                .gap(24.0)
        }),
        // Action bar at bottom
        h_stack((
            button("Refresh")
                .action(move || {
                    state_for_refresh.workspace.update(|ws| {
                        if let Some(w) = ws {
                            w.refresh_status();
                        }
                    });
                })
                .style(move |s| {
                    let colors = theme_signal()
                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                        .unwrap_or_else(ThemeColors::dark);
                    s.padding_horiz(16.0)
                        .padding_vert(8.0)
                        .background(colors.bg_elevated)
                        .color(colors.text_primary)
                        .border(1.0)
                        .border_color(colors.border)
                        .border_radius(6.0)
                        .hover(|s| s.background(colors.bg_hover))
                }),
            button("Open in Finder")
                .action(move || {
                    if let Some(ref w) = state_for_finder.workspace.get() {
                        let _ = std::process::Command::new("open")
                            .arg(&w.project_dir)
                            .spawn();
                    }
                })
                .style(move |s| {
                    let colors = theme_signal()
                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                        .unwrap_or_else(ThemeColors::dark);
                    s.padding_horiz(16.0)
                        .padding_vert(8.0)
                        .background(colors.bg_elevated)
                        .color(colors.text_primary)
                        .border(1.0)
                        .border_color(colors.border)
                        .border_radius(6.0)
                        .hover(|s| s.background(colors.bg_hover))
                }),
            empty().style(|s| s.flex_grow(1.0)),
            button("Close Project")
                .action(move || {
                    state_for_close.workspace.set(None);
                    state_for_close.result_message.set(None);
                    state_for_close.error_message.set(None);
                })
                .style(move |s| {
                    let colors = theme_signal()
                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                        .unwrap_or_else(ThemeColors::dark);
                    s.padding_horiz(16.0)
                        .padding_vert(8.0)
                        .background(colors.bg_elevated)
                        .color(colors.error)
                        .border(1.0)
                        .border_color(colors.border)
                        .border_radius(6.0)
                        .hover(|s| s.background(colors.bg_hover))
                }),
        ))
        .style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.width_full()
                .padding(16.0)
                .gap(8.0)
                .items_center()
                .background(colors.bg_surface)
                .border_top(1.0)
                .border_color(colors.border)
        }),
    ))
    .style(|s| s.width_full().height_full())
}
