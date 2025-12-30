//! Results log section for GR2 conversion tab

use floem::prelude::*;
use floem::text::Weight;
use floem::views::{VirtualDirection, VirtualItemSize, virtual_list};
use im::Vector as ImVector;

use crate::state::Gr2State;
use super::sections::card_style;

pub fn results_section(state: Gr2State) -> impl IntoView {
    let state_clear = state.clone();
    let show_failures_only = RwSignal::new(false);

    // Filtered results based on toggle
    let filtered_results = move || {
        let log = state.results_log.get();
        let filter = show_failures_only.get();
        if filter {
            log.into_iter()
                .filter(|msg| msg.starts_with("Error") || msg.starts_with("Failed"))
                .collect::<ImVector<_>>()
        } else {
            log
        }
    };

    // Count failures for badge
    let failure_count = move || {
        state.results_log.get()
            .iter()
            .filter(|msg| msg.starts_with("Error") || msg.starts_with("Failed"))
            .count()
    };

    v_stack((
        h_stack((
            label(|| "Results Log")
                .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD)),
            empty().style(|s| s.flex_grow(1.0)),
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
                    .padding_vert(4.0)
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
                    state_clear.clear_results();
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
                    container(
                        label(move || msg.clone())
                            .style(move |s| {
                                let s = s.font_size(11.0)
                                    .font_family("monospace".to_string());
                                if is_error {
                                    s.color(Color::rgb8(180, 30, 30))
                                } else {
                                    s.color(Color::rgb8(46, 125, 50))
                                }
                            }),
                    )
                    .style(move |s| {
                        let s = s.width_full()
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
