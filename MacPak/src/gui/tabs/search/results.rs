//! Search results display components

use floem::prelude::*;
use floem::text::Weight;
use floem::views::{VirtualDirection, VirtualItemSize};
use im::Vector as ImVector;
use MacLarian::search::FileType;

use crate::gui::state::{IndexStatus, SearchResult, SearchState};

use super::operations::copy_to_clipboard;

/// Fixed row height for virtual list (must accommodate context snippets from deep search)
const RESULT_ROW_HEIGHT: f64 = 72.0;

pub fn search_results(state: SearchState, active_filter: RwSignal<Option<FileType>>) -> impl IntoView {
    let results = state.results;
    let is_searching = state.is_searching;
    let index_status = state.index_status;

    // Create a derived signal that filters results for count display
    let filtered_results = move || {
        let all_results = results.get();
        let filter = active_filter.get();

        match filter {
            None => all_results,
            Some(ft) => {
                let filter_name = ft.display_name().to_lowercase();
                all_results
                    .into_iter()
                    .filter(|r| r.file_type.to_lowercase() == filter_name)
                    .collect()
            }
        }
    };

    v_stack((
        // Results count
        h_stack((
            empty().style(|s| s.flex_grow(1.0)),
            label(move || {
                let filtered = filtered_results();
                let total = results.get().len();
                if filtered.len() == total {
                    format!("{} matches", total)
                } else {
                    format!("{} of {} matches", filtered.len(), total)
                }
            })
            .style(|s| s.color(Color::rgb8(128, 128, 128)).font_size(12.0)),
        ))
        .style(|s| {
            s.width_full()
                .padding(8.0)
                .background(Color::rgb8(250, 250, 250))
                .border_bottom(1.0)
                .border_color(Color::rgb8(220, 220, 220))
        }),

        // Status overlay for empty/loading states
        dyn_container(
            move || (is_searching.get(), index_status.get(), results.get().is_empty()),
            move |(searching, status, no_results)| {
                if searching {
                    label(|| "Searching...")
                        .style(|s| {
                            s.width_full()
                                .padding(40.0)
                                .items_center()
                                .color(Color::rgb8(128, 128, 128))
                        })
                        .into_any()
                } else if !matches!(status, IndexStatus::Ready { .. }) {
                    v_stack((
                        label(|| "No Index Built")
                            .style(|s| s.font_size(18.0).font_weight(Weight::SEMIBOLD).color(Color::rgb8(80, 80, 80))),
                        label(|| "Click 'Build Index' to index PAK files from your BG3 installation.")
                            .style(|s| s.color(Color::rgb8(120, 120, 120)).margin_top(8.0)),
                    ))
                    .style(|s| {
                        s.width_full()
                            .padding(40.0)
                            .items_center()
                            .justify_center()
                    })
                    .into_any()
                } else if no_results {
                    v_stack((
                        label(|| "Ready to Search")
                            .style(|s| s.font_size(18.0).font_weight(Weight::SEMIBOLD).color(Color::rgb8(80, 80, 80))),
                        label(|| "Search by filename, UUID, or enable Deep Search for content.")
                            .style(|s| s.color(Color::rgb8(120, 120, 120)).margin_top(8.0)),
                        label(|| "Use filters to narrow results by file type.")
                            .style(|s| s.color(Color::rgb8(140, 140, 140)).margin_top(4.0)),
                    ))
                    .style(|s| {
                        s.width_full()
                            .padding(40.0)
                            .items_center()
                            .justify_center()
                    })
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),

        // Results list
        scroll(
            virtual_list(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| RESULT_ROW_HEIGHT)),
                move || filtered_results().into_iter().collect::<ImVector<_>>(),
                |result| result.path.clone(),
                move |result| search_result_row(result),
            )
            .style(|s| s.width_full().flex_col()),
        )
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .min_height(0.0)
            .background(Color::WHITE)
    })
}

