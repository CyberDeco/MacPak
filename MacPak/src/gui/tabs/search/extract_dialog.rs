//! Extraction options dialog for Search tab
//!
//! Shows GR2 bundle options when extracting files that include GR2 models.

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::{ConfigState, SearchState};

use super::operations::execute_extraction;

/// Extraction dialog with GR2 bundle options
pub fn extract_dialog(state: SearchState, config_state: ConfigState) -> impl IntoView {
    let show = state.show_extract_dialog;
    let pending_files = state.pending_extract_files;

    // GR2 options
    let extract_gr2 = state.gr2_extract_gr2;
    let convert_to_glb = state.gr2_convert_to_glb;
    let convert_to_gltf = state.gr2_convert_to_gltf;
    let extract_textures = state.gr2_extract_textures;
    let convert_to_png = state.gr2_convert_to_png;
    let game_data_path = config_state.bg3_data_path;

    let state_extract = state.clone();
    let state_cancel = state.clone();

    // Check if selection contains GR2 files
    let has_gr2 = move || {
        pending_files.get().iter().any(|(path, _)| path.to_lowercase().ends_with(".gr2"))
    };

    // Check if all options are enabled (bundle mode)
    let is_bundle = move || {
        extract_gr2.get() && convert_to_glb.get() && extract_textures.get() && convert_to_png.get()
    };

    // Show warning when texture extraction enabled but path not configured
    let needs_warning = move || {
        extract_textures.get() && game_data_path.get().is_empty()
    };

    dyn_container(
        move || show.get(),
        move |visible| {
            if !visible {
                return empty().into_any();
            }

            // Clone for use in nested closures
            let state_extract = state_extract.clone();
            let state_cancel = state_cancel.clone();
            let config_for_extract = config_state.clone();

            let file_count = pending_files.get().len();
            let gr2_count = pending_files.get().iter()
                .filter(|(p, _)| p.to_lowercase().ends_with(".gr2"))
                .count();

            container(
                v_stack((
                    // Header
                    label(|| "Extract Files")
                        .style(|s| {
                            s.font_size(16.0)
                                .font_weight(Weight::BOLD)
                                .margin_bottom(8.0)
                        }),

                    // File count summary
                    label(move || {
                        if gr2_count > 0 {
                            format!("{} files ({} GR2 models)", file_count, gr2_count)
                        } else {
                            format!("{} files", file_count)
                        }
                    })
                    .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100)).margin_bottom(16.0)),

                    // GR2 options panel (only shown when GR2 files are in selection)
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
                                    extract_gr2.set(checked);
                                    convert_to_glb.set(checked);
                                    convert_to_gltf.set(false); // GLB takes precedence
                                    extract_textures.set(checked);
                                    convert_to_png.set(checked);
                                })
                                .style(|s| s.margin_right(8.0)),
                            label(|| "Full Bundle")
                                .style(|s| s.font_size(12.0).font_weight(Weight::MEDIUM)),
                            label(|| " (all options)")
                                .style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100))),
                        ))
                        .style(|s| s.items_center().margin_bottom(10.0)),

                        // Extract GR2
                        h_stack((
                            checkbox(move || extract_gr2.get())
                                .on_update(move |checked| extract_gr2.set(checked))
                                .style(|s| s.margin_right(6.0)),
                            label(|| "Extract GR2")
                                .style(|s| s.font_size(12.0)),
                        ))
                        .style(|s| s.items_center().margin_left(20.0).margin_bottom(4.0)),

                        // Convert to GLB
                        h_stack((
                            checkbox(move || convert_to_glb.get())
                                .on_update(move |checked| {
                                    convert_to_glb.set(checked);
                                    if checked {
                                        convert_to_gltf.set(false);
                                    }
                                })
                                .style(|s| s.margin_right(6.0)),
                            label(|| "Convert to GLB")
                                .style(move |s| {
                                    let disabled = convert_to_gltf.get();
                                    s.font_size(12.0)
                                        .color(if disabled { Color::rgb8(160, 160, 160) } else { Color::BLACK })
                                }),
                        ))
                        .style(|s| s.items_center().margin_left(20.0).margin_bottom(4.0)),

                        // Convert to glTF
                        h_stack((
                            checkbox(move || convert_to_gltf.get())
                                .on_update(move |checked| {
                                    convert_to_gltf.set(checked);
                                    if checked {
                                        convert_to_glb.set(false);
                                    }
                                })
                                .style(|s| s.margin_right(6.0)),
                            label(|| "Convert to glTF")
                                .style(move |s| {
                                    let disabled = convert_to_glb.get();
                                    s.font_size(12.0)
                                        .color(if disabled { Color::rgb8(160, 160, 160) } else { Color::BLACK })
                                }),
                        ))
                        .style(|s| s.items_center().margin_left(20.0).margin_bottom(4.0)),

                        // Extract textures DDS
                        h_stack((
                            checkbox(move || extract_textures.get())
                                .on_update(move |checked| extract_textures.set(checked))
                                .style(|s| s.margin_right(6.0)),
                            label(|| "Extract textures DDS")
                                .style(|s| s.font_size(12.0)),
                        ))
                        .style(|s| s.items_center().margin_left(20.0).margin_bottom(4.0)),

                        // Convert textures DDS to PNG
                        h_stack((
                            checkbox(move || convert_to_png.get())
                                .on_update(move |checked| convert_to_png.set(checked))
                                .style(|s| s.margin_right(6.0)),
                            label(|| "Convert textures DDS to PNG")
                                .style(|s| s.font_size(12.0)),
                        ))
                        .style(|s| s.items_center().margin_left(20.0)),

                        // Warning when textures enabled but path not configured
                        dyn_container(
                            needs_warning,
                            move |show_warning| {
                                if show_warning {
                                    label(|| "Warning: BG3 game data path not set in Settings")
                                        .style(|s| {
                                            s.font_size(11.0)
                                                .color(Color::rgb8(180, 80, 30))
                                                .margin_top(8.0)
                                                .margin_left(20.0)
                                        })
                                        .into_any()
                                } else {
                                    empty().into_any()
                                }
                            },
                        ),
                    ))
                    .style(move |s| {
                        let visible = has_gr2();
                        let s = s
                            .width_full()
                            .margin_bottom(16.0)
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
                    }),

                    // Action buttons
                    h_stack((
                        empty().style(|s| s.flex_grow(1.0)),
                        button("Cancel")
                            .action(move || {
                                state_cancel.show_extract_dialog.set(false);
                                state_cancel.pending_extract_files.set(Vec::new());
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
                        button("Extract")
                            .action(move || {
                                execute_extraction(state_extract.clone(), config_for_extract.clone());
                            })
                            .style(|s| {
                                s.padding_vert(8.0)
                                    .padding_horiz(20.0)
                                    .background(Color::rgb8(33, 150, 243))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                                    .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                            }),
                    ))
                    .style(|s| s.width_full()),
                ))
                .style(|s| {
                    s.padding(24.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .width(450.0)
                        .box_shadow_blur(20.0)
                        .box_shadow_color(Color::rgba8(0, 0, 0, 50))
                }),
            )
            .into_any()
        },
    )
    .style(move |s| {
        if show.get() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
}
