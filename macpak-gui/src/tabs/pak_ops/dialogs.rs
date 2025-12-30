//! Dialog overlays for PAK operations

use floem::action::exec_after;
use floem::prelude::*;
use floem::text::Weight;
use floem_reactive::create_effect;
use std::path::Path;
use std::time::Duration;

use crate::state::PakOpsState;
use super::operations::{execute_create_pak, extract_dropped_file, list_dropped_file};
use super::types::get_shared_progress;
use super::widgets::{compression_selector, priority_input};

/// Progress overlay shown during long-running operations
/// Uses a polling timer to read shared atomic state updated by background threads
pub fn progress_overlay(state: PakOpsState) -> impl IntoView {
    let show = state.show_progress;

    // Local signals for the polled values - these will be updated by the timer
    let polled_pct = RwSignal::new(0u32);
    let polled_msg = RwSignal::new(String::new());

    // Signal to track the current timer token so we can cancel it
    let timer_active = RwSignal::new(false);

    // Animation counter for indeterminate progress (0-100, bounces back and forth)
    let anim_pos = RwSignal::new(0i32);
    let anim_direction = RwSignal::new(1i32); // 1 = forward, -1 = backward

    // Function to poll shared progress and schedule next poll
    fn poll_and_schedule(
        polled_pct: RwSignal<u32>,
        polled_msg: RwSignal<String>,
        anim_pos: RwSignal<i32>,
        anim_direction: RwSignal<i32>,
        show: RwSignal<bool>,
        timer_active: RwSignal<bool>,
    ) {
        // Read from shared atomic state
        let shared = get_shared_progress();
        let pct = shared.get_pct();
        let msg = shared.get_message();

        // Update local signals
        polled_pct.set(pct);
        if !msg.is_empty() {
            polled_msg.set(msg);
        }

        // Update animation position for indeterminate progress
        let current_pos = anim_pos.get_untracked();
        let dir = anim_direction.get_untracked();
        let new_pos = current_pos + dir * 3; // Move 3% per frame

        if new_pos >= 70 {
            anim_direction.set(-1);
            anim_pos.set(70);
        } else if new_pos <= 0 {
            anim_direction.set(1);
            anim_pos.set(0);
        } else {
            anim_pos.set(new_pos);
        }

        // Schedule next poll if still active
        if show.get_untracked() && timer_active.get_untracked() {
            exec_after(Duration::from_millis(30), move |_| {
                if show.get_untracked() && timer_active.get_untracked() {
                    poll_and_schedule(polled_pct, polled_msg, anim_pos, anim_direction, show, timer_active);
                }
            });
        }
    }

    // Start/stop polling timer based on visibility
    create_effect(move |_| {
        let visible = show.get();
        if visible {
            // Reset and start polling immediately
            polled_pct.set(0);
            polled_msg.set(String::new());
            anim_pos.set(0);
            anim_direction.set(1);
            timer_active.set(true);
            // Do first poll immediately, then schedule subsequent ones
            poll_and_schedule(polled_pct, polled_msg, anim_pos, anim_direction, show, timer_active);
        } else {
            // Stop polling
            timer_active.set(false);
        }
    });

    dyn_container(
        move || {
            let visible = show.get();
            let pct = polled_pct.get();
            let msg = polled_msg.get();
            let anim = anim_pos.get();
            (visible, pct, msg, anim)
        },
        move |(visible, pct, msg, anim)| {
            if visible {
                // Determine if we're in indeterminate mode (reading phase with low/no progress)
                let is_reading = msg.contains("Reading");
                let is_indeterminate = is_reading && pct < 100;

                v_stack((
                    label(move || {
                        if is_indeterminate {
                            msg.clone()
                        } else if pct >= 100 {
                            "Finishing...".to_string()
                        } else {
                            msg.clone()
                        }
                    }).style(|s| {
                        s.font_size(14.0)
                            .font_weight(Weight::MEDIUM)
                            .margin_bottom(12.0)
                    }),
                    // Progress bar
                    h_stack((
                        container(
                            container(empty()).style(move |s| {
                                if is_indeterminate {
                                    // Animated sliding bar for indeterminate progress
                                    s.width_pct(30.0)
                                        .height_full()
                                        .background(Color::rgb8(33, 150, 243))
                                        .border_radius(4.0)
                                        .margin_left_pct(anim as f64)
                                } else {
                                    // Normal progress bar
                                    let display_pct = if pct > 100 { 100 } else { pct };
                                    s.width_pct(display_pct as f64)
                                        .height_full()
                                        .background(Color::rgb8(33, 150, 243))
                                        .border_radius(4.0)
                                }
                            }),
                        )
                        .style(|s| {
                            s.flex_grow(1.0)
                                .height(8.0)
                                .background(Color::rgb8(230, 230, 230))
                                .border_radius(4.0)
                        }),
                        label(move || {
                            if is_indeterminate {
                                "...".to_string()
                            } else {
                                format!("{}%", if pct > 100 { 100 } else { pct })
                            }
                        })
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
                        .width(500.0)
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
