//! UI sections for Virtual Textures tab

use floem::event::Event;
use floem::prelude::*;
use floem::text::Weight;
use maclarian::converter::DdsFormat;
use walkdir::WalkDir;

use super::extraction::{
    convert_dds_png_batch, convert_dds_to_png_file, convert_png_to_dds_file, extract_batch,
    extract_from_pak, extract_single,
};
use crate::gui::shared::{card_style, drop_zone, operation_button};
use crate::gui::state::{ConfigState, VirtualTexturesState};

/// Main operations row with columns
pub fn operations_row(state: VirtualTexturesState, config: ConfigState) -> impl IntoView {
    h_stack((
        // Extraction operations
        extraction_group(state.clone(), config.clone()),
        // DDS ↔ PNG conversion
        dds_png_group(state.clone()),
        // Drop zone
        vt_drop_zone(state.clone(), config.clone()),
    ))
    .style(|s| s.width_full().gap(20.0).margin_bottom(20.0))
}

fn extraction_group(state: VirtualTexturesState, config: ConfigState) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();
    let state3 = state.clone();
    let config1 = config.clone();
    let config2 = config;

    v_stack((
        // Header row with title and layer selector
        h_stack((
            label(|| "Extract Textures").style(|s| {
                s.font_size(13.0)
                    .font_weight(Weight::SEMIBOLD)
                    .color(Color::rgb8(80, 80, 80))
            }),
            empty().style(|s| s.flex_grow(1.0)),
            // Layer selector
            label(|| "Layer:").style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100))),
            layer_toggle_button("All", None, state.clone()),
            layer_toggle_button("0", Some(0), state.clone()),
            layer_toggle_button("1", Some(1), state.clone()),
            layer_toggle_button("2", Some(2), state.clone()),
        ))
        .style(|s| s.width_full().gap(4.0).items_center().margin_bottom(8.0)),
        // Extract single file
        operation_button("Extract GTS/GTP File", move || {
            select_and_extract_single(state1.clone(), config1.clone());
        }),
        // Batch extract
        operation_button("Batch Extract Directory", move || {
            select_and_extract_batch(state2.clone(), config2.clone());
        }),
        // Extract by GTex hash
        operation_button("Extract by GTex Hash", move || {
            state3.show_gtex_dialog.set(true);
        }),
    ))
    .style(|s| card_style(s).flex_grow(1.0).flex_basis(0.0).gap(8.0))
}

fn dds_png_group(state: VirtualTexturesState) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();
    let state3 = state.clone();
    let dds_format = state.dds_format;

    v_stack((
        // Header row with title and format selector
        h_stack((
            label(|| "DDS \u{2194} PNG").style(|s| {
                s.font_size(13.0)
                    .font_weight(Weight::SEMIBOLD)
                    .color(Color::rgb8(80, 80, 80))
            }),
            empty().style(|s| s.flex_grow(1.0)),
            label(|| "Format:").style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100))),
            format_toggle_button("BC1", DdsFormat::BC1, dds_format),
            format_toggle_button("BC3", DdsFormat::BC3, dds_format),
            format_toggle_button("RGBA", DdsFormat::Rgba, dds_format),
        ))
        .style(|s| s.width_full().gap(4.0).items_center().margin_bottom(8.0)),
        // DDS → PNG
        operation_button("DDS \u{2192} PNG", move || {
            convert_dds_to_png_file(state1.clone());
        }),
        // PNG → DDS
        operation_button("PNG \u{2192} DDS", move || {
            convert_png_to_dds_file(state2.clone());
        }),
        // Batch convert
        operation_button("Batch Convert Directory", move || {
            convert_dds_png_batch(state3.clone());
        }),
    ))
    .style(|s| card_style(s).flex_grow(1.0).flex_basis(0.0).gap(8.0))
}

