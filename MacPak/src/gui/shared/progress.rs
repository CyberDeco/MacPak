//! Shared progress tracking and overlay for batch operations

use floem::action::exec_after;
use floem::prelude::*;
use floem::style::Position;
use floem_reactive::create_effect;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::BatchOperationState;

/// Shared progress state that can be updated from background threads
/// and polled from the UI thread.
///
/// This is a thread-safe structure using atomic operations for efficient
/// cross-thread communication without locks for numeric values.
#[derive(Clone)]
pub struct SharedProgress {
    /// Progress as integer percentage (0-100)
    pub progress_pct: Arc<AtomicU32>,
    /// Current item index (1-based for display)
    pub current: Arc<AtomicU32>,
    /// Total items
    pub total: Arc<AtomicU32>,
    /// Current progress message (filename)
    pub message: Arc<Mutex<String>>,
}

impl SharedProgress {
    pub fn new() -> Self {
        Self {
            progress_pct: Arc::new(AtomicU32::new(0)),
            current: Arc::new(AtomicU32::new(0)),
            total: Arc::new(AtomicU32::new(0)),
            message: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Update progress from background thread
    pub fn update(&self, current: usize, total: usize, description: &str) {
        let pct = if total > 0 {
            ((current as f64 / total as f64) * 100.0) as u32
        } else {
            0
        };
        self.progress_pct.store(pct, Ordering::SeqCst);
        self.current.store((current + 1) as u32, Ordering::SeqCst); // 1-based for display
        self.total.store(total as u32, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            *msg = description.to_string();
        }
    }

    /// Get current progress percentage (0-100)
    pub fn get_pct(&self) -> u32 {
        self.progress_pct.load(Ordering::SeqCst)
    }

    /// Get current/total as (current, total)
    pub fn get_counts(&self) -> (u32, u32) {
        (
            self.current.load(Ordering::SeqCst),
            self.total.load(Ordering::SeqCst),
        )
    }

    /// Get current message
    pub fn get_message(&self) -> String {
        self.message.lock().map(|m| m.clone()).unwrap_or_default()
    }

    /// Reset progress
    pub fn reset(&self) {
        self.progress_pct.store(0, Ordering::SeqCst);
        self.current.store(0, Ordering::SeqCst);
        self.total.store(0, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            msg.clear();
        }
    }
}

impl Default for SharedProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic progress overlay that works with any BatchOperationState.
///
/// Shows a centered modal with progress bar, percentage, and current file name.
/// Automatically polls the shared progress state and updates the UI.
pub fn progress_overlay<S: BatchOperationState>(state: S) -> impl IntoView {
    let show = state.is_processing();
    let shared_progress = state.get_shared_progress();

    // Local signals for polled values - updated by timer
    let polled_pct = RwSignal::new(0u32);
    let polled_current = RwSignal::new(0u32);
    let polled_total = RwSignal::new(0u32);
    let polled_msg = RwSignal::new(String::new());
    let timer_active = RwSignal::new(false);

    // Polling function
    fn poll_and_schedule(
        shared: &'static SharedProgress,
        polled_pct: RwSignal<u32>,
        polled_current: RwSignal<u32>,
        polled_total: RwSignal<u32>,
        polled_msg: RwSignal<String>,
        show: RwSignal<bool>,
        timer_active: RwSignal<bool>,
    ) {
        // Read from shared atomic state
        let pct = shared.get_pct();
        let (current, total) = shared.get_counts();
        let msg = shared.get_message();

        // Update local signals
        polled_pct.set(pct);
        polled_current.set(current);
        polled_total.set(total);
        if !msg.is_empty() {
            polled_msg.set(msg);
        }

        // Schedule next poll if still active
        if show.get_untracked() && timer_active.get_untracked() {
            exec_after(Duration::from_millis(50), move |_| {
                if show.get_untracked() && timer_active.get_untracked() {
                    poll_and_schedule(
                        shared,
                        polled_pct,
                        polled_current,
                        polled_total,
                        polled_msg,
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
            // Reset and start polling
            shared_progress.reset();
            polled_pct.set(0);
            polled_current.set(0);
            polled_total.set(0);
            polled_msg.set("Starting...".to_string());
            timer_active.set(true);

            exec_after(Duration::from_millis(50), move |_| {
                if show.get_untracked() {
                    poll_and_schedule(
                        shared_progress,
                        polled_pct,
                        polled_current,
                        polled_total,
                        polled_msg,
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
        move |is_processing| {
            if is_processing {
                container(
                    v_stack((
                        // Count display (e.g., "1/5") - only show for batch
                        label(move || {
                            let total = polled_total.get();
                            if total > 1 {
                                format!("{}/{}", polled_current.get(), total)
                            } else {
                                String::new()
                            }
                        })
                        .style(|s| {
                            s.font_size(13.0)
                                .color(Color::rgb8(100, 100, 100))
                                .margin_bottom(4.0)
                        }),
                        // Filename
                        label(move || polled_msg.get())
                            .style(|s| s.font_size(14.0).margin_bottom(12.0)),
                        // Progress bar - full width
                        container(
                            container(empty()).style(move |s| {
                                let pct = polled_pct.get();
                                s.height_full()
                                    .width_pct(pct as f64)
                                    .background(Color::rgb8(76, 175, 80))
                                    .border_radius(4.0)
                            }),
                        )
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
            s.position(Position::Absolute)
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
