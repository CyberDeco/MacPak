//! Validate choice dialog - select folder or PAK file

use floem::prelude::*;
use floem::text::Weight;

use super::super::operations::{validate_mod_structure, validate_pak_mod_structure};
use crate::gui::state::{ActiveDialog, PakOpsState};

pub fn validate_choice_content(state: PakOpsState) -> impl IntoView {
    let state_folder = state.clone();
    let state_pak = state.clone();
    let state_cancel = state.clone();

    v_stack((
        label(|| "Validate Mod Structure").style(|s| {
            s.font_size(16.0)
                .font_weight(Weight::BOLD)
                .margin_bottom(16.0)
        }),
        label(|| "What would you like to validate?".to_string()).style(|s| s.margin_bottom(12.0)),
        button("📁 Select Folder")
            .action(move || {
                state_folder.active_dialog.set(ActiveDialog::None);
                validate_mod_structure(state_folder.clone());
            })
            .style(|s| {
                s.width_full()
                    .padding_vert(10.0)
                    .margin_bottom(8.0)
                    .background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(25, 118, 210)))
            }),
        button("📦 Select PAK File")
            .action(move || {
                state_pak.active_dialog.set(ActiveDialog::None);
                validate_pak_mod_structure(state_pak.clone());
            })
            .style(|s| {
                s.width_full()
                    .padding_vert(10.0)
                    .margin_bottom(8.0)
                    .background(Color::rgb8(156, 39, 176))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(123, 31, 162)))
            }),
        button("Cancel")
            .action(move || {
                state_cancel.active_dialog.set(ActiveDialog::None);
            })
            .style(|s| {
                s.width_full()
                    .padding_vert(10.0)
                    .background(Color::rgb8(240, 240, 240))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
    ))
    .style(|s| {
        s.padding(24.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(200, 200, 200))
            .border_radius(8.0)
            .width(320.0)
    })
}
