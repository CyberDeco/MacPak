//! Dialogue tab toolbar

use std::path::PathBuf;
use floem::prelude::*;
use floem::reactive::SignalGet;
use crate::gui::state::{ConfigState, DialogueState};
use super::operations;

/// Expand all nodes in the tree
fn expand_all_nodes(state: &DialogueState) {
    let nodes = state.display_nodes.get();
    for node in nodes.iter() {
        node.is_expanded.set(true);
        node.is_visible.set(true); // All nodes visible when fully expanded
    }
}

/// Collapse all nodes in the tree
fn collapse_all_nodes(state: &DialogueState) {
    let nodes = state.display_nodes.get();
    for node in nodes.iter() {
        node.is_expanded.set(false);
        // Only root nodes (depth 0) remain visible when collapsed
        node.is_visible.set(node.depth == 0);
    }
}

/// Toolbar with open, language, and export controls
pub fn toolbar(state: DialogueState, config: ConfigState) -> impl IntoView {
    let state_for_gustav = state.clone();
    let state_for_shared = state.clone();
    let config_for_gustav = config.clone();
    let config_for_shared = config.clone();
    let state_for_expand = state.clone();
    let state_for_collapse = state.clone();
    let state_for_status = state.clone();

    h_stack((
        // Load from Gustav.pak button
        button("Load from Gustav.pak")
            .style(|s| {
                s.padding_horiz(12.0)
                    .padding_vert(6.0)
                    .border_radius(4.0)
                    .font_size(13.0)
                    .background(Color::rgb8(59, 130, 246))
                    .color(Color::WHITE)
                    .hover(|s| s.background(Color::rgb8(37, 99, 235)))
            })
            .action(move || {
                let bg3_path = config_for_gustav.bg3_data_path.get();
                if !bg3_path.is_empty() {
                    let gustav_path = PathBuf::from(&bg3_path).join("Gustav.pak");
                    operations::load_pak_directly(state_for_gustav.clone(), gustav_path);
                } else {
                    state_for_gustav.status_message.set("BG3 path not configured. Set it in Preferences (⌘,)".to_string());
                }
            }),

        // Load from Shared.pak button
        button("Load from Shared.pak")
            .style(|s| {
                s.padding_horiz(12.0)
                    .padding_vert(6.0)
                    .border_radius(4.0)
                    .font_size(13.0)
                    .background(Color::rgb8(107, 114, 128))
                    .color(Color::WHITE)
                    .hover(|s| s.background(Color::rgb8(75, 85, 99)))
            })
            .action(move || {
                let bg3_path = config_for_shared.bg3_data_path.get();
                if !bg3_path.is_empty() {
                    let shared_path = PathBuf::from(&bg3_path).join("Shared.pak");
                    operations::load_pak_directly(state_for_shared.clone(), shared_path);
                } else {
                    state_for_shared.status_message.set("BG3 path not configured. Set it in Preferences (⌘,)".to_string());
                }
            }),

        // Language selector
        language_selector(state.clone()),

        // Expand/Collapse All buttons
        button("Expand All")
            .style(|s| {
                s.padding_horiz(8.0)
                    .padding_vert(5.0)
                    .border_radius(4.0)
                    .font_size(12.0)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .background(Color::WHITE)
                    .color(Color::rgb8(60, 60, 60))
                    .hover(|s| s.background(Color::rgb8(245, 245, 245)))
            })
            .action(move || {
                expand_all_nodes(&state_for_expand);
            }),

        button("Collapse All")
            .style(|s| {
                s.padding_horiz(8.0)
                    .padding_vert(5.0)
                    .border_radius(4.0)
                    .font_size(12.0)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .background(Color::WHITE)
                    .color(Color::rgb8(60, 60, 60))
                    .hover(|s| s.background(Color::rgb8(245, 245, 245)))
            })
            .action(move || {
                collapse_all_nodes(&state_for_collapse);
            }),

        // Spacer
        empty().style(|s| s.flex_grow(1.0)),

        // Export buttons (disabled when no dialog loaded)
        export_buttons(state.clone()),

        // Status message
        label(move || state_for_status.status_message.get())
            .style(|s| {
                s.font_size(12.0)
                    .color(Color::rgb8(100, 100, 100))
                    .margin_left(12.0)
            }),
    ))
    .style(|s| {
        s.width_full()
            .min_height(56.0)
            .padding_horiz(12.0)
            .padding_vert(8.0)
            .gap(8.0)
            .items_center()
            .background(Color::rgb8(245, 245, 245))
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

/// Language selector dropdown
fn language_selector(state: DialogueState) -> impl IntoView {
    let state_for_label = state.clone();

    h_stack((
        label(|| "Language:")
            .style(|s| s.font_size(13.0).color(Color::rgb8(80, 80, 80))),
        label(move || state_for_label.language.get())
            .style(|s| {
                s.font_size(13.0)
                    .padding_horiz(8.0)
                    .padding_vert(4.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .background(Color::WHITE)
            }),
    ))
    .style(|s| s.gap(4.0).items_center())
}

/// Export buttons
fn export_buttons(state: DialogueState) -> impl IntoView {
    let has_dialog = state.current_dialog;
    let state_for_html = state.clone();
    let state_for_de2 = state.clone();

    h_stack((
        button("Export HTML")
            .style(move |s| {
                let base = s
                    .padding_horiz(10.0)
                    .padding_vert(5.0)
                    .border_radius(4.0)
                    .font_size(12.0)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200));

                if has_dialog.get().is_some() {
                    base.background(Color::WHITE)
                        .color(Color::rgb8(60, 60, 60))
                        .hover(|s| s.background(Color::rgb8(245, 245, 245)))
                } else {
                    base.background(Color::rgb8(240, 240, 240))
                        .color(Color::rgb8(180, 180, 180))
                                        }
            })
            .action(move || {
                if state_for_html.current_dialog.get().is_some() {
                    operations::export_html(state_for_html.clone());
                }
            }),

        button("Export DE2")
            .style(move |s| {
                let base = s
                    .padding_horiz(10.0)
                    .padding_vert(5.0)
                    .border_radius(4.0)
                    .font_size(12.0)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200));

                if has_dialog.get().is_some() {
                    base.background(Color::WHITE)
                        .color(Color::rgb8(60, 60, 60))
                        .hover(|s| s.background(Color::rgb8(245, 245, 245)))
                } else {
                    base.background(Color::rgb8(240, 240, 240))
                        .color(Color::rgb8(180, 180, 180))
                                        }
            })
            .action(move || {
                if state_for_de2.current_dialog.get().is_some() {
                    operations::export_de2(state_for_de2.clone());
                }
            }),
    ))
    .style(|s| s.gap(4.0))
}
