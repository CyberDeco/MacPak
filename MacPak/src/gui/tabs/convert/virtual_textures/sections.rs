//! UI sections for Virtual Textures tab

use floem::event::Event;
use floem::prelude::*;
use floem::text::Weight;
use walkdir::WalkDir;

use super::extraction::{extract_batch, extract_single};
use crate::gui::shared::{card_style, checkbox_option, drop_zone, operation_button};
use crate::gui::state::{ConfigState, VirtualTexturesState};

/// Main operations row with columns
pub fn operations_row(state: VirtualTexturesState, config: ConfigState) -> impl IntoView {
    v_stack((
        h_stack((
            // Extraction operations
            extraction_group(state.clone(), config.clone()),
            // Drop zone
            vt_drop_zone(state.clone(), config.clone()),
        ))
        .style(|s| s.width_full().gap(20.0)),
        // Options panel
        options_panel(state, config),
    ))
    .style(|s| s.gap(16.0).margin_bottom(20.0))
}

fn extraction_group(state: VirtualTexturesState, config: ConfigState) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();
    let config1 = config.clone();
    let config2 = config;

    let from_pak = state.from_pak;

    v_stack((
        // Header row with title, source toggle, and layer selector
        h_stack((
            label(|| "Extract Textures").style(|s| {
                s.font_size(13.0)
                    .font_weight(Weight::SEMIBOLD)
                    .color(Color::rgb8(80, 80, 80))
            }),
            empty().style(|s| s.flex_grow(1.0)),
            // Source toggle
            label(|| "Source:").style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100))),
            source_toggle_button("PAK", true, from_pak),
            source_toggle_button("Files", false, from_pak),
            empty().style(|s| s.width(16.0)),
            // Layer selector
            label(|| "Layer:").style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100))),
            layer_toggle_button("All", None, state.clone()),
            layer_toggle_button("0", Some(0), state.clone()),
            layer_toggle_button("1", Some(1), state.clone()),
            layer_toggle_button("2", Some(2), state.clone()),
        ))
        .style(|s| s.width_full().gap(4.0).items_center().margin_bottom(8.0)),
        // Extract single file
        operation_button("🖼 Extract GTS/GTP File", move || {
            select_and_extract_single(state1.clone(), config1.clone());
        }),
        // Batch extract
        operation_button("📁 Batch Extract Directory", move || {
            select_and_extract_batch(state2.clone(), config2.clone());
        }),
    ))
    .style(|s| card_style(s).flex_grow(1.0).gap(8.0))
}

fn vt_drop_zone(state: VirtualTexturesState, config: ConfigState) -> impl IntoView {
    let state_for_drop = state.clone();
    let config_for_drop = config;

    drop_zone("🖼", ".gts, .gtp", false, move |e| {
        if let Event::DroppedFile(drop_event) = e {
            let path = drop_event.path.to_string_lossy().to_string();
            let path_lower = path.to_lowercase();

            let file_name = drop_event
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if path_lower.ends_with(".gts") || path_lower.ends_with(".gtp") {
                // Dropped files always use Files mode (not PAK)
                state_for_drop.from_pak.set(false);
                state_for_drop.add_result(&format!("Extracting: {}", file_name));
                state_for_drop.gts_file.set(Some(path));
                let game_data = config_for_drop.bg3_data_path.get_untracked();
                extract_single(state_for_drop.clone(), game_data);
            } else {
                state_for_drop.add_result("Only .gts or .gtp files can be dropped here");
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

/// Toggle button for source selection (PAK/Files)
fn source_toggle_button(text: &'static str, value: bool, signal: RwSignal<bool>) -> impl IntoView {
    button(text)
        .action(move || signal.set(value))
        .style(move |s| {
            let is_selected = signal.get() == value;
            let s = s
                .padding_vert(4.0)
                .padding_horiz(12.0)
                .border_radius(4.0)
                .font_size(12.0)
                .cursor(floem::style::CursorStyle::Pointer);
            if is_selected {
                s.background(Color::rgb8(76, 175, 80))
                    .color(Color::WHITE)
                    .border(1.0)
                    .border_color(Color::rgb8(76, 175, 80))
            } else {
                s.background(Color::rgb8(245, 245, 245))
                    .color(Color::rgb8(100, 100, 100))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .hover(|s| s.background(Color::rgb8(235, 235, 235)))
            }
        })
}

/// Options panel for extraction settings
fn options_panel(state: VirtualTexturesState, config: ConfigState) -> impl IntoView {
    let convert_png = state.convert_to_png;
    let from_pak = state.from_pak;
    let game_data_path = config.bg3_data_path;

    // Show warning only when PAK mode is enabled AND game data path is not set
    let needs_game_data_warning = move || {
        let pak_enabled = from_pak.get();
        let path_missing = game_data_path.get().is_empty();
        pak_enabled && path_missing
    };

    v_stack((
        // Header
        label(|| "Options").style(|s| {
            s.font_size(13.0)
                .font_weight(Weight::SEMIBOLD)
                .color(Color::rgb8(80, 80, 80))
                .margin_bottom(8.0)
        }),
        // Checkboxes
        h_stack((checkbox_option("Convert to PNG", convert_png),))
            .style(|s| s.gap(24.0).items_center()),
        // Warning only shown when PAK mode enabled but path not configured
        dyn_container(
            move || needs_game_data_warning(),
            move |show_warning| {
                if show_warning {
                    label(|| "⚠ BG3 game data path not set in Settings")
                        .style(|s| {
                            s.font_size(11.0)
                                .color(Color::rgb8(180, 80, 30))
                                .margin_top(8.0)
                        })
                        .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(16.0)
            .background(Color::rgb8(248, 248, 252))
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 230))
            .border_radius(8.0)
    })
}

/// Select a GTS/GTP file and extract it
fn select_and_extract_single(state: VirtualTexturesState, config: ConfigState) {
    let game_data = config.bg3_data_path.get_untracked();

    let mut dialog = rfd::FileDialog::new()
        .set_title("Select GTS/GTP File")
        .add_filter("Virtual Texture Files", &["gts", "gtp"]);

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(file) = dialog.pick_file() {
        if let Some(parent) = file.parent() {
            state
                .working_dir
                .set(Some(parent.to_string_lossy().to_string()));
        }
        state.gts_file.set(Some(file.to_string_lossy().to_string()));
        extract_single(state, game_data);
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
