//! Results area UI component

use floem::prelude::*;
use floem::style::{AlignItems, CursorStyle};
use floem::text::Weight;
use floem::views::{text_input, virtual_list, VirtualDirection, VirtualItemSize};
use floem_reactive::create_memo;
use im::Vector as ImVector;

use crate::state::PakOpsState;

/// Fixed height for each file list item (required for virtual_list)
const ITEM_HEIGHT: f64 = 24.0;

/// Results area showing file list and status bar
pub fn results_area(state: PakOpsState) -> impl IntoView {
    let list_contents = state.list_contents;
    let file_search = state.file_search;
    let results_log = state.results_log;

    v_stack((
        // File list section (shown when files are available, takes up most space)
        file_list_section(list_contents, file_search),
        // Status bar at bottom
        status_bar(results_log),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .min_height(0.0)
            .flex_grow(1.0)
            .gap(10.0)
    })
}

/// File list with search bar
fn file_list_section(
    list_contents: RwSignal<ImVector<String>>,
    file_search: RwSignal<String>,
) -> impl IntoView {
    dyn_container(
        move || list_contents.get().len(),
        move |count| {
            if count == 0 {
                // Show placeholder when no files loaded
                empty_placeholder().into_any()
            } else {
                file_list_view(list_contents, file_search, count).into_any()
            }
        },
    )
    .style(|s| {
        s.width_full()
            .height_full()
            .min_height(0.0)
            .flex_grow(1.0)
            .flex_basis(0.0)
    })
}

/// Placeholder when no PAK is loaded
fn empty_placeholder() -> impl IntoView {
    label(|| "Load a PAK file to view its contents".to_string())
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .padding(40.0)
                .color(Color::rgb8(150, 150, 150))
                .font_size(14.0)
                .justify_center()
                .items_center()
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        })
}

/// The actual file list view with search - uses virtual_list for performance
fn file_list_view(
    list_contents: RwSignal<ImVector<String>>,
    file_search: RwSignal<String>,
    total_count: usize,
) -> impl IntoView {
    // Create a derived signal for filtered files
    let filtered_files = create_memo(move |_| {
        let files = list_contents.get();
        let search = file_search.get();

        if search.is_empty() {
            files
        } else {
            let search_lower = search.to_lowercase();
            files
                .into_iter()
                .filter(|f| f.to_lowercase().contains(&search_lower))
                .collect()
        }
    });

    v_stack((
        // Header with count and search
        h_stack((
            // File count label
            dyn_container(
                move || {
                    let filtered_count = filtered_files.get().len();
                    (filtered_count, total_count)
                },
                move |(filtered, total)| {
                    let text = if filtered == total {
                        format!("{} files", total)
                    } else {
                        format!("{} of {} files", filtered, total)
                    };
                    label(move || text.clone())
                        .style(|s| {
                            s.font_size(13.0)
                                .font_weight(Weight::MEDIUM)
                                .color(Color::rgb8(60, 60, 60))
                        })
                },
            ),
            // Spacer
            empty().style(|s| s.flex_grow(1.0)),
            // Search input
            text_input(file_search)
                .placeholder("Search files...")
                .style(|s| {
                    s.width(200.0)
                        .height(28.0)
                        .padding_horiz(8.0)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                        .font_size(12.0)
                        .background(Color::WHITE)
                }),
            // Clear button
            dyn_container(
                move || !file_search.get().is_empty(),
                move |has_text| {
                    if has_text {
                        label(|| "Clear".to_string())
                            .on_click_stop(move |_| {
                                file_search.set(String::new());
                            })
                            .style(|s| {
                                s.padding_horiz(8.0)
                                    .padding_vert(4.0)
                                    .font_size(11.0)
                                    .color(Color::rgb8(100, 100, 100))
                                    .cursor(CursorStyle::Pointer)
                                    .hover(|s| s.color(Color::rgb8(50, 50, 50)))
                            })
                            .into_any()
                    } else {
                        empty().into_any()
                    }
                },
            ),
        ))
        .style(|s| {
            s.width_full()
                .align_items(AlignItems::Center)
                .padding(8.0)
                .gap(8.0)
        }),
        // Virtual file list (scrollable, handles millions of items)
        scroll(
            virtual_list(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| ITEM_HEIGHT)),
                move || filtered_files.get(),
                |file: &String| file.clone(),
                |file| {
                    label(move || file.clone())
                        .style(|s| {
                            s.width_full()
                                .height(ITEM_HEIGHT)
                                .padding_vert(3.0)
                                .padding_horiz(8.0)
                                .font_size(12.0)
                                .font_family("Monaco, Menlo, monospace".to_string())
                                .color(Color::rgb8(50, 50, 50))
                                .hover(|s| s.background(Color::rgb8(240, 240, 240)))
                        })
                },
            )
            .style(|s| s.flex_col().width_full()),
        )
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)  // Critical for scroll to work
        }),
    ))
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
    })
}

/// Status bar showing the last operation result
fn status_bar(results_log: RwSignal<Vec<String>>) -> impl IntoView {
    dyn_container(
        move || results_log.get().last().cloned(),
        move |last_msg| {
            if let Some(msg) = last_msg {
                let is_success = msg.starts_with('✅') || msg.starts_with('✓');
                let is_error = msg.starts_with('❌') || msg.starts_with('⚠');

                label(move || msg.clone())
                    .style(move |s| {
                        let mut s = s
                            .font_size(12.0)
                            .padding_horiz(12.0)
                            .padding_vert(8.0);

                        if is_success {
                            s = s.color(Color::rgb8(46, 125, 50));
                        } else if is_error {
                            s = s.color(Color::rgb8(211, 47, 47));
                        } else {
                            s = s.color(Color::rgb8(100, 100, 100));
                        }
                        s
                    })
                    .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(|s| {
        s.width_full()
            .min_height(32.0)
            .background(Color::rgb8(248, 248, 248))
            .border(1.0)
            .border_color(Color::rgb8(230, 230, 230))
            .border_radius(4.0)
    })
}
