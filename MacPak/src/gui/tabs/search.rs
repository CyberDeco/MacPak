//! Index Search Tab
//!
//! Search across indexed PAK files for assets by name, type, or content.

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::{AppState, SearchState, SearchResult};

pub fn search_tab(_app_state: AppState, search_state: SearchState) -> impl IntoView {
    v_stack((
        search_toolbar(search_state.clone()),
        search_results(search_state.clone()),
        search_status_bar(search_state),
    ))
    .style(|s| s.width_full().height_full())
}

fn search_toolbar(state: SearchState) -> impl IntoView {
    let query = state.query;
    let is_searching = state.is_searching;

    h_stack((
        // Search input
        text_input(query)
            .placeholder("Search for files, UUIDs, or content...")
            .style(|s| s.flex_grow(1.0).max_width(500.0)),

        // Search button (dynamic text based on searching state)
        dyn_container(
            move || is_searching.get(),
            move |searching| {
                if searching {
                    button("⏳ Searching...")
                        .disabled(|| true)
                        .style(|s| {
                            s.padding_horiz(16.0)
                                .background(Color::rgb8(150, 150, 150))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                        })
                        .into_any()
                } else {
                    let query = query.clone();
                    button("🔍 Search")
                        .style(|s| {
                            s.padding_horiz(16.0)
                                .background(Color::rgb8(33, 150, 243))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                        })
                        .action(move || {
                            let q = query.get();
                            if !q.is_empty() {
                                println!("Search for: {} - TODO: call MacPak index search", q);
                            }
                        })
                        .into_any()
                }
            },
        ),

        separator(),

        // Filter options
        label(|| "Filter:").style(|s| s.margin_right(8.0)),
        button("All"),
        button("LSX"),
        button("LSJ"),
        button("LSF"),
        button("Images"),
        button("Audio"),

        empty().style(|s| s.flex_grow(1.0)),

        // Index status
        button("🔄 Rebuild Index").action(|| {
            println!("Rebuild index - TODO");
        }),
    ))
    .style(|s| {
        s.width_full()
            .height(50.0)
            .padding(10.0)
            .gap(8.0)
            .items_center()
            .background(Color::rgb8(245, 245, 245))
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn search_results(state: SearchState) -> impl IntoView {
    let results = state.results;
    let is_searching = state.is_searching;

    v_stack((
        // Results header
        h_stack((
            label(|| "Results").style(|s| s.font_weight(Weight::BOLD)),
            empty().style(|s| s.flex_grow(1.0)),
            label(move || format!("{} matches", results.get().len()))
                .style(|s| s.color(Color::rgb8(128, 128, 128)).font_size(12.0)),
        ))
        .style(|s| {
            s.width_full()
                .padding(8.0)
                .background(Color::rgb8(250, 250, 250))
                .border_bottom(1.0)
                .border_color(Color::rgb8(220, 220, 220))
        }),

        // Results list
        scroll(
            dyn_container(
                move || (results.get().is_empty(), is_searching.get()),
                move |(is_empty, searching)| {
                    if searching {
                        label(|| "Searching...")
                            .style(|s| {
                                s.width_full()
                                    .padding(40.0)
                                    .items_center()
                                    .color(Color::rgb8(128, 128, 128))
                            })
                            .into_any()
                    } else if is_empty {
                        label(|| "No results. Enter a search query above.")
                            .style(|s| {
                                s.width_full()
                                    .padding(40.0)
                                    .items_center()
                                    .color(Color::rgb8(160, 160, 160))
                            })
                            .into_any()
                    } else {
                        dyn_stack(
                            move || results.get(),
                            |result| result.path.clone(),
                            |result| search_result_row(result),
                        )
                        .style(|s| s.width_full())
                        .into_any()
                    }
                },
            )
            .style(|s| s.width_full()),
        )
        .style(|s| s.width_full().flex_grow(1.0)),
    ))
    .style(|s| s.width_full().flex_grow(1.0).background(Color::WHITE))
}

fn search_result_row(result: SearchResult) -> impl IntoView {
    let icon = get_type_icon(&result.file_type);

    h_stack((
        // Icon
        label(move || icon.to_string()).style(|s| s.width(30.0)),

        // File info
        v_stack((
            label(move || result.name.clone()).style(|s| s.font_weight(Weight::MEDIUM)),
            label(move || result.path.clone())
                .style(|s| s.font_size(12.0).color(Color::rgb8(128, 128, 128))),
        ))
        .style(|s| s.flex_grow(1.0)),

        // PAK file
        label(move || result.pak_file.clone())
            .style(|s| {
                s.font_size(12.0)
                    .color(Color::rgb8(100, 100, 100))
                    .padding_horiz(8.0)
                    .padding_vert(4.0)
                    .background(Color::rgb8(240, 240, 240))
                    .border_radius(4.0)
            }),

        // Actions
        button("📂").action(|| println!("Open in browser - TODO")),
        button("📋").action(|| println!("Copy path - TODO")),
    ))
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .gap(8.0)
            .items_center()
            .border_bottom(1.0)
            .border_color(Color::rgb8(240, 240, 240))
            .hover(|s| s.background(Color::rgb8(250, 252, 255)))
    })
}

fn search_status_bar(_state: SearchState) -> impl IntoView {
    h_stack((
        label(|| "Index: Ready")
            .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0)),

        empty().style(|s| s.flex_grow(1.0)),

        label(|| "Last updated: Never")
            .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0)),
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

fn separator() -> impl IntoView {
    empty().style(|s| {
        s.width(1.0)
            .height(30.0)
            .background(Color::rgb8(200, 200, 200))
            .margin_horiz(4.0)
    })
}

fn get_type_icon(file_type: &str) -> &'static str {
    match file_type.to_lowercase().as_str() {
        "lsx" | "xml" => "📄",
        "lsj" | "json" => "📋",
        "lsf" => "🔷",
        "dds" | "png" | "jpg" => "🖼️",
        "gr2" => "🦴",
        "wem" | "ogg" | "wav" => "🔊",
        _ => "📄",
    }
}
