//! Search toolbar components

use std::path::PathBuf;

use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem::views::PlaceholderTextClass;
use maclarian::search::FileType;

use crate::gui::state::{ConfigState, IndexStatus, SearchState};

use super::operations::{build_index, extract_selected_results, find_pak_files, perform_search};

pub fn search_toolbar(state: SearchState, config_state: ConfigState) -> impl IntoView {
    let query = state.query;
    let active_filter = state.active_filter;
    let index_status = state.index_status;

    let state_enter = state.clone();

    h_stack((
        // Search input
        text_input(query)
            .placeholder("Search for files, UUIDs, or content...")
            .style(|s| {
                s.flex_grow(1.0)
                    .max_width(400.0)
                    .class(PlaceholderTextClass, |s| {
                        s.color(Color::rgb8(120, 120, 120))
                    })
            })
            .on_key_down(
                Key::Named(NamedKey::Enter),
                |_| true,
                move |_| {
                    perform_search(state_enter.clone());
                },
            ),
        // Search button
        search_button(state.clone()),
        separator(),
        // Filter buttons
        filter_buttons(active_filter),
        separator(),
        // Extract selected button
        extract_selected_button(state.clone()),
        empty().style(|s| s.flex_grow(1.0)),
        // Rebuild index button (uses BG3 path from preferences)
        rebuild_index_button(state.clone(), config_state, index_status),
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

fn search_button(state: SearchState) -> impl IntoView {
    let is_searching = state.is_searching;
    let query = state.query;
    let index_status = state.index_status;

    dyn_container(
        move || (is_searching.get(), index_status.get()),
        move |(searching, status)| {
            let is_ready = matches!(status, IndexStatus::Ready { .. });

            if searching {
                button("Searching...")
                    .disabled(|| true)
                    .style(|s| {
                        s.padding_horiz(16.0)
                            .background(Color::rgb8(150, 150, 150))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                    })
                    .into_any()
            } else {
                let state_clone = state.clone();
                button("Search")
                    .disabled(move || !is_ready || query.get().is_empty())
                    .style(move |s| {
                        let bg = if is_ready {
                            Color::rgb8(33, 150, 243)
                        } else {
                            Color::rgb8(150, 150, 150)
                        };
                        s.padding_horiz(16.0)
                            .background(bg)
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                    })
                    .action(move || {
                        perform_search(state_clone.clone());
                    })
                    .into_any()
            }
        },
    )
}

fn filter_buttons(active_filter: RwSignal<Option<FileType>>) -> impl IntoView {
    h_stack((
        filter_button("All", None, active_filter),
        filter_button("LSX", Some(FileType::Lsx), active_filter),
        filter_button("LSF", Some(FileType::Lsf), active_filter),
        filter_button("LSJ", Some(FileType::Lsj), active_filter),
        filter_button("GR2", Some(FileType::Gr2), active_filter),
        filter_button("Images", Some(FileType::Dds), active_filter),
        filter_button("Audio", Some(FileType::Wem), active_filter),
    ))
    .style(|s| s.gap(4.0))
}

fn filter_button(
    label_text: &'static str,
    filter_type: Option<FileType>,
    active_filter: RwSignal<Option<FileType>>,
) -> impl IntoView {
    dyn_container(
        move || active_filter.get() == filter_type,
        move |is_active| {
            let bg = if is_active {
                Color::rgb8(33, 150, 243)
            } else {
                Color::rgb8(230, 230, 230)
            };
            let fg = if is_active {
                Color::WHITE
            } else {
                Color::rgb8(60, 60, 60)
            };

            button(label_text)
                .style(move |s| {
                    s.padding_horiz(10.0)
                        .padding_vert(4.0)
                        .font_size(12.0)
                        .background(bg)
                        .color(fg)
                        .border_radius(4.0)
                })
                .action(move || {
                    active_filter.set(filter_type);
                })
                .into_any()
        },
    )
}

fn rebuild_index_button(
    state: SearchState,
    config_state: ConfigState,
    index_status: RwSignal<IndexStatus>,
) -> impl IntoView {
    let bg3_path = config_state.bg3_data_path;

    dyn_container(
        move || (index_status.get(), bg3_path.get()),
        move |(status, path)| {
            let is_building = matches!(status, IndexStatus::Building { .. });
            let is_ready = matches!(status, IndexStatus::Ready { .. });
            let state_clone = state.clone();
            let path_valid = !path.is_empty() && std::path::Path::new(&path).is_dir();

            if is_building {
                button("Building...")
                    .disabled(|| true)
                    .style(|s| {
                        s.padding_horiz(12.0)
                            .background(Color::rgb8(150, 150, 150))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                    })
                    .into_any()
            } else {
                let path_for_action = path.clone();
                // Change label based on whether index already exists
                let button_label = if is_ready {
                    "Rebuild Index"
                } else {
                    "Build Index"
                };

                button(button_label)
                    .disabled(move || !path_valid)
                    .style(move |s| {
                        let bg = if path_valid {
                            Color::rgb8(76, 175, 80)
                        } else {
                            Color::rgb8(150, 150, 150)
                        };
                        s.padding_horiz(12.0)
                            .background(bg)
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .hover(|s| s.background(Color::rgb8(56, 142, 60)))
                    })
                    .action(move || {
                        // Show confirmation dialog if index already exists
                        let should_build = if is_ready {
                            rfd::MessageDialog::new()
                                .set_title("Rebuild Index?")
                                .set_description(
                                    "An index already exists. Rebuilding will take some time.\n\n\
                                    Do you want to rebuild the index?",
                                )
                                .set_buttons(rfd::MessageButtons::YesNo)
                                .show()
                                == rfd::MessageDialogResult::Yes
                        } else {
                            true
                        };

                        if should_build {
                            // Find PAK files in the configured BG3 data path
                            let paks = find_pak_files(&PathBuf::from(&path_for_action));
                            if !paks.is_empty() {
                                state_clone.pak_paths.set(paks);
                                build_index(state_clone.clone());
                            }
                        }
                    })
                    .into_any()
            }
        },
    )
}

pub fn separator() -> impl IntoView {
    empty().style(|s| {
        s.width(1.0)
            .height(30.0)
            .background(Color::rgb8(200, 200, 200))
            .margin_horiz(4.0)
    })
}

fn extract_selected_button(state: SearchState) -> impl IntoView {
    let selected = state.selected_results;
    let state_for_action = state;

    dyn_container(
        move || selected.get().len(),
        move |count| {
            let has_selection = count > 0;
            let state_clone = state_for_action.clone();

            button(format!("Extract Selected ({})", count))
                .disabled(move || !has_selection)
                .style(move |s| {
                    let bg = if has_selection {
                        Color::rgb8(33, 150, 243)
                    } else {
                        Color::rgb8(180, 180, 180)
                    };
                    s.padding_horiz(10.0)
                        .padding_vert(4.0)
                        .font_size(12.0)
                        .background(bg)
                        .color(Color::WHITE)
                        .border_radius(4.0)
                        .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                })
                .action(move || {
                    extract_selected_results(state_clone.clone());
                })
                .into_any()
        },
    )
}
