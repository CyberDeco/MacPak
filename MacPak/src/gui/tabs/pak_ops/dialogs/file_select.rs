//! File select dialog content

use floem::prelude::*;
use floem::text::Weight;
use floem::views::{text_input, virtual_list, VirtualDirection, VirtualItemSize};
use im::Vector as ImVector;
use std::path::Path;

use crate::gui::state::{ActiveDialog, PakOpsState};
use super::super::operations::execute_individual_extract;

pub fn file_select_content(state: PakOpsState) -> impl IntoView {
    let files = state.file_select_list;
    let selected = state.file_select_selected;
    let pak_path = state.file_select_pak;
    let ext_filter = RwSignal::new(String::new());

    let state_extract = state.clone();
    let state_cancel = state.clone();
    let state_select_all = state.clone();
    let state_deselect = state.clone();

    let filtered_files = move || {
        let filter = ext_filter.get().to_lowercase();
        let all_files = files.get();
        if filter.is_empty() {
            all_files.into_iter().collect::<ImVector<_>>()
        } else {
            all_files
                .into_iter()
                .filter(|f| f.to_lowercase().contains(&filter))
                .collect::<ImVector<_>>()
        }
    };

    let pak_name = pak_path.get()
        .map(|p| Path::new(&p).file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "PAK".to_string()))
        .unwrap_or_else(|| "PAK".to_string());

    v_stack((
        label(move || format!("Select files from {}", pak_name)).style(|s| {
            s.font_size(16.0)
                .font_weight(Weight::BOLD)
                .margin_bottom(8.0)
        }),
        h_stack((
            label(move || {
                let sel_count = selected.get().len();
                let filtered = filtered_files().len();
                let total = files.get().len();
                if filtered == total {
                    format!("{} of {} files selected", sel_count, total)
                } else {
                    format!("{} selected, {} of {} shown", sel_count, filtered, total)
                }
            })
            .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
            empty().style(|s| s.flex_grow(1.0)),
            text_input(ext_filter)
                .placeholder("Filter (e.g. .lsf, .xml)")
                .style(|s| {
                    s.width(160.0)
                        .height(26.0)
                        .padding_horiz(8.0)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                        .font_size(12.0)
                        .background(Color::WHITE)
                }),
        ))
        .style(|s| s.width_full().margin_bottom(12.0).items_center()),
        h_stack((
            button("Select All Visible")
                .action(move || {
                    let visible_files: std::collections::HashSet<String> =
                        filtered_files().into_iter().collect();
                    state_select_all.file_select_selected.update(|set| {
                        for f in visible_files {
                            set.insert(f);
                        }
                    });
                })
                .style(|s| {
                    s.padding_vert(6.0)
                        .padding_horiz(12.0)
                        .font_size(12.0)
                        .background(Color::rgb8(240, 240, 240))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                }),
            button("Deselect All")
                .action(move || {
                    state_deselect.file_select_selected.set(std::collections::HashSet::new());
                })
                .style(|s| {
                    s.padding_vert(6.0)
                        .padding_horiz(12.0)
                        .font_size(12.0)
                        .margin_left(8.0)
                        .background(Color::rgb8(240, 240, 240))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                }),
        ))
        .style(|s| s.margin_bottom(12.0)),
        scroll(
            virtual_list(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| 28.0)),
                filtered_files,
                |file_path: &String| file_path.clone(),
                move |file_path| {
                    let file_path_clone = file_path.clone();
                    let file_path_for_check = file_path.clone();
                    let file_path_for_update = file_path.clone();
                    let file_path_for_label = file_path.clone();

                    h_stack((
                        checkbox(move || selected.get().contains(&file_path_for_check))
                            .on_update(move |checked| {
                                let fp = file_path_for_update.clone();
                                selected.update(|set| {
                                    if checked {
                                        set.insert(fp);
                                    } else {
                                        set.remove(&fp);
                                    }
                                });
                            })
                            .style(|s| s.margin_right(8.0)),
                        label(move || file_path_clone.clone())
                            .on_click_stop(move |_| {
                                let fp = file_path_for_label.clone();
                                selected.update(|set| {
                                    if set.contains(&fp) {
                                        set.remove(&fp);
                                    } else {
                                        set.insert(fp);
                                    }
                                });
                            })
                            .style(|s| {
                                s.font_size(12.0)
                                    .text_overflow(floem::style::TextOverflow::Clip)
                                    .cursor(floem::style::CursorStyle::Pointer)
                            }),
                    ))
                    .style(|s| {
                        s.height(28.0)
                            .padding_vert(4.0)
                            .padding_horiz(8.0)
                            .items_center()
                            .flex_shrink(0.0)
                            .hover(|s| s.background(Color::rgb8(245, 245, 245)))
                    })
                },
            )
            .style(|s| s.flex_col())
        )
        .scroll_style(|s| s.handle_thickness(8.0))
        .style(|s| {
            s.width_full()
                .height(300.0)
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
                .background(Color::WHITE)
        }),
        h_stack((
            empty().style(|s| s.flex_grow(1.0)),
            button("Cancel")
                .action(move || {
                    state_cancel.file_select_pak.set(None);
                    state_cancel.file_select_list.set(Vec::new());
                    state_cancel.file_select_selected.set(std::collections::HashSet::new());
                    state_cancel.clear_results();
                    state_cancel.active_dialog.set(ActiveDialog::None);
                })
                .style(|s| {
                    s.padding_vert(8.0)
                        .padding_horiz(20.0)
                        .margin_right(8.0)
                        .background(Color::rgb8(240, 240, 240))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                }),
            button("Extract Selected")
                .action(move || {
                    execute_individual_extract(state_extract.clone());
                })
                .disabled(move || selected.get().is_empty())
                .style(move |s| {
                    let disabled = selected.get().is_empty();
                    let s = s
                        .padding_vert(8.0)
                        .padding_horiz(20.0)
                        .border_radius(4.0);
                    if disabled {
                        s.background(Color::rgb8(200, 200, 200))
                            .color(Color::rgb8(150, 150, 150))
                    } else {
                        s.background(Color::rgb8(33, 150, 243))
                            .color(Color::WHITE)
                            .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                    }
                }),
        ))
        .style(|s| s.width_full().margin_top(16.0)),
    ))
    .style(|s| {
        s.padding(24.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(200, 200, 200))
            .border_radius(8.0)
            .width(800.0)
            .max_height(500.0)
    })
}
