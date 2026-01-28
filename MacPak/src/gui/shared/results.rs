//! Shared results log section for batch operations

use floem::prelude::*;
use floem::text::Weight;
use floem::views::{VirtualDirection, VirtualItemSize, virtual_list};
use im::Vector as ImVector;

use super::BatchOperationState;
use super::styles::card_style;

/// Generic results log section that works with any BatchOperationState.
///
/// Displays a scrollable, virtualized list of operation results with:
/// - Color-coded success (green) and failure (red) messages
/// - "Show Failures Only" filter toggle with failure count badge
/// - Clear button to reset the log
pub fn results_section<S: BatchOperationState>(state: S) -> impl IntoView {
    let state_for_clear = state.clone();
    let state_for_log = state.clone();
    let state_for_filter = state.clone();
    let state_for_filter2 = state.clone();
    let show_failures_only = RwSignal::new(false);

    // Filtered results based on toggle
    let filtered_results = move || {
        let log = state_for_log.results_log().get();
        let filter = show_failures_only.get();
        if filter {
            log.into_iter()
                .filter(|msg| msg.starts_with("Error") || msg.starts_with("Failed"))
                .collect::<ImVector<_>>()
        } else {
            log
        }
    };

    v_stack((
        h_stack((
            label(|| "Results Log").style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD)),
            empty().style(|s| s.flex_grow(1.0)),
            // Show Failures Only toggle button
            button(label(move || {
                let count = state_for_filter
                    .results_log()
                    .get()
                    .iter()
                    .filter(|msg| msg.starts_with("Error") || msg.starts_with("Failed"))
                    .count();
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
                let has_failures = state_for_filter2
                    .results_log()
                    .get()
                    .iter()
                    .any(|msg| msg.starts_with("Error") || msg.starts_with("Failed"));
                let s = s
                    .padding_horiz(10.0)
                    .padding_vert(4.0)
                    .font_size(11.0)
                    .border_radius(4.0)
                    .margin_right(8.0);

                if is_active {
                    s.background(Color::rgb8(211, 47, 47)).color(Color::WHITE)
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
            button("Clear")
                .style(|s| {
                    s.padding_horiz(12.0)
                        .padding_vert(4.0)
                        .font_size(11.0)
                        .background(Color::rgb8(240, 240, 240))
                        .border_radius(4.0)
                        .hover(|s| s.background(Color::rgb8(220, 220, 220)))
                })
                .action(move || {
                    state_for_clear.clear_results();
                    show_failures_only.set(false);
                }),
        ))
        .style(|s| s.width_full().margin_bottom(8.0)),
        scroll(
            virtual_list(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| 22.0)),
                filtered_results,
                |msg: &String| msg.clone(),
                |msg| {
                    let is_error = msg.starts_with("Error") || msg.starts_with("Failed");
                    container(label(move || msg.clone()).style(move |s| {
                        let s = s.font_size(11.0).font_family("monospace".to_string());
                        if is_error {
                            s.color(Color::rgb8(180, 30, 30))
                        } else {
                            s.color(Color::rgb8(46, 125, 50))
                        }
                    }))
                    .style(move |s| {
                        let s = s
                            .width_full()
                            .height(22.0)
                            .padding_vert(2.0)
                            .padding_horiz(4.0);
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
                .height_full()
                .min_height(0.0)
                .flex_grow(1.0)
                .flex_basis(0.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        card_style(s)
            .height_full()
            .min_height(0.0)
            .flex_grow(1.0)
            .flex_basis(0.0)
    })
}