fn search_result_row(result: SearchResult) -> impl IntoView {
    let icon = get_type_icon(&result.file_type);
    let has_context = result.context.is_some();
    let context_text = result.context.clone().unwrap_or_default();
    let line_num = result.line_number;

    // Clone values that need to be used in multiple closures
    let name = result.name.clone();
    let path_display = result.path.clone();
    let path_copy = result.path.clone();
    let pak_file = result.pak_file.clone();

    v_stack((
        // Main row
        h_stack((
            // Icon - fixed width
            label(move || icon.to_string())
                .style(|s| s.width(30.0).flex_shrink(0.0)),

            // File info - flexible, can shrink with text ellipsis
            v_stack((
                label(move || name.clone())
                    .style(|s| {
                        s.font_weight(Weight::MEDIUM)
                            .text_ellipsis()
                            .min_width(0.0)
                    }),
                label(move || path_display.clone())
                    .style(|s| {
                        s.font_size(12.0)
                            .color(Color::rgb8(128, 128, 128))
                            .text_ellipsis()
                            .min_width(0.0)
                    }),
            ))
            .style(|s| s.flex_grow(1.0).min_width(0.0)),

            // PAK file badge - fixed width
            label(move || pak_file.clone())
                .style(|s| {
                    s.font_size(12.0)
                        .color(Color::rgb8(100, 100, 100))
                        .padding_horiz(8.0)
                        .padding_vert(4.0)
                        .background(Color::rgb8(240, 240, 240))
                        .border_radius(4.0)
                        .flex_shrink(0.0)
                }),

            // Actions - fixed width
            button("Copy")
                .style(|s| {
                    s.padding_horiz(8.0)
                        .font_size(12.0)
                        .background(Color::rgb8(230, 230, 230))
                        .border_radius(4.0)
                        .flex_shrink(0.0)
                })
                .action(move || {
                    copy_to_clipboard(&path_copy);
                }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center().min_width(0.0)),

        // Context snippet (if present)
        dyn_container(
            move || has_context,
            move |show| {
                if show {
                    let line_label = line_num.map(|n| format!("Line {}: ", n)).unwrap_or_default();
                    let ctx = context_text.clone();
                    h_stack((
                        label(move || line_label.clone())
                            .style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100)).flex_shrink(0.0)),
                        label(move || ctx.clone())
                            .style(|s| {
                                s.font_size(11.0)
                                    .color(Color::rgb8(80, 80, 80))
                                    .padding(4.0)
                                    .background(Color::rgb8(255, 255, 230))
                                    .border_radius(2.0)
                                    .text_ellipsis()
                                    .min_width(0.0)
                            }),
                    ))
                    .style(|s| s.margin_left(30.0).margin_top(4.0).min_width(0.0))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .height(RESULT_ROW_HEIGHT)
            .min_height(RESULT_ROW_HEIGHT)
            .max_height(RESULT_ROW_HEIGHT)
            .min_width(0.0)
            .padding(12.0)
            .border_bottom(1.0)
            .border_color(Color::rgb8(240, 240, 240))
            .hover(|s| s.background(Color::rgb8(250, 252, 255)))
    })
    .on_event_stop(floem::event::EventListener::PointerDown, |_| {})
}

pub fn search_status_bar(state: SearchState) -> impl IntoView {
    let index_status = state.index_status;

    h_stack((
        // Index status
        dyn_container(
            move || index_status.get(),
            move |status| {
                let (text, color) = match status {
                    IndexStatus::NotBuilt => ("Click 'Build Index' to index PAK files".to_string(), Color::rgb8(150, 150, 150)),
                    IndexStatus::Building { progress } => (format!("Building: {}", progress), Color::rgb8(255, 152, 0)),
                    IndexStatus::Ready { file_count, pak_count } => {
                        (format!("Ready: {} files from {} PAKs", file_count, pak_count), Color::rgb8(76, 175, 80))
                    }
                    IndexStatus::Error(msg) => (format!("Error: {}", msg), Color::rgb8(244, 67, 54)),
                };

                label(move || text.clone())
                    .style(move |s| s.color(color).font_size(12.0))
                    .into_any()
            },
        ),

        empty().style(|s| s.flex_grow(1.0)),

        // Hint about preferences
        label(|| "BG3 path configured in Preferences")
            .style(|s| s.color(Color::rgb8(130, 130, 130)).font_size(11.0)),
    ))
    .style(|s| {
        s.width_full()
            .height(32.0)
            .padding_horiz(12.0)
            .items_center()
            .background(Color::rgb8(248, 248, 248))
            .border_top(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn get_type_icon(file_type: &str) -> &'static str {
    match file_type.to_lowercase().as_str() {
        "lsx" | "xml" => "📄",
        "lsj" | "json" => "📋",
        "lsf" | "lsbc" => "🔷",
        "dds" | "image" | "png" | "jpg" => "🖼️",
        "gr2" => "🦴",
        "wem" | "audio" | "ogg" | "wav" => "🔊",
        "gts" | "gtp" => "🗺️",
        _ => "📄",
    }
}


