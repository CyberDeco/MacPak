//! Create options dialog content

use floem::prelude::*;
use floem::text::Weight;

use super::super::operations::execute_create_pak;
use super::super::widgets::{compression_selector, priority_input};
use crate::gui::state::{ActiveDialog, PakOpsState};

pub fn create_options_content(state: PakOpsState) -> impl IntoView {
    let compression = state.compression;
    let priority = state.priority;
    let generate_info_json = state.generate_info_json;
    let pending = state.pending_create;
    let state_confirm = state.clone();
    let state_cancel = state.clone();

    v_stack((
        label(|| "PAK Creation Options".to_string()).style(|s| {
            s.font_size(18.0)
                .font_weight(Weight::BOLD)
                .margin_bottom(16.0)
        }),
        h_stack((
            label(|| "Compression:".to_string()).style(|s| s.width(120.0)),
            compression_selector(compression),
        ))
        .style(|s| s.width_full().items_center().margin_bottom(12.0)),
        h_stack((
            label(|| "Load Priority:".to_string()).style(|s| s.width(120.0)),
            priority_input(priority),
        ))
        .style(|s| s.width_full().items_center().margin_bottom(12.0)),
        h_stack((
            checkbox(move || generate_info_json.get())
                .on_update(move |checked| {
                    generate_info_json.set(checked);
                })
                .style(|s| s.margin_right(8.0)),
            label(|| "Generate info.json (for BaldursModManager)".to_string())
                .on_click_stop(move |_| {
                    generate_info_json.set(!generate_info_json.get());
                })
                .style(|s| s.cursor(floem::style::CursorStyle::Pointer)),
        ))
        .style(|s| s.width_full().items_center().margin_bottom(12.0)),
        label(|| {
            "lz4hc = best compression (default)\n\
             lz4 = fast compression, none = no compression\n\
             Priority 0 = normal mod, 50+ = override mod\n\
             info.json enables drag-and-drop import in BaldursModManager"
                .to_string()
        })
        .style(|s| {
            s.font_size(11.0)
                .color(Color::rgb8(100, 100, 100))
                .margin_bottom(16.0)
        }),
        h_stack((
            empty().style(|s| s.flex_grow(1.0)),
            button("Cancel")
                .action(move || {
                    state_cancel.pending_create.set(None);
                    state_cancel.active_dialog.set(ActiveDialog::None);
                })
                .style(|s| {
                    s.padding_vert(8.0)
                        .padding_horiz(20.0)
                        .margin_right(8.0)
                        .background(Color::rgb8(240, 240, 240))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                }),
            button("Create PAK")
                .action(move || {
                    if let Some((source, dest)) = pending.get() {
                        state_confirm.active_dialog.set(ActiveDialog::None);
                        execute_create_pak(state_confirm.clone(), source, dest);
                    }
                })
                .style(|s| {
                    s.padding_vert(8.0)
                        .padding_horiz(20.0)
                        .background(Color::rgb8(33, 150, 243))
                        .color(Color::WHITE)
                        .border_radius(4.0)
                        .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                }),
        ))
        .style(|s| s.width_full()),
    ))
    .style(|s| {
        s.padding(24.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(200, 200, 200))
            .border_radius(8.0)
            .width(400.0)
    })
}
