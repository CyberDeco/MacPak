//! UI sections for Virtual Textures tab

use floem::event::{Event, EventListener};
use floem::prelude::*;
use floem::text::Weight;
use walkdir::WalkDir;

use crate::gui::shared::operation_button;
use crate::gui::state::VirtualTexturesState;
use super::extraction::{extract_batch, extract_single};

/// Main operations row with columns
pub fn operations_row(state: VirtualTexturesState) -> impl IntoView {
    h_stack((
        // Extraction operations
        extraction_group(state.clone()),
        // Drop zone
        drop_zone(state),
    ))
    .style(|s| s.width_full().gap(20.0).margin_bottom(20.0))
}

fn extraction_group(state: VirtualTexturesState) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();

    v_stack((
        // Header row with title and layer selector
        h_stack((
            label(|| "Extract Textures").style(|s| {
                s.font_size(13.0)
                    .font_weight(Weight::SEMIBOLD)
                    .color(Color::rgb8(80, 80, 80))
            }),
            empty().style(|s| s.flex_grow(1.0)),
            label(|| "Layer:").style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100))),
            layer_toggle_button("All", None, state.clone()),
            layer_toggle_button("0", Some(0), state.clone()),
            layer_toggle_button("1", Some(1), state.clone()),
            layer_toggle_button("2", Some(2), state.clone()),
        ))
        .style(|s| s.width_full().gap(4.0).items_center().margin_bottom(8.0)),
        // Extract single file
        operation_button("🖼 Extract GTS/GTP File", move || {
            select_and_extract_single(state1.clone());
        }),
        // Batch extract
        operation_button("📁 Batch Extract Directory", move || {
            select_and_extract_batch(state2.clone());
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

fn drop_zone(state: VirtualTexturesState) -> impl IntoView {
    let state_for_drop = state.clone();

    container(
        v_stack((
            label(|| "🖼".to_string()).style(|s| s.font_size(32.0)),
            label(|| "Drag files here".to_string()).style(|s| {
                s.font_size(14.0)
                    .color(Color::rgb8(100, 100, 100))
                    .margin_top(8.0)
            }),
            label(|| ".gts, .gtp".to_string())
                .style(|s| s.font_size(12.0).color(Color::rgb8(150, 150, 150))),
        ))
        .style(|s| s.items_center()),
    )
    .on_event_cont(EventListener::DroppedFile, move |e| {
        if let Event::DroppedFile(drop_event) = e {
            let path = drop_event.path.to_string_lossy().to_string();
            let path_lower = path.to_lowercase();

            let file_name = drop_event
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if path_lower.ends_with(".gts") || path_lower.ends_with(".gtp") {
                state_for_drop.add_result(&format!("Extracting: {}", file_name));
                state_for_drop.gts_file.set(Some(path));
                extract_single(state_for_drop.clone());
            } else {
                state_for_drop.add_result("Only .gts or .gtp files can be dropped here");
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

/// Toggle button for layer selection
fn layer_toggle_button(
    text: &'static str,
    value: Option<usize>,
    state: VirtualTexturesState,
) -> impl IntoView {
    button(text)
        .action(move || state.selected_layer.set(value))
        .style(move |s| {
            let is_selected = state.selected_layer.get() == value;
            let s = s
                .padding_vert(4.0)
                .padding_horiz(8.0)
                .border_radius(4.0)
                .font_size(11.0)
                .cursor(floem::style::CursorStyle::Pointer);
            if is_selected {
                s.background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
                    .border(1.0)
                    .border_color(Color::rgb8(33, 150, 243))
            } else {
                s.background(Color::rgb8(245, 245, 245))
                    .color(Color::rgb8(100, 100, 100))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .hover(|s| s.background(Color::rgb8(235, 235, 235)))
            }
        })
}

/// Select a GTS/GTP file and extract it
fn select_and_extract_single(state: VirtualTexturesState) {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Select GTS/GTP File")
        .add_filter("Virtual Texture Files", &["gts", "gtp"]);

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(file) = dialog.pick_file() {
        if let Some(parent) = file.parent() {
            state.working_dir.set(Some(parent.to_string_lossy().to_string()));
        }
        state.gts_file.set(Some(file.to_string_lossy().to_string()));
        extract_single(state);
    }
}

/// Select a directory and batch extract all GTS files
fn select_and_extract_batch(state: VirtualTexturesState) {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Select Directory with GTS Files");

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(dir) = dialog.pick_folder() {
        state.working_dir.set(Some(dir.to_string_lossy().to_string()));
        state.batch_input_dir.set(Some(dir.to_string_lossy().to_string()));

        // Scan for GTS files only
        let mut files = Vec::new();
        for entry in WalkDir::new(&dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "gts" {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        files.sort();
        state.batch_gts_files.set(files);
        extract_batch(state);
    }
}
