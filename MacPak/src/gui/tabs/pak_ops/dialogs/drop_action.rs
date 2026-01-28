//! Drop action dialog content (for .pak files)

use floem::prelude::*;
use floem::text::Weight;
use std::path::Path;

use super::super::operations::{
    extract_dropped_file, extract_individual_dropped_file, list_dropped_file, validate_dropped_pak,
};
use crate::gui::state::{ActiveDialog, PakOpsState};

pub fn drop_action_content(state: PakOpsState) -> impl IntoView {
    let dropped_file = state.dropped_file;
    let file_path = dropped_file.get().unwrap_or_default();
    let file_name = Path::new(&file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "PAK file".to_string());

    let state_extract = state.clone();
    let state_individual = state.clone();
    let state_list = state.clone();
    let state_validate = state.clone();
    let state_cancel = state.clone();
    let file_path_extract = file_path.clone();
    let file_path_individual = file_path.clone();
    let file_path_list = file_path.clone();
    let file_path_validate = file_path.clone();

    v_stack((
        label(move || format!("Dropped: {}", file_name)).style(|s| {
            s.font_size(16.0)
                .font_weight(Weight::BOLD)
                .margin_bottom(16.0)
        }),
        label(|| "What would you like to do?".to_string()).style(|s| s.margin_bottom(12.0)),
        button("📦 Extract PAK")
            .action(move || {
                state_extract.active_dialog.set(ActiveDialog::None);
                extract_dropped_file(state_extract.clone(), file_path_extract.clone());
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
        button("📄 Extract Individual Files")
            .action(move || {
                state_individual.active_dialog.set(ActiveDialog::None);
                extract_individual_dropped_file(
                    state_individual.clone(),
                    file_path_individual.clone(),
                );
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
        button("📋 List Contents")
            .action(move || {
                state_list.active_dialog.set(ActiveDialog::None);
                list_dropped_file(state_list.clone(), file_path_list.clone());
            })
            .style(|s| {
                s.width_full()
                    .padding_vert(10.0)
                    .margin_bottom(8.0)
                    .background(Color::rgb8(76, 175, 80))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(56, 142, 60)))
            }),
        button("✓ Validate Mod Structure")
            .action(move || {
                validate_dropped_pak(state_validate.clone(), file_path_validate.clone());
            })
            .style(|s| {
                s.width_full()
                    .padding_vert(10.0)
                    .margin_bottom(8.0)
                    .background(Color::rgb8(255, 152, 0))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(245, 124, 0)))
            }),
        button("Cancel")
            .action(move || {
                state_cancel.dropped_file.set(None);
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
