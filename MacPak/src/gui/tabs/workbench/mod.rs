//! Workbench Tab - Project management for BG3 mods

mod build;
mod dashboard;
mod file_checklist;
mod file_tree;
mod new_project;

use floem::prelude::*;

use crate::gui::shared::{ThemeColors, theme_signal};
use crate::gui::state::{EditorTabsState, WorkbenchState};

pub fn workbench_tab(
    workbench_state: WorkbenchState,
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    let ws = workbench_state.clone();
    let ws_for_new = workbench_state.clone();

    v_stack((
        // Header bar
        workbench_header(workbench_state.clone()),
        // Content: either welcome screen or project dashboard
        dyn_container(
            move || ws.workbench.get().is_some(),
            move |has_project| {
                if has_project {
                    dashboard::project_dashboard(
                        workbench_state.clone(),
                        editor_tabs_state.clone(),
                        active_tab,
                    )
                    .into_any()
                } else {
                    welcome_screen(workbench_state.clone()).into_any()
                }
            },
        )
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
        }),
        // New Project dialog overlay
        new_project::new_project_dialog(ws_for_new),
    ))
    .style(move |s| {
        let colors = theme_signal()
            .map(|t| ThemeColors::for_theme(t.get().effective()))
            .unwrap_or_else(ThemeColors::dark);
        s.width_full()
            .height_full()
            .background(colors.bg_base)
            .position(floem::style::Position::Relative)
    })
}

fn workbench_header(state: WorkbenchState) -> impl IntoView {
    let ws = state.workbench;
    let result_msg = state.result_message;
    let error_msg = state.error_message;

    h_stack((
        // Title
        label(move || {
            if let Some(ref w) = ws.get() {
                format!("Workbench: {}", w.manifest.project.name)
            } else {
                "Workbench".to_string()
            }
        })
        .style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(18.0)
                .font_weight(floem::text::Weight::BOLD)
                .color(colors.text_primary)
        }),
        // Recipe badge
        dyn_container(
            move || ws.get().map(|w| w.recipe.recipe.name.clone()),
            move |maybe_name| {
                if let Some(name) = maybe_name {
                    label(move || name.clone())
                        .style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.padding_horiz(8.0)
                                .padding_vert(2.0)
                                .border_radius(4.0)
                                .font_size(11.0)
                                .background(colors.accent)
                                .color(colors.text_inverse)
                        })
                        .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        empty().style(|s| s.flex_grow(1.0)),
        // Result message
        dyn_container(
            move || result_msg.get(),
            move |msg| {
                if let Some(msg) = msg {
                    label(move || msg.clone())
                        .style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.padding_horiz(12.0)
                                .padding_vert(4.0)
                                .border_radius(4.0)
                                .font_size(12.0)
                                .background(colors.success_bg)
                                .color(colors.success)
                        })
                        .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        // Error message
        dyn_container(
            move || error_msg.get(),
            move |msg| {
                if let Some(msg) = msg {
                    label(move || msg.clone())
                        .style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.padding_horiz(12.0)
                                .padding_vert(4.0)
                                .border_radius(4.0)
                                .font_size(12.0)
                                .background(colors.error_bg)
                                .color(colors.error)
                        })
                        .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
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
            .border_bottom(1.0)
            .border_color(colors.border)
    })
}

fn welcome_screen(state: WorkbenchState) -> impl IntoView {
    let state_for_new = state.clone();
    let state_for_open = state.clone();

    v_stack((
        // Welcome title
        label(|| "Get Started").style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(24.0)
                .font_weight(floem::text::Weight::BOLD)
                .color(colors.text_primary)
                .margin_bottom(8.0)
        }),
        label(|| "Create a new mod project or open an existing one.").style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(14.0)
                .color(colors.text_secondary)
                .margin_bottom(24.0)
        }),
        // Action buttons
        h_stack((
            // New Project button
            button("New Project")
                .action(move || {
                    state_for_new.show_new_dialog.set(true);
                })
                .style(move |s| {
                    let colors = theme_signal()
                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                        .unwrap_or_else(ThemeColors::dark);
                    s.padding_horiz(24.0)
                        .padding_vert(12.0)
                        .background(colors.accent)
                        .color(colors.text_inverse)
                        .border_radius(6.0)
                        .font_size(14.0)
                        .hover(|s| s.background(colors.accent_hover))
                }),
            // Open Project button
            button("Open Project")
                .action(move || {
                    open_project_dialog(state_for_open.clone());
                })
                .style(move |s| {
                    let colors = theme_signal()
                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                        .unwrap_or_else(ThemeColors::dark);
                    s.padding_horiz(24.0)
                        .padding_vert(12.0)
                        .background(colors.bg_elevated)
                        .color(colors.text_primary)
                        .border(1.0)
                        .border_color(colors.border)
                        .border_radius(6.0)
                        .font_size(14.0)
                        .hover(|s| s.background(colors.bg_hover))
                }),
        ))
        .style(|s| s.gap(12.0)),
    ))
    .style(|s| s.width_full().height_full().items_center().justify_center())
}

/// Open an existing project via folder picker
pub fn open_project_dialog(state: WorkbenchState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Open MacPak Project")
        .set_directory(crate::workbench::Workbench::default_projects_dir());

    if let Some(path) = dialog.pick_folder() {
        match crate::workbench::Workbench::open(&path) {
            Ok(ws) => {
                state.error_message.set(None);
                state
                    .result_message
                    .set(Some(format!("Opened: {}", ws.manifest.project.name)));
                state.workbench.set(Some(ws));
            }
            Err(e) => {
                state
                    .error_message
                    .set(Some(format!("Failed to open project: {}", e)));
            }
        }
    }
}
