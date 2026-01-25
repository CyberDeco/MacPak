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
    let ext_filter = state.file_select_filter;

    // GR2 options signals
    let gr2_convert = state.gr2_auto_convert;
    let gr2_textures = state.gr2_auto_textures;
    let gr2_virtual_textures = state.gr2_auto_virtual_textures;
    let keep_gr2 = state.keep_original_gr2;

    let state_extract = state.clone();
    let state_cancel = state.clone();
    let state_select_all = state.clone();
    let state_deselect = state.clone();
    let state_bundle = state.clone();

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
        // GR2 Processing Options (shown only when GR2 files are selected)
        gr2_options_panel(
            gr2_convert,
            gr2_textures,
            gr2_virtual_textures,
            keep_gr2,
            state_bundle.clone(),
            selected,
        ),
        h_stack((
            empty().style(|s| s.flex_grow(1.0)),
            button("Cancel")
                .action(move || {
                    state_cancel.file_select_pak.set(None);
                    state_cancel.file_select_list.set(Vec::new());
                    state_cancel.file_select_selected.set(std::collections::HashSet::new());
                    state_cancel.file_select_filter.set(String::new());
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
            .max_height(600.0)
    })
}

/// GR2 Processing Options panel (hidden when no GR2 files are selected)
fn gr2_options_panel(
    gr2_convert: RwSignal<bool>,
    gr2_textures: RwSignal<bool>,
    gr2_virtual_textures: RwSignal<bool>,
    keep_gr2: RwSignal<bool>,
    _state: PakOpsState,
    selected: RwSignal<std::collections::HashSet<String>>,
) -> impl View {
    // Check if all options are enabled (bundle mode)
    let is_bundle = move || {
        gr2_convert.get() && gr2_textures.get() && gr2_virtual_textures.get()
    };

    // Check if selection contains GR2 files
    let has_gr2 = move || {
        selected.get().iter().any(|f| f.to_lowercase().ends_with(".gr2"))
    };

    v_stack((
        label(|| "GR2 Processing Options").style(|s| {
            s.font_size(13.0)
                .font_weight(Weight::BOLD)
                .margin_bottom(8.0)
        }),
        // Full Bundle checkbox
        h_stack((
            checkbox(is_bundle)
                .on_update(move |checked| {
                    gr2_convert.set(checked);
                    gr2_textures.set(checked);
                    gr2_virtual_textures.set(checked);
                })
                .style(|s| s.margin_right(8.0)),
            label(|| "Full Bundle")
                .style(|s| s.font_size(12.0).font_weight(Weight::MEDIUM)),
            label(|| " (convert + all textures)")
                .style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100))),
        ))
        .style(|s| s.items_center().margin_bottom(6.0)),
        // Individual options
        h_stack((
            // Convert to GLB
            h_stack((
                checkbox(move || gr2_convert.get())
                    .on_update(move |checked| gr2_convert.set(checked))
                    .style(|s| s.margin_right(6.0)),
                label(|| "Convert to GLB")
                    .style(|s| s.font_size(11.0)),
            ))
            .style(|s| s.items_center().margin_right(16.0)),
            // Extract textures
            h_stack((
                checkbox(move || gr2_textures.get())
                    .on_update(move |checked| gr2_textures.set(checked))
                    .style(|s| s.margin_right(6.0)),
                label(|| "Extract textures")
                    .style(|s| s.font_size(11.0)),
            ))
            .style(|s| s.items_center().margin_right(16.0)),
            // Extract virtual textures
            h_stack((
                checkbox(move || gr2_virtual_textures.get())
                    .on_update(move |checked| gr2_virtual_textures.set(checked))
                    .style(|s| s.margin_right(6.0)),
                label(|| "Extract virtual textures")
                    .style(|s| s.font_size(11.0)),
            ))
            .style(|s| s.items_center().margin_right(16.0)),
            // Keep original GR2
            h_stack((
                checkbox(move || keep_gr2.get())
                    .on_update(move |checked| keep_gr2.set(checked))
                    .style(|s| s.margin_right(6.0)),
                label(|| "Keep original GR2")
                    .style(|s| s.font_size(11.0)),
            ))
            .style(|s| s.items_center()),
        ))
        .style(|s| s.margin_left(20.0).flex_wrap(floem::style::FlexWrap::Wrap)),
    ))
    .style(move |s| {
        let visible = has_gr2();
        let s = s
            .width_full()
            .margin_top(12.0)
            .padding(12.0)
            .background(Color::rgb8(248, 248, 248))
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(4.0);
        if visible {
            s
        } else {
            s.display(floem::style::Display::None)
        }
    })
}