fn vt_drop_zone(state: VirtualTexturesState, config: ConfigState) -> impl IntoView {
    let state_for_drop = state.clone();
    let config_for_drop = config;

    drop_zone("\u{1f5bc}", ".gts, .gtp, .pak\n.dds, .png", false, move |e| {
        if let Event::DroppedFile(drop_event) = e {
            let path = drop_event.path.to_string_lossy().to_string();
            let path_lower = path.to_lowercase();

            let file_name = drop_event
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if path_lower.ends_with(".gts") || path_lower.ends_with(".gtp") {
                state_for_drop.from_pak.set(false);
                state_for_drop.add_result(&format!("Extracting: {}", file_name));
                state_for_drop.gts_file.set(Some(path));
                let game_data = config_for_drop.bg3_data_path.get_untracked();
                extract_single(state_for_drop.clone(), game_data);
            } else if path_lower.ends_with(".dds") {
                state_for_drop.add_result(&format!("Converting DDS \u{2192} PNG: {}", file_name));
                let dds_path = drop_event.path.clone();
                let png_path = dds_path.with_extension("png");
                let output_name = png_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                state_for_drop.is_extracting.set(true);
                state_for_drop
                    .status_message
                    .set("Converting DDS \u{2192} PNG...".to_string());

                let send_result = super::types::create_result_sender(state_for_drop.clone());
                let input_name_clone = file_name.clone();
                let output_name_clone = output_name.clone();
                std::thread::spawn(move || {
                    match maclarian::converter::convert_dds_to_png(&dds_path, &png_path) {
                        Ok(()) => {
                            send_result(super::types::VtResult::DdsConvertDone {
                                success: true,
                                input_name: input_name_clone,
                                output_name: output_name_clone,
                                error: None,
                            });
                        }
                        Err(e) => {
                            send_result(super::types::VtResult::DdsConvertDone {
                                success: false,
                                input_name: input_name_clone,
                                output_name: output_name_clone,
                                error: Some(e.to_string()),
                            });
                        }
                    }
                });
            } else if path_lower.ends_with(".png") {
                let format = state_for_drop.dds_format.get_untracked();
                state_for_drop.add_result(&format!(
                    "Converting PNG \u{2192} DDS ({:?}): {}",
                    format, file_name
                ));
                let png_path = drop_event.path.clone();
                let dds_path = png_path.with_extension("dds");
                let output_name = dds_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                state_for_drop.is_extracting.set(true);
                state_for_drop
                    .status_message
                    .set(format!("Converting PNG \u{2192} DDS ({:?})...", format));

                let send_result = super::types::create_result_sender(state_for_drop.clone());
                let input_name_clone = file_name.clone();
                let output_name_clone = output_name.clone();
                std::thread::spawn(move || {
                    match maclarian::converter::convert_png_to_dds_with_format(
                        &png_path, &dds_path, format,
                    ) {
                        Ok(()) => {
                            send_result(super::types::VtResult::DdsConvertDone {
                                success: true,
                                input_name: input_name_clone,
                                output_name: output_name_clone,
                                error: None,
                            });
                        }
                        Err(e) => {
                            send_result(super::types::VtResult::DdsConvertDone {
                                success: false,
                                input_name: input_name_clone,
                                output_name: output_name_clone,
                                error: Some(e.to_string()),
                            });
                        }
                    }
                });
            } else if path_lower.ends_with(".pak") {
                // PAK file: prompt for output directory and extract VT files
                let mut out_dialog = rfd::FileDialog::new()
                    .set_title("Select Output Directory for VT Extraction");
                if let Some(dir) = state_for_drop.working_dir.get() {
                    out_dialog = out_dialog.set_directory(&dir);
                }
                if let Some(out_dir) = out_dialog.pick_folder() {
                    state_for_drop
                        .working_dir
                        .set(Some(out_dir.to_string_lossy().to_string()));
                    state_for_drop.add_result(&format!("Extracting VT from PAK: {}", file_name));
                    extract_from_pak(
                        state_for_drop.clone(),
                        path,
                        out_dir.to_string_lossy().to_string(),
                    );
                }
            } else {
                state_for_drop.add_result("Drop .gts, .gtp, .pak, .dds, or .png files here");
            }
        }
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

/// Toggle button for DDS format selection
fn format_toggle_button(
    text: &'static str,
    value: DdsFormat,
    signal: RwSignal<DdsFormat>,
) -> impl IntoView {
    button(text)
        .action(move || signal.set(value))
        .style(move |s| {
            let is_selected = signal.get() == value;
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

/// Select a GTS/GTP or PAK file and extract it
fn select_and_extract_single(state: VirtualTexturesState, config: ConfigState) {
    let game_data = config.bg3_data_path.get_untracked();

    let mut dialog = rfd::FileDialog::new()
        .set_title("Select GTS/GTP or PAK File")
        .add_filter("Virtual Texture Files", &["gts", "gtp", "pak"]);

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(file) = dialog.pick_file() {
        if let Some(parent) = file.parent() {
            state
                .working_dir
                .set(Some(parent.to_string_lossy().to_string()));
        }

        let is_pak = file
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase() == "pak")
            .unwrap_or(false);

        if is_pak {
            // PAK file: prompt for output directory and extract VT files
            let mut out_dialog =
                rfd::FileDialog::new().set_title("Select Output Directory for VT Extraction");
            if let Some(dir) = state.working_dir.get() {
                out_dialog = out_dialog.set_directory(&dir);
            }
            if let Some(out_dir) = out_dialog.pick_folder() {
                state
                    .working_dir
                    .set(Some(out_dir.to_string_lossy().to_string()));
                extract_from_pak(
                    state,
                    file.to_string_lossy().to_string(),
                    out_dir.to_string_lossy().to_string(),
                );
            }
        } else {
            // Loose GTS/GTP file
            state.from_pak.set(false);
            state.gts_file.set(Some(file.to_string_lossy().to_string()));
            extract_single(state, game_data);
        }
    }
}

/// Select a directory and batch extract all GTS files
fn select_and_extract_batch(state: VirtualTexturesState, config: ConfigState) {
    let game_data = config.bg3_data_path.get_untracked();

    let mut dialog = rfd::FileDialog::new().set_title("Select Directory with GTS Files");

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(dir) = dialog.pick_folder() {
        state
            .working_dir
            .set(Some(dir.to_string_lossy().to_string()));
        state
            .batch_input_dir
            .set(Some(dir.to_string_lossy().to_string()));

        // Directory mode = Files (not PAK)
        state.from_pak.set(false);

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
        extract_batch(state, game_data);
    }
}

/// Public function to open a GTS file for extraction (CMD+O shortcut)
pub fn open_gts_file(state: VirtualTexturesState, config: ConfigState) {
    select_and_extract_single(state, config);
}
