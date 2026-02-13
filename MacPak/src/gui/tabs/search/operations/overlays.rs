//! Progress overlay UI components

use std::time::Duration;

use floem::action::exec_after;
use floem::prelude::*;
use floem::text::Weight;
use floem_reactive::create_effect;

use crate::gui::state::SearchState;

use super::progress::SEARCH_PROGRESS;

/// Shared progress overlay with polling-based progress display.
///
/// `title` — header text (e.g. "Indexing..." or "Searching...")
/// `show` — signal that controls visibility
/// `reset_on_show` — whether to call `SEARCH_PROGRESS.reset()` when shown
/// `initial_msg` — message shown before first poll update
fn search_progress_overlay(
    title: &'static str,
    show: RwSignal<bool>,
    reset_on_show: bool,
    initial_msg: &'static str,
) -> impl IntoView {
    // Local signals for polled values
    let polled_current = RwSignal::new(0usize);
    let polled_total = RwSignal::new(0usize);
    let polled_msg = RwSignal::new(String::new());
    let polled_pct = RwSignal::new(0u32);
    let timer_active = RwSignal::new(false);

    // Polling function
    fn poll_and_schedule(
        polled_current: RwSignal<usize>,
        polled_total: RwSignal<usize>,
        polled_msg: RwSignal<String>,
        polled_pct: RwSignal<u32>,
        show: RwSignal<bool>,
        timer_active: RwSignal<bool>,
    ) {
        let (current, total, msg) = SEARCH_PROGRESS.get();
        polled_current.set(current);
        polled_total.set(total);
        if !msg.is_empty() {
            polled_msg.set(msg);
        }
        if total > 0 {
            polled_pct.set(((current as f64 / total as f64) * 100.0) as u32);
        }

        // Schedule next poll if still active
        if show.get_untracked() && timer_active.get_untracked() {
            exec_after(Duration::from_millis(50), move |_| {
                if show.get_untracked() && timer_active.get_untracked() {
                    poll_and_schedule(
                        polled_current,
                        polled_total,
                        polled_msg,
                        polled_pct,
                        show,
                        timer_active,
                    );
                }
            });
        }
    }

    // Start/stop polling based on visibility
    create_effect(move |_| {
        let visible = show.get();
        if visible {
            if reset_on_show {
                SEARCH_PROGRESS.reset();
            }
            polled_current.set(0);
            polled_total.set(0);
            polled_msg.set(initial_msg.to_string());
            polled_pct.set(0);
            timer_active.set(true);

            exec_after(Duration::from_millis(50), move |_| {
                if show.get_untracked() {
                    poll_and_schedule(
                        polled_current,
                        polled_total,
                        polled_msg,
                        polled_pct,
                        show,
                        timer_active,
                    );
                }
            });
        } else {
            timer_active.set(false);
        }
    });

    dyn_container(
        move || show.get(),
        move |is_visible| {
            if is_visible {
                container(
                    v_stack((
                        // Title
                        label(move || title).style(|s| {
                            s.font_size(16.0)
                                .font_weight(Weight::BOLD)
                                .margin_bottom(12.0)
                        }),
                        // Count display (e.g., "1/5000")
                        label(move || {
                            let t = polled_total.get();
                            let c = polled_current.get();
                            if t > 0 {
                                format!("{}/{}", c, t)
                            } else {
                                String::new()
                            }
                        })
                        .style(|s| {
                            s.font_size(13.0)
                                .color(Color::rgb8(100, 100, 100))
                                .margin_bottom(4.0)
                        }),
                        // Current file being processed
                        label(move || polled_msg.get()).style(|s| {
                            s.font_size(12.0)
                                .color(Color::rgb8(120, 120, 120))
                                .margin_bottom(12.0)
                                .text_ellipsis()
                                .max_width(450.0)
                        }),
                        // Progress bar
                        container(container(empty()).style(move |s| {
                            let pct = polled_pct.get();
                            s.height_full()
                                .width_pct(pct as f64)
                                .background(Color::rgb8(33, 150, 243))
                                .border_radius(4.0)
                        }))
                        .style(|s| {
                            s.width_full()
                                .height(8.0)
                                .background(Color::rgb8(220, 220, 220))
                                .border_radius(4.0)
                        }),
                        label(move || format!("{}%", polled_pct.get())).style(|s| {
                            s.font_size(12.0)
                                .margin_top(8.0)
                                .color(Color::rgb8(100, 100, 100))
                        }),
                    ))
                    .style(|s| {
                        s.padding(24.0)
                            .background(Color::WHITE)
                            .border(1.0)
                            .border_color(Color::rgb8(200, 200, 200))
                            .border_radius(8.0)
                            .width(500.0)
                    }),
                )
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

/// Progress overlay shown during long-running indexing operations
pub fn progress_overlay(state: SearchState) -> impl IntoView {
    search_progress_overlay("Indexing...", state.show_progress, true, "Preparing...")
}

/// Overlay shown while search is in progress with progress bar
pub fn search_overlay(state: SearchState) -> impl IntoView {
    search_progress_overlay("Searching...", state.is_searching, false, "Searching...")
}
