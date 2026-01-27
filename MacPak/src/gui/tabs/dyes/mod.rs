//! Dyes Tab - Custom dye color creator for BG3 modding

mod color_row;
mod export;
mod generate;
mod import;
mod sections;
pub mod shared;

use floem::prelude::*;

use crate::gui::state::{AppState, DyesState};
use crate::gui::utils::meta_dialog::{meta_dialog_with_signals_and_extra, MetaDialogSignals};
use crate::gui::utils::vendor_selection_section;
use export::export_section;
use generate::generate_dye_section;
use import::import_section;
pub use import::import_from_mod_folder;
use sections::{common_section, header_section, recommended_section, required_section};
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
        // meta.lsx is generated as part of export_dye_mod
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
