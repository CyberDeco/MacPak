//! Import functionality for the Dyes tab

mod components;
mod operations;

use floem::prelude::*;
use floem::text::Weight;

use crate::state::DyesState;
use super::shared::secondary_button_style;

use components::{imported_fields_display, txt_import_selector, lsf_import_selector};
use operations::{import_from_file, import_from_lsf};

/// Import section UI for loading existing dye definitions
pub fn import_section(state: DyesState) -> impl IntoView {
    // Local signals for displaying imported data (independent from export)
    let imported_dye_name: RwSignal<String> = RwSignal::new(String::new());
    let imported_preset_uuid: RwSignal<String> = RwSignal::new(String::new());
    let imported_template_uuid: RwSignal<String> = RwSignal::new(String::new());

    v_stack((
        // Section header
        h_stack((
            label(|| "Import")
                .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD)),
            empty().style(|s| s.flex_grow(1.0)),
            {
                let state = state.clone();
                let imported_dye_name = imported_dye_name;
                let imported_preset_uuid = imported_preset_uuid;
                let imported_template_uuid = imported_template_uuid;
                label(|| "From LSF...")
                    .style(secondary_button_style)
                    .on_click_stop(move |_| {
                        import_from_lsf(state.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);
                    })
            },
            {
                let state = state.clone();
                let imported_dye_name = imported_dye_name;
                let imported_preset_uuid = imported_preset_uuid;
                let imported_template_uuid = imported_template_uuid;
                label(|| "From TXT...")
                    .style(secondary_button_style)
                    .on_click_stop(move |_| {
                        import_from_file(state.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);
                    })
            },
        ))
        .style(|s| s.width_full().items_center().gap(8.0).margin_bottom(8.0)),

        // Inner card with imported data display
        v_stack((
            // TXT Import selector (shows when entries are loaded)
            txt_import_selector(state.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid),

            // LSF Import selector (shows when LSF entries are loaded)
            lsf_import_selector(state.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid),

            // Display imported values (editable name)
            imported_fields_display(state.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid),
        ))
        .style(|s| {
            s.width_full()
                .padding(12.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.flex_grow(1.0)
            .flex_basis(0.0)
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
    })
}
