//! Vendor selection UI for dye export
//!
//! Provides vendor checkboxes organized by act for selecting
//! where dyes should spawn in the game.

use floem::prelude::*;

use crate::gui::state::VENDOR_DEFS;

/// Vendor indices by act (slimmed-down selection)
const ACT1_VENDOR_INDICES: &[usize] = &[0, 1, 2, 6, 7, 10, 14, 15, 16];
const ACT2_VENDOR_INDICES: &[usize] = &[17, 18, 19, 20, 21];
const ACT3_VENDOR_INDICES: &[usize] = &[24, 25, 26, 29, 33, 34, 37, 38, 40];

/// Vendor selection section for export dialog
pub fn vendor_selection_section(selected_vendors: RwSignal<Vec<bool>>) -> impl IntoView {
    v_stack((
        h_stack((
            label(|| "Spawn Locations")
                .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
            empty().style(|s| s.flex_grow(1.0)),
            // Select All / Deselect All buttons
            label(|| "All")
                .style(|s| {
                    s.font_size(11.0)
                        .color(Color::rgb8(59, 130, 246))
                        .cursor(floem::style::CursorStyle::Pointer)
                        .padding_horiz(4.0)
                })
                .on_click_stop(move |_| {
                    selected_vendors.set(vec![true; VENDOR_DEFS.len()]);
                }),
            label(|| "|").style(|s| s.font_size(11.0).color(Color::rgb8(180, 180, 180))),
            label(|| "None")
                .style(|s| {
                    s.font_size(11.0)
                        .color(Color::rgb8(59, 130, 246))
                        .cursor(floem::style::CursorStyle::Pointer)
                        .padding_horiz(4.0)
                })
                .on_click_stop(move |_| {
                    let vendors: Vec<bool> = VENDOR_DEFS.iter().map(|v| v.always_enabled).collect();
                    selected_vendors.set(vendors);
                }),
        ))
        .style(|s| s.width_full().items_center().margin_top(12.0)),
        // 3-column layout by act
        h_stack((
            vendor_column("Act 1", ACT1_VENDOR_INDICES, selected_vendors),
            vendor_column("Act 2", ACT2_VENDOR_INDICES, selected_vendors),
            vendor_column("Act 3", ACT3_VENDOR_INDICES, selected_vendors),
        ))
        .style(|s| s.width_full().gap(8.0).margin_top(4.0)),
    ))
    .style(|s| s.width_full())
}

/// Single column of vendors for an act
fn vendor_column(
    title: &'static str,
    indices: &'static [usize],
    selected_vendors: RwSignal<Vec<bool>>,
) -> impl IntoView {
    v_stack((
        label(move || title).style(|s| {
            s.font_size(11.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(Color::rgb8(80, 80, 80))
                .padding_bottom(4.0)
        }),
        scroll(
            dyn_stack(
                move || indices.to_vec(),
                |idx| *idx,
                move |idx| {
                    let vendor = &VENDOR_DEFS[idx];
                    let is_always = vendor.always_enabled;
                    let name = vendor.display_name;
                    let loc = vendor.location;

                    h_stack((
                        {
                            let selected_vendors = selected_vendors;
                            checkbox(move || {
                                if is_always {
                                    true
                                } else {
                                    selected_vendors.get().get(idx).copied().unwrap_or(true)
                                }
                            })
                            .disabled(move || is_always)
                            .style(move |s| {
                                if is_always {
                                    s.cursor(floem::style::CursorStyle::Default)
                                } else {
                                    s.cursor(floem::style::CursorStyle::Pointer)
                                }
                            })
                            .on_click_stop(move |_| {
                                if !is_always {
                                    let mut vendors = selected_vendors.get();
                                    if idx < vendors.len() {
                                        vendors[idx] = !vendors[idx];
                                        selected_vendors.set(vendors);
                                    }
                                }
                            })
                        },
                        v_stack((
                            label(move || name.to_string()).style(move |s| {
                                let base = s.font_size(11.0);
                                if is_always {
                                    base.color(Color::rgb8(140, 140, 140))
                                } else {
                                    base
                                }
                            }),
                            label(move || loc.to_string())
                                .style(|s| s.font_size(9.0).color(Color::rgb8(140, 140, 140))),
                        ))
                        .style(|s| s.margin_left(4.0)),
                    ))
                    .style(|s| s.items_center().padding_vert(2.0))
                },
            )
            .style(|s| {
                s.width_full()
                    .flex_direction(floem::style::FlexDirection::Column)
            }),
        )
        .style(|s| s.flex_grow(1.0).max_height(220.0)),
    ))
    .style(|s| {
        s.flex_grow(1.0)
            .flex_basis(0.0)
            .padding(8.0)
            .background(Color::rgb8(250, 250, 250))
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(4.0)
    })
}
