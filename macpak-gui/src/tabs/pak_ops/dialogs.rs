//! Dialog overlays for PAK operations

use floem::prelude::*;
use floem::text::Weight;
use std::path::Path;

use crate::state::PakOpsState;
use super::operations::{execute_create_pak, extract_dropped_file, list_dropped_file};
use super::widgets::{compression_selector, priority_input};

/// Progress overlay shown during long-running operations
pub fn progress_overlay(state: PakOpsState) -> impl IntoView {
    let show = state.show_progress;
    let progress = state.progress;
    let message = state.progress_message;

    dyn_container(
        move || show.get(),
        move |visible| {
            if visible {
                let progress = progress;
                let message = message;

                v_stack((
                    label(move || message.get()).style(|s| {
                        s.font_size(14.0)
                            .font_weight(Weight::MEDIUM)
                            .margin_bottom(12.0)
                    }),
                    // Progress bar
                    h_stack((
                        container(empty().style(move |s| {
                            let pct = progress.get();
                            s.width_pct(pct as f64 * 100.0)
                                .height_full()
                                .background(Color::rgb8(33, 150, 243))
                                .border_radius(4.0)
                        }))
                        .style(|s| {
                            s.flex_grow(1.0)
                                .height(8.0)
                                .background(Color::rgb8(230, 230, 230))
                                .border_radius(4.0)
                        }),
                        label(move || format!("{:.0}%", progress.get() * 100.0))
                            .style(|s| s.width(50.0).font_size(12.0)),
                    ))
                    .style(|s| s.width_full().gap(8.0).items_center()),
                ))
                .style(|s| {
                    s.padding(20.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .width(400.0)
                })
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
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

/// Dialog for PAK creation options (compression, priority)
pub fn create_options_dialog(state: PakOpsState) -> impl IntoView {
    let show = state.show_create_options;
    let compression = state.compression;
    let priority = state.priority;
    let pending = state.pending_create;
    let state_confirm = state.clone();
    let state_cancel = state.clone();

    dyn_container(
        move || show.get(),
        move |visible| {
            if visible {
                let compression = compression;
                let priority = priority;
                let state_confirm = state_confirm.clone();
                let state_cancel = state_cancel.clone();

                v_stack((
                    // Title
                    label(|| "PAK Creation Options".to_string()).style(|s| {
                        s.font_size(18.0)
                            .font_weight(Weight::BOLD)
                            .margin_bottom(16.0)
                    }),
                    // Compression selector
                    h_stack((
                        label(|| "Compression:".to_string()).style(|s| s.width(120.0)),
                        compression_selector(compression),
                    ))
                    .style(|s| s.width_full().items_center().margin_bottom(12.0)),
                    // Priority input
                    h_stack((
                        label(|| "Load Priority:".to_string()).style(|s| s.width(120.0)),
                        priority_input(priority),
                    ))
                    .style(|s| s.width_full().items_center().margin_bottom(12.0)),
                    // Help text
                    label(|| {
                        "lz4hc = best compression (default)\n\
                         lz4 = fast compression\n\
                         Priority 0 = normal mod, 50+ = override mod"
                            .to_string()
                    })
                    .style(|s| {
                        s.font_size(11.0)
                            .color(Color::rgb8(100, 100, 100))
                            .margin_bottom(16.0)
                    }),
                    // Buttons
                    h_stack((
                        empty().style(|s| s.flex_grow(1.0)),
                        button("Cancel")
                            .action(move || {
                                state_cancel.show_create_options.set(false);
                                state_cancel.pending_create.set(None);
                            })
                            .style(|s| {
                                s.padding_vert(8.0)
                                    .padding_horiz(20.0)
                                    .margin_right(8.0)
                                    .background(Color::rgb8(240, 240, 240))
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                            }),
                        button("Create PAK")
                            .action(move || {
                                if let Some((source, dest)) = pending.get() {
                                    state_confirm.show_create_options.set(false);
                                    execute_create_pak(state_confirm.clone(), source, dest);
                                }
                            })
                            .style(|s| {
                                s.padding_vert(8.0)
                                    .padding_horiz(20.0)
                                    .background(Color::rgb8(33, 150, 243))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                                    .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                            }),
                    ))
                    .style(|s| s.width_full()),
                ))
                .style(|s| {
                    s.padding(24.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .width(400.0)
                })
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
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

/// Dialog shown when a file is dropped, asking what action to take
pub fn drop_action_dialog(state: PakOpsState) -> impl IntoView {
    let show = state.show_drop_dialog;
    let dropped_file = state.dropped_file;

    let state_extract = state.clone();
    let state_list = state.clone();
    let state_cancel = state.clone();

    dyn_container(
        move || show.get(),
        move |visible| {
            if visible {
                let file_path = dropped_file.get().unwrap_or_default();
                let file_name = Path::new(&file_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "PAK file".to_string());

                let state_extract = state_extract.clone();
                let state_list = state_list.clone();
                let state_cancel = state_cancel.clone();
                let file_path_extract = file_path.clone();
                let file_path_list = file_path.clone();

                v_stack((
                    // Title
                    label(move || format!("Dropped: {}", file_name)).style(|s| {
                        s.font_size(16.0)
                            .font_weight(Weight::BOLD)
                            .margin_bottom(16.0)
                    }),
                    // Action buttons
                    label(|| "What would you like to do?".to_string())
                        .style(|s| s.margin_bottom(12.0)),
                    // Extract button
                    button("📦 Extract PAK")
                        .action(move || {
                            state_extract.show_drop_dialog.set(false);
                            extract_dropped_file(state_extract.clone(), file_path_extract.clone());
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .margin_bottom(8.0)
                                .background(Color::rgb8(33, 150, 243))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                        }),
                    // List button
                    button("📋 List Contents")
                        .action(move || {
                            state_list.show_drop_dialog.set(false);
                            list_dropped_file(state_list.clone(), file_path_list.clone());
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .margin_bottom(8.0)
                                .background(Color::rgb8(76, 175, 80))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(56, 142, 60)))
                        }),
                    // Cancel button
                    button("Cancel")
                        .action(move || {
                            state_cancel.show_drop_dialog.set(false);
                            state_cancel.dropped_file.set(None);
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .background(Color::rgb8(240, 240, 240))
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0)
                        }),
                ))
                .style(|s| {
                    s.padding(24.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .width(320.0)
                })
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
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
