//! Dyes Tab - Custom dye color creator for BG3 modding

mod color_row;
mod export;
mod generate;
mod import;
mod sections;
mod shared;

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::{AppState, DyesState};
use crate::gui::utils::meta_dialog::meta_dialog;
use export::export_section;
use generate::generate_dye_section;
use import::import_section;
use sections::{common_section, recommended_section, required_section};
use shared::copy_to_clipboard;
use shared::constants::*;

pub fn dyes_tab(_app_state: AppState, state: DyesState) -> impl IntoView {
    let status = state.status_message;
    let show_meta = state.show_meta_dialog;
    let mod_name = state.mod_name;

    // Callback for meta dialog - copy to clipboard
    let status_for_callback = status;
    let on_meta_create = move |content: String| {
        copy_to_clipboard(&content);
        status_for_callback.set("Copied meta.lsx to clipboard".to_string());
    };

    v_stack((
        // Header - matches PAK Ops style
        header_section(status),

        // Color sections
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
        // Import and Export sections side by side
        h_stack((
            import_section(state.clone()),
            export_section(state),
        ))
        .style(|s| {
            s.width_full()
                .padding_horiz(24.0)
                .padding_bottom(24.0)
                .gap(GAP_LG)
                .items_start()
        }),
        // Meta.lsx dialog overlay
        meta_dialog(show_meta, Some(mod_name.get()), on_meta_create, Some(status)),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(BG_CARD)
            .position(floem::style::Position::Relative)
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
