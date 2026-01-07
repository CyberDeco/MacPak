//! Results area UI component

use floem::prelude::*;
use floem::style::AlignItems;
use floem::text::Weight;
use floem::views::{text_input, virtual_list, VirtualDirection, VirtualItemSize};
use im::Vector as ImVector;

use crate::gui::state::PakOpsState;

const LOG_ITEM_HEIGHT: f64 = 22.0;

/// Check if a message indicates an error/failure (not just containing the word in a filename).
/// Matches actual error patterns from MacLarian errors and GUI status messages.
pub fn is_error_message(msg: &str) -> bool {
    // Check for emoji indicators (most reliable)
    if msg.starts_with('❌') || msg.starts_with('⚠') {
        return true;
    }

    // GUI status message patterns
    if msg.contains("Error: ") || msg.contains("Failed: ") || msg.contains("Failed to ") {
        return true;
    }

    // MacLarian error patterns (from error.rs #[error("...")] attributes)
    msg.contains(" error: ")         // "IO error: ", "XML parse error: ", "JSON error: ", etc.
        || msg.contains(" failed: ") // "Decompression failed: ", "Compression failed: "
        || msg.starts_with("Invalid ")
        || msg.starts_with("Unsupported ")
        || msg.starts_with("Unexpected ")
        || msg.starts_with("File not found ")
}

/// Results area - unified log showing operations and file listings
pub fn results_area(state: PakOpsState) -> impl IntoView {
    results_log_section(state)
}

/// Results log section with filtering, search, and virtual list
fn results_log_section(state: PakOpsState) -> impl IntoView {
    let state_clear = state.clone();
    let show_failures_only = RwSignal::new(false);
    let file_search = state.file_search;

    // Filtered results based on toggle and search
    let filtered_results = move || {
        let log = state.results_log.get();
        let filter_failures = show_failures_only.get();
        let search = file_search.get();
        let search_lower = search.to_lowercase();

        log.into_iter()
            .filter(|msg| {
                // Apply failure filter
                if filter_failures {
                    if !is_error_message(msg) {
                        return false;
                    }
                }
                // Apply search filter
                if !search.is_empty() {
                    return msg.to_lowercase().contains(&search_lower);
                }
                true
            })
            .collect::<ImVector<_>>()
    };

    // Count for display
    let total_count = move || state.results_log.get().len();
    let filtered_count = move || filtered_results().len();

    // Count failures for badge
    let failure_count = move || {
        state.results_log.get()
            .iter()
            .filter(|msg| is_error_message(msg))
            .count()
    };

    v_stack((
        // Header row
        h_stack((
            // Count label
            dyn_container(
                move || (filtered_count(), total_count()),
                move |(filtered, total)| {
                    let text = if filtered == total {
                        if total == 0 {
                            "Results Log".to_string()
                        } else {
                            format!("Results Log ({} items)", total)
                        }
                    } else {
                        format!("Results Log ({} of {} items)", filtered, total)
                    };
                    label(move || text.clone())
                        .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD))
                },
            ),
            empty().style(|s| s.flex_grow(1.0)),
            // Search input
            text_input(file_search)
                .placeholder("Search...")
                .style(|s| {
                    s.width(180.0)
                        .height(26.0)
                        .padding_horiz(8.0)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                        .font_size(12.0)
                        .background(Color::WHITE)
                        .margin_right(8.0)
                }),
            // Show Failures Only toggle button
            button(label(move || {
                let count = failure_count();
                if show_failures_only.get() {
                    "Show All".to_string()
                } else if count > 0 {
                    format!("Failures ({})", count)
                } else {
                    "Failures".to_string()
                }
            }))
            .style(move |s| {
                let is_active = show_failures_only.get();
                let has_failures = failure_count() > 0;
                let s = s
                    .padding_horiz(10.0)
                    .padding_vert(5.0)
                    .font_size(11.0)
                    .border_radius(4.0)
                    .margin_right(8.0);

                if is_active {
                    s.background(Color::rgb8(211, 47, 47))
                        .color(Color::WHITE)
                } else if has_failures {
                    s.background(Color::rgb8(255, 235, 235))
                        .color(Color::rgb8(180, 30, 30))
                        .hover(|s| s.background(Color::rgb8(255, 220, 220)))
                } else {
                    s.background(Color::rgb8(240, 240, 240))
                        .color(Color::rgb8(150, 150, 150))
                }
            })
            .action(move || {
                show_failures_only.set(!show_failures_only.get());
            }),
            // Clear button
            button("Clear")
                .style(|s| {
                    s.padding_horiz(12.0)
                        .padding_vert(5.0)
                        .font_size(11.0)
                        .background(Color::rgb8(240, 240, 240))
                        .border_radius(4.0)
                        .hover(|s| s.background(Color::rgb8(220, 220, 220)))
                })
                .action(move || {
                    state_clear.clear_results();
                    show_failures_only.set(false);
                    file_search.set(String::new());
                }),
        ))
        .style(|s| s.width_full().margin_bottom(8.0).gap(4.0).align_items(AlignItems::Center)),
        // Virtual list for results
        scroll(
            virtual_list(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| LOG_ITEM_HEIGHT)),
                filtered_results,
                |msg: &String| msg.clone(),
                |msg| {
                    let is_error = is_error_message(&msg);
                    let is_success = msg.starts_with('✅') || msg.starts_with('✓');

                    container(
                        label(move || msg.clone())
                            .style(move |s| {
                                let s = s.font_size(11.0)
                                    .font_family("Monaco, Menlo, monospace".to_string());
                                if is_error {
                                    s.color(Color::rgb8(180, 30, 30))
                                } else if is_success {
                                    s.color(Color::rgb8(46, 125, 50))
                                } else {
                                    s.color(Color::rgb8(60, 60, 60))
                                }
                            }),
                    )
                    .style(move |s| {
                        let s = s.width_full()
                            .height(LOG_ITEM_HEIGHT)
                            .padding_vert(2.0)
                            .padding_horiz(8.0);
                        if is_error {
                            s.background(Color::rgb8(255, 235, 235))
                        } else {
                            s
                        }
                    })
                },
            )
            .style(|s| s.flex_col().width_full()),
        )
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .min_height(0.0)
            .flex_grow(1.0)
            .flex_basis(0.0)
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
    })
}
