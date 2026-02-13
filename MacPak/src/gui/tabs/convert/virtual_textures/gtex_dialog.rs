//! GTex hash extraction dialog overlay

use floem::prelude::*;
use floem::text::Weight;
use floem::views::{PlaceholderTextClass, text_input};

use super::extraction::extract_by_gtex_hash;
use crate::gui::shared::checkbox_option;
use crate::gui::state::{ConfigState, VirtualTexturesState};

/// Dialog overlay for extracting textures by GTex hash
pub fn gtex_dialog_overlay(state: VirtualTexturesState, config: ConfigState) -> impl IntoView {
    let show = state.show_gtex_dialog;
    let hash_input = state.gtex_hash_input;
    let search_paths = state.gtex_search_paths;
    let convert_to_png = state.convert_to_png;
    let game_data_path = config.bg3_data_path;

    let state_extract = state.clone();
    let config_extract = config.clone();
    let state_cancel = state.clone();

    dyn_container(
        move || show.get(),
        move |visible| {
            if !visible {
                return empty().into_any();
            }

            let state_extract = state_extract.clone();
            let config_extract = config_extract.clone();
            let state_cancel = state_cancel.clone();

            container(
                v_stack((
                    // Title
                    label(|| "Extract by GTex Hash").style(|s| {
                        s.font_size(16.0)
                            .font_weight(Weight::BOLD)
                            .margin_bottom(16.0)
                    }),
                    // Hash input
                    label(|| "GTex Hash(es)").style(|s| {
                        s.font_size(12.0)
                            .color(Color::rgb8(80, 80, 80))
                            .margin_bottom(4.0)
                    }),
                    text_input(hash_input)
                        .placeholder("Enter hash (comma or newline separated)")
                        .style(|s| {
                            s.width_full()
                                .height(32.0)
                                .padding_horiz(8.0)
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0)
                                .font_size(13.0)
                                .background(Color::WHITE)
                                .margin_bottom(16.0)
                                .class(PlaceholderTextClass, |s| {
                                    s.color(Color::rgb8(120, 120, 120))
                                })
                        }),
                    // Search directories section
                    label(|| "Search Directories").style(|s| {
                        s.font_size(12.0)
                            .font_weight(Weight::SEMIBOLD)
                            .color(Color::rgb8(80, 80, 80))
                            .margin_bottom(4.0)
                    }),
                    // Game data path (auto-included)
                    dyn_container(
                        move || game_data_path.get(),
                        move |path| {
                            if path.is_empty() {
                                label(|| "Game data path not set (configure in Settings)")
                                    .style(|s| {
                                        s.font_size(11.0)
                                            .color(Color::rgb8(180, 80, 30))
                                            .margin_bottom(4.0)
                                    })
                                    .into_any()
                            } else {
                                label(move || format!("Game data: {}", path))
                                    .style(|s| {
                                        s.font_size(11.0)
                                            .color(Color::rgb8(100, 100, 100))
                                            .margin_bottom(4.0)
                                            .max_width_full()
                                            .text_overflow(floem::style::TextOverflow::Ellipsis)
                                    })
                                    .into_any()
                            }
                        },
                    ),
                    // User-added search paths list
                    dyn_stack(
                        move || {
                            search_paths
                                .get()
                                .into_iter()
                                .enumerate()
                                .collect::<Vec<_>>()
                        },
                        |(i, p)| (*i, p.clone()),
                        move |(i, p)| {
                            let p_display = p.clone();
                            h_stack((
                                label(move || p_display.clone()).style(|s| {
                                    s.font_size(11.0)
                                        .color(Color::rgb8(100, 100, 100))
                                        .flex_grow(1.0)
                                        .min_width(0.0)
                                        .text_overflow(floem::style::TextOverflow::Ellipsis)
                                }),
                                button("x")
                                    .action(move || {
                                        search_paths.update(|paths| {
                                            if i < paths.len() {
                                                paths.remove(i);
                                            }
                                        });
                                    })
                                    .style(|s| {
                                        s.padding_vert(2.0)
                                            .padding_horiz(6.0)
                                            .font_size(10.0)
                                            .background(Color::rgb8(240, 240, 240))
                                            .border(1.0)
                                            .border_color(Color::rgb8(200, 200, 200))
                                            .border_radius(3.0)
                                            .cursor(floem::style::CursorStyle::Pointer)
                                    }),
                            ))
                            .style(|s| s.items_center().gap(4.0).margin_bottom(2.0))
                        },
                    )
                    .style(|s| s.flex_col()),
                    // Add search directory button
                    button("Add Search Directory")
                        .action(move || {
                            if let Some(dir) = rfd::FileDialog::new()
                                .set_title("Add Search Directory")
                                .pick_folder()
                            {
                                search_paths.update(|paths| {
                                    paths.push(dir.to_string_lossy().to_string());
                                });
                            }
                        })
                        .style(|s| {
                            s.padding_vert(6.0)
                                .padding_horiz(12.0)
                                .font_size(12.0)
                                .background(Color::rgb8(240, 240, 240))
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0)
                                .margin_top(4.0)
                                .margin_bottom(16.0)
                        }),
                    // Convert to PNG checkbox
                    checkbox_option("Convert to PNG", convert_to_png),
                    // Action buttons
                    h_stack((
                        empty().style(|s| s.flex_grow(1.0)),
                        button("Cancel")
                            .action(move || {
                                state_cancel.show_gtex_dialog.set(false);
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
                                extract_by_gtex_hash(
                                    state_extract.clone(),
                                    config_extract.clone(),
                                );
                            })
                            .disabled(move || hash_input.get().trim().is_empty())
                            .style(move |s| {
                                let disabled = hash_input.get().trim().is_empty();
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
                        .width(600.0)
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
    .on_event_stop(floem::event::EventListener::KeyDown, move |e| {
        if let floem::event::Event::KeyDown(key_event) = e {
            if key_event.key.logical_key
                == floem::keyboard::Key::Named(floem::keyboard::NamedKey::Escape)
            {
                show.set(false);
            }
        }
    })
    .keyboard_navigable()
}
