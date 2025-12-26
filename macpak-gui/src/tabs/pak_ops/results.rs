//! Results area UI component

use floem::prelude::*;
use floem::text::Weight;

use crate::state::PakOpsState;

/// Results area showing operation output
pub fn results_area(state: PakOpsState) -> impl IntoView {
    let results = state.results_log;
    let state_clear = state.clone();

    v_stack((
        // Header row with title and clear button
        h_stack((
            label(|| "Operation Results".to_string()).style(|s| {
                s.font_size(16.0).font_weight(Weight::SEMIBOLD)
            }),
            empty().style(|s| s.flex_grow(1.0)),
            button("Clear Results")
                .action(move || {
                    state_clear.clear_results();
                })
                .style(|s| {
                    s.padding_vert(4.0)
                        .padding_horiz(12.0)
                        .font_size(12.0)
                        .background(Color::rgb8(245, 245, 245))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                        .hover(|s| s.background(Color::rgb8(230, 230, 230)))
                }),
        ))
        .style(|s| s.width_full().items_center().margin_bottom(8.0)),
        // Results text area
        scroll(dyn_container(
            move || results.get(),
            move |log| {
                if log.is_empty() {
                    label(|| "Operation results will appear here...".to_string())
                        .style(|s| {
                            s.color(Color::rgb8(150, 150, 150))
                                .font_size(13.0)
                                .padding(8.0)
                        })
                        .into_any()
                } else {
                    dyn_stack(
                        move || log.clone(),
                        |line| line.clone(),
                        |line| {
                            let is_success = line.starts_with('✅') || line.starts_with('✓');
                            let is_error = line.starts_with('❌') || line.starts_with('⚠');
                            let is_separator = line.starts_with('-') && line.len() > 10;

                            label(move || line.clone()).style(move |s| {
                                let mut s = s
                                    .width_full()
                                    .padding_vert(2.0)
                                    .padding_horiz(8.0)
                                    .font_size(12.0)
                                    .font_family("Monaco, Menlo, monospace".to_string());

                                if is_success {
                                    s = s.color(Color::rgb8(46, 125, 50));
                                } else if is_error {
                                    s = s.color(Color::rgb8(211, 47, 47));
                                } else if is_separator {
                                    s = s.color(Color::rgb8(180, 180, 180));
                                }
                                s
                            })
                        },
                    )
                    .style(|s| s.width_full().flex_col())
                    .into_any()
                }
            },
        ))
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .min_height(200.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}
