//! UI sections for PAK Operations tab

use floem::event::{Event, EventListener};
use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::{ActiveDialog, PakOpsState};
use super::operations::{
    batch_create_paks, batch_extract_paks, create_pak_file, extract_individual_files,
    extract_pak_file, list_pak_contents, rebuild_pak_file, validate_mod_structure,
};
use super::results::is_error_message;

pub fn header_section(state: PakOpsState) -> impl IntoView {
    h_stack((
        label(|| "PAK Operations")
            .style(|s| s.font_size(18.0).font_weight(Weight::BOLD)),
        empty().style(|s| s.flex_grow(1.0)),
        // Status message
        dyn_container(
            move || state.status_message.get(),
            move |msg| {
                if msg.is_empty() {
                    empty().into_any()
                } else {
                    let is_error = is_error_message(&msg);
                    label(move || msg.clone())
                        .style(move |s| {
                            let s = s
                                .padding_horiz(12.0)
                                .padding_vert(6.0)
                                .border_radius(4.0)
                                .font_size(12.0);
                            if is_error {
                                s.background(Color::rgb8(255, 235, 235))
                                    .color(Color::rgb8(180, 30, 30))
                            } else {
                                s.background(Color::rgb8(232, 245, 233))
                                    .color(Color::rgb8(46, 125, 50))
                            }
                        })
                        .into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(16.0)
            .gap(8.0)
            .items_center()
            .background(Color::WHITE)
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

/// Main operations row with 3 columns
pub fn operations_row(state: PakOpsState) -> impl IntoView {
    h_stack((
        // Extract operations group
        extract_group(state.clone()),
        // Create operations group
        create_group(state.clone()),
        // Drop zone
        drop_zone(state),
    ))
    .style(|s| s.width_full().gap(20.0).margin_bottom(20.0))
}

fn extract_group(state: PakOpsState) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();
    let state3 = state.clone();
    let state4 = state.clone();

    v_stack((
        // Extract PAK button
        operation_button("📦 Extract PAK File", state.clone(), move || {
            extract_pak_file(state1.clone());
        }),
        // List Contents button
        operation_button("📋 List PAK Contents", state.clone(), move || {
            list_pak_contents(state2.clone());
        }),
        // Extract Individual button
        operation_button("📄 Extract Individual Files", state.clone(), move || {
            extract_individual_files(state3.clone());
        }),
        // Batch Extract button
        operation_button("📦 Batch Extract PAKs", state.clone(), move || {
            batch_extract_paks(state4.clone());
        }),
    ))
    .style(|s| {
        s.flex_grow(1.0)
            .padding(16.0)
            .gap(8.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn create_group(state: PakOpsState) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();
    let state3 = state.clone();
    let state4 = state.clone();

    v_stack((
        // Create PAK button
        operation_button("🔧 Create PAK from Folder", state.clone(), move || {
            create_pak_file(state1.clone());
        }),
        // Rebuild PAK button
        operation_button("🔧 Rebuild Modified PAK", state.clone(), move || {
            rebuild_pak_file(state2.clone());
        }),
        // Validate button
        operation_button("✓ Validate Mod Structure", state.clone(), move || {
            validate_mod_structure(state3.clone());
        }),
        // Batch Create button
        operation_button("🔧 Batch Create PAKs", state.clone(), move || {
            batch_create_paks(state4.clone());
        }),
    ))
    .style(|s| {
        s.flex_grow(1.0)
            .padding(16.0)
            .gap(8.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn drop_zone(state: PakOpsState) -> impl IntoView {
    let state_for_drop = state.clone();

    container(
        v_stack((
            label(|| "📦".to_string()).style(|s| s.font_size(32.0)),
            label(|| "Drag files here".to_string()).style(|s| {
                s.font_size(14.0)
                    .color(Color::rgb8(100, 100, 100))
                    .margin_top(8.0)
            }),
            label(|| ".pak or folder".to_string())
                .style(|s| s.font_size(12.0).color(Color::rgb8(150, 150, 150))),
        ))
        .style(|s| s.items_center()),
    )
    .on_event_cont(EventListener::DroppedFile, move |e| {
        if let Event::DroppedFile(drop_event) = e {
            let path = drop_event.path.to_string_lossy().to_string();
            let display_name = drop_event
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Check if it's a directory (for creating PAK)
            if drop_event.path.is_dir() {
                state_for_drop.dropped_folder.set(Some(path.clone()));
                state_for_drop.add_result(&format!("Dropped folder: {}", display_name));
                state_for_drop.active_dialog.set(ActiveDialog::FolderDropAction);
            }
            // Check if it's a .pak file (for extract/list)
            else if path.to_lowercase().ends_with(".pak") {
                state_for_drop.dropped_file.set(Some(path.clone()));
                state_for_drop.add_result(&format!("Dropped: {}", display_name));
                state_for_drop.active_dialog.set(ActiveDialog::DropAction);
            } else {
                state_for_drop.add_result("⚠ Only .pak files or folders can be dropped here");
            }
        }
    })
    .style(|s| {
        s.flex_grow(1.0)
            .min_height(120.0)
            .padding(16.0)
            .items_center()
            .justify_center()
            .background(Color::rgb8(249, 249, 249))
            .border(2.0)
            .border_color(Color::rgb8(204, 204, 204))
            .border_radius(8.0)
    })
}

fn operation_button(text: &'static str, state: PakOpsState, on_click: impl Fn() + 'static) -> impl IntoView {
    let state_for_action = state.clone();
    let state_for_disabled = state.clone();
    let state_for_style = state.clone();
    button(text)
        .action(move || {
            if !state_for_action.is_busy() {
                on_click();
            }
        })
        .disabled(move || state_for_disabled.is_busy())
        .style(move |s| {
            let busy = state_for_style.is_busy();
            let s = s
                .width_full()
                .padding_vert(10.0)
                .padding_horiz(16.0)
                .border(1.0)
                .border_radius(6.0);

            if busy {
                s.background(Color::rgb8(230, 230, 230))
                    .border_color(Color::rgb8(210, 210, 210))
                    .color(Color::rgb8(160, 160, 160))
            } else {
                s.background(Color::rgb8(245, 245, 245))
                    .border_color(Color::rgb8(200, 200, 200))
                    .hover(|s| {
                        s.background(Color::rgb8(230, 230, 230))
                            .border_color(Color::rgb8(180, 180, 180))
                    })
            }
        })
}
