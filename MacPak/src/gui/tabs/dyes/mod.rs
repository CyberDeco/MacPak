//! Dyes Tab - Custom dye color creator for BG3 modding

mod color_row;
mod export;
mod generate;
mod import;
mod sections;
pub mod shared;

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::{AppState, DyesState, VENDOR_DEFS};
use crate::gui::utils::meta_dialog::{meta_dialog_with_signals_and_extra, MetaDialogSignals};
use export::export_section;
use generate::generate_dye_section;
use import::import_section;
use sections::{common_section, recommended_section, required_section};
use shared::constants::*;

pub fn dyes_tab(_app_state: AppState, state: DyesState) -> impl IntoView {
    let status = state.status_message;
    let show_meta = state.show_meta_dialog;
    let state_for_export = state.clone();
    let selected_vendors = state.selected_vendors;

    // Create signals struct for meta dialog
    let meta_signals = MetaDialogSignals {
        mod_name: state.mod_name,
        author: state.mod_author,
        description: state.mod_description,
        uuid: state.mod_uuid,
        version_major: state.mod_version_major,
        version_minor: state.mod_version_minor,
        version_patch: state.mod_version_patch,
        version_build: state.mod_version_build,
    };

    // Callback for meta dialog - export the dye mod
    let on_meta_create = move |_content: String| {
        // The meta.lsx content is generated, but we use export_dye_mod which generates all files
        let name = state_for_export.mod_name.get();
        if name.is_empty() {
            state_for_export.status_message.set("Error: Mod name is required".to_string());
            return;
        }

        // Open folder picker and export
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select output folder for mod")
            .pick_folder()
        {
            let message = export::export_dye_mod(&state_for_export, &path, &name);
            state_for_export.status_message.set(message);
        }
    };

    // Vendor selection extra content for export dialog
    let vendor_selection_content = move || {
        vendor_selection_section(selected_vendors)
    };

    v_stack((
        // Header - matches PAK Ops style
        header_section(status),

        // Color sections (scrollable to handle overflow)
        scroll(
            h_stack((
                required_section(state.clone(), status),
                common_section(state.clone(), status),
                // Recommended + Generate Dye stacked in same column
                v_stack((
                    recommended_section(state.clone(), status),
                    generate_dye_section(state.clone()),
                ))
                .style(|s| s.flex_grow(1.0).flex_basis(0.0).gap(GAP_LG)),
            ))
            .style(|s| {
                s.width_full()
                    .items_start()
                    .padding(20.0)
                    .gap(GAP_LG)
            }),
        )
        .style(|s| s.width_full()),
        // Import and Export sections side by side
        h_stack((
            import_section(state.clone()),
            export_section(state.clone()),
        ))
        .style(|s| {
            s.width_full()
                .padding_horiz(24.0)
                .padding_bottom(24.0)
                .gap(GAP_LG)
                .items_start()
        }),
        // Meta.lsx / Export dialog overlay with vendor selection
        meta_dialog_with_signals_and_extra(
            show_meta,
            meta_signals,
            on_meta_create,
            Some(status),
            "Export Dye Mod",
            "Export",
            vendor_selection_content,
        ),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(BG_CARD)
            .position(floem::style::Position::Relative)
    })
}

/// Indices for vendors by act (based on VENDOR_DEFS order)
// const ACT1_VENDOR_INDICES: &[usize] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
// const ACT2_VENDOR_INDICES: &[usize] = &[16, 17, 18, 19, 20, 21, 22, 23];
// const ACT3_VENDOR_INDICES: &[usize] = &[24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44];

/// Slimmed-down version
const ACT1_VENDOR_INDICES: &[usize] = &[0, 1, 2, 6, 7, 10, 14, 15, 16];
const ACT2_VENDOR_INDICES: &[usize] = &[17, 18, 19, 20, 21];
const ACT3_VENDOR_INDICES: &[usize] = &[24, 25, 26, 29, 33, 34, 37, 38, 40];

/// Vendor selection section for export dialog
fn vendor_selection_section(selected_vendors: RwSignal<Vec<bool>>) -> impl IntoView {
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
                    let vendors: Vec<bool> = VENDOR_DEFS.iter()
                        .map(|v| v.always_enabled)
                        .collect();
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
        label(move || title)
            .style(|s| {
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
                            label(move || name.to_string())
                                .style(move |s| {
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
            .style(|s| s.width_full().flex_direction(floem::style::FlexDirection::Column))
        )
        .style(|s| {
            s.flex_grow(1.0)
                .max_height(220.0)
        }),
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

fn header_section(status: RwSignal<String>) -> impl IntoView {
    h_stack((
        label(|| "Dye Lab")
            .style(|s| s.font_size(FONT_TITLE).font_weight(Weight::BOLD)),
        empty().style(|s| s.flex_grow(1.0)),
        // Status message
        dyn_container(
            move || status.get(),
            move |msg| {
                if msg.is_empty() {
                    empty().into_any()
                } else {
                    label(move || msg.clone())
                        .style(|s| {
                            s.padding_horiz(PADDING_BTN_H)
                                .padding_vert(PADDING_BTN_V)
                                .border_radius(RADIUS_STD)
                                .font_size(FONT_STATUS)
                                .background(BG_SUCCESS)
                                .color(TEXT_SUCCESS)
                        })
                        .into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(PADDING_LG)
            .gap(GAP_STD)
            .items_center()
            .background(Color::WHITE)
            .border_bottom(1.0)
            .border_color(BORDER_CARD)
    })
}
