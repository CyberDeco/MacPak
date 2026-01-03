//! Import functionality for the Dyes tab

mod components;
mod operations;

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::DyesState;
use super::shared::secondary_button_style;
use super::shared::constants::*;

use components::{imported_fields_display, txt_import_selector, lsf_import_selector};
use operations::import_from_mod_folder;

/// Import section UI for loading existing dye definitions
pub fn import_section(state: DyesState) -> impl IntoView {
    // Local signals for displaying imported data (independent from export)
    let imported_dye_name: RwSignal<String> = RwSignal::new(String::new());
    let imported_display_name: RwSignal<String> = RwSignal::new(String::new());
    let imported_mod_name: RwSignal<String> = RwSignal::new(String::new());
    let imported_mod_author: RwSignal<String> = RwSignal::new(String::new());

    v_stack((
        // Section header
        h_stack((
            label(|| "Import")
                .style(|s| s.font_size(FONT_HEADER).font_weight(Weight::SEMIBOLD)),
            empty().style(|s| s.flex_grow(1.0)),
            {
                let state = state.clone();
                let imported_dye_name = imported_dye_name;
                let imported_display_name = imported_display_name;
                let imported_mod_name = imported_mod_name;
                let imported_mod_author = imported_mod_author;
                label(|| "Mod Folder...")
                    .style(secondary_button_style)
                    .on_click_stop(move |_| {
                        import_from_mod_folder(state.clone(), imported_dye_name, imported_display_name, imported_mod_name, imported_mod_author);
                    })
            },
        ))
        .style(|s| s.width_full().items_center().gap(GAP_STD).margin_bottom(PADDING_STD)),

        // Inner card with imported data display
        v_stack((
            // TXT Import selector (shows when entries are loaded)
            txt_import_selector(state.clone(), imported_dye_name, imported_display_name, imported_mod_name, imported_mod_author),

            // LSF Import selector (shows when LSF entries are loaded)
            lsf_import_selector(state.clone(), imported_dye_name, imported_display_name, imported_mod_name, imported_mod_author),

            // Display imported values (editable name)
            imported_fields_display(state.clone(), imported_dye_name, imported_display_name, imported_mod_name, imported_mod_author),
        ))
        .style(|s| {
            s.width_full()
                .padding(PADDING_BTN_H)
                .background(BG_CARD)
                .border(1.0)
                .border_color(BORDER_CARD)
                .border_radius(RADIUS_STD)
        }),
    ))
    .style(|s| {
        s.flex_grow(1.0)
            .flex_basis(0.0)
            .padding(PADDING_LG)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(BORDER_CARD)
            .border_radius(6.0)
    })
}
