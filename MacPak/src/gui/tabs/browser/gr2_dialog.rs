//! GR2 conversion dialog for Browser tab
//!
//! Shows conversion options when right-clicking a GR2 file and selecting "Convert to..."

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::{BrowserState, ConfigState};

use super::operations::convert_gr2_file;

/// GR2 conversion dialog with bundle options
pub fn gr2_conversion_dialog(state: BrowserState, config_state: ConfigState) -> impl IntoView {
    let show = state.show_gr2_dialog;
    let gr2_path = state.gr2_convert_path;

    // GR2 options
    let extract_gr2 = state.gr2_extract_gr2;
    let convert_to_glb = state.gr2_convert_to_glb;
    let convert_to_gltf = state.gr2_convert_to_gltf;
    let extract_textures = state.gr2_extract_textures;
    let convert_to_png = state.gr2_convert_to_png;
    let game_data_path = config_state.bg3_data_path;

    let state_convert = state.clone();
    let state_cancel = state.clone();

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
            let state_convert = state_convert.clone();
            let state_cancel = state_cancel.clone();
            let config_for_convert = config_state.clone();

            let file_name = gr2_path.get()
                .map(|p| std::path::Path::new(&p)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "GR2 file".to_string()))
                .unwrap_or_else(|| "GR2 file".to_string());

            container(
                v_stack((
                    // Header
                    label(|| "Convert GR2")
                        .style(|s| {
                            s.font_size(16.0)
                                .font_weight(Weight::BOLD)
                                .margin_bottom(8.0)
                        }),

                    // File name
                    label(move || file_name.clone())
                        .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100)).margin_bottom(16.0)),

                    // GR2 options panel
                    v_stack((
                        label(|| "Conversion Options").style(|s| {
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

                        // Keep original GR2
                        h_stack((
                            checkbox(move || extract_gr2.get())
                                .on_update(move |checked| extract_gr2.set(checked))
                                .style(|s| s.margin_right(6.0)),
                            label(|| "Keep original GR2")
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
                    .style(|s| {
                        s.width_full()
                            .margin_bottom(16.0)
                            .padding(12.0)
                            .background(Color::rgb8(248, 248, 248))
                            .border(1.0)
                            .border_color(Color::rgb8(220, 220, 220))
                            .border_radius(4.0)
                    }),

                    // Action buttons
                    h_stack((
                        empty().style(|s| s.flex_grow(1.0)),
                        button("Cancel")
                            .action(move || {
                                state_cancel.show_gr2_dialog.set(false);
                                state_cancel.gr2_convert_path.set(None);
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
                        button("Convert")
                            .action(move || {
                                convert_gr2_file(state_convert.clone(), config_for_convert.clone());
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
