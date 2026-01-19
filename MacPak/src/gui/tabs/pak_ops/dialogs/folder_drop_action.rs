//! Folder drop action dialog content

use floem::prelude::*;
use floem::text::Weight;
use std::path::Path;

use crate::gui::state::{ActiveDialog, PakOpsState};
use super::super::operations::{
    create_pak_from_dropped_folder, rebuild_pak_from_dropped_folder, validate_dropped_folder,
};

pub fn folder_drop_action_content(state: PakOpsState) -> impl IntoView {
    let dropped_folder = state.dropped_folder;
    let folder_path = dropped_folder.get().unwrap_or_default();
    let folder_name = Path::new(&folder_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "folder".to_string());

    let state_create = state.clone();
    let state_rebuild = state.clone();
    let state_validate = state.clone();
    let state_cancel = state.clone();
    let folder_path_create = folder_path.clone();
    let folder_path_rebuild = folder_path.clone();
    let folder_path_validate = folder_path.clone();

    v_stack((
        label(move || format!("Dropped: {}", folder_name)).style(|s| {
            s.font_size(16.0)
                .font_weight(Weight::BOLD)
                .margin_bottom(16.0)
        }),
        label(|| "What would you like to do?".to_string())
            .style(|s| s.margin_bottom(12.0)),
        button("🔧 Create PAK from Folder")
            .action(move || {
                state_create.active_dialog.set(ActiveDialog::None);
                create_pak_from_dropped_folder(state_create.clone(), folder_path_create.clone());
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
        button("🔧 Rebuild Modified PAK")
            .action(move || {
                state_rebuild.active_dialog.set(ActiveDialog::None);
                rebuild_pak_from_dropped_folder(state_rebuild.clone(), folder_path_rebuild.clone());
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
                state_validate.active_dialog.set(ActiveDialog::None);
                validate_dropped_folder(state_validate.clone(), folder_path_validate.clone());
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
                state_cancel.dropped_folder.set(None);
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
