//! UI sections for GR2 conversion tab

use floem::event::Event;
use floem::prelude::*;
use floem::text::Weight;

use super::conversion::{convert_batch_with_options, convert_single_with_options};
use crate::gui::shared::{card_style, checkbox_option, drop_zone, operation_button};
use crate::gui::state::{ConfigState, Gr2State};

/// Main operations row with 3 columns
pub fn operations_row(state: Gr2State, config: ConfigState) -> impl IntoView {
    // Toggle state for output format: true = GLB, false = glTF
    // Shared between the conversion group and bundle options panel
    let use_glb = RwSignal::new(true);

    // Clone before moving into bundle_options_panel
    let state_for_drop = state.clone();
    let config_for_drop = config.clone();

    h_stack((
        // Left column: card groups + bundle options
        v_stack((
            h_stack((
                gr2_to_gltf_group(state.clone(), config.clone(), use_glb),
                gltf_to_gr2_group(state.clone()),
            ))
            .style(|s| s.width_full().gap(20.0)),
            bundle_options_panel(state, config, use_glb),
        ))
        .style(|s| s.flex_grow(1.0).flex_basis(0.0).gap(16.0)),
        // Drop zone to the right, stretching full height
        gr2_drop_zone(state_for_drop, config_for_drop, use_glb),
    ))
    .style(|s| s.width_full().gap(20.0).margin_bottom(20.0))
}

fn gr2_to_gltf_group(
    state: Gr2State,
    config: ConfigState,
    use_glb: RwSignal<bool>,
) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();
    let config1 = config.clone();
    let config2 = config;

    v_stack((
        // Header row with title and format toggle
        h_stack((
            label(|| "GR2 → glTF/GLB").style(|s| {
                s.font_size(13.0)
                    .font_weight(Weight::SEMIBOLD)
                    .color(Color::rgb8(80, 80, 80))
            }),
            empty().style(|s| s.flex_grow(1.0)),
            format_toggle_button("GLB", true, use_glb),
            format_toggle_button("glTF", false, use_glb),
        ))
        .style(|s| s.width_full().gap(4.0).items_center().margin_bottom(8.0)),
        // Convert single file
        dynamic_operation_button(
            use_glb,
            |glb| {
                if glb {
                    "🔄 Convert GR2 → GLB"
                } else {
                    "🔄 Convert GR2 → glTF"
                }
            },
            move || {
                let to_glb = use_glb.get_untracked();
                select_and_convert_gr2(state1.clone(), config1.clone(), to_glb, false);
            },
        ),
        // Batch convert
        dynamic_operation_button(
            use_glb,
            |glb| {
                if glb {
                    "📁 Batch GR2 → GLB"
                } else {
                    "📁 Batch GR2 → glTF"
                }
            },
            move || {
                let to_glb = use_glb.get_untracked();
                select_and_convert_gr2(state2.clone(), config2.clone(), to_glb, true);
            },
        ),
    ))
    .style(|s| card_style(s).flex_grow(1.0).flex_basis(0.0).gap(8.0))
}

fn gltf_to_gr2_group(state: Gr2State) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();

    v_stack((
        label(|| "glTF/GLB → GR2").style(|s| {
            s.font_size(13.0)
                .font_weight(Weight::SEMIBOLD)
                .color(Color::rgb8(80, 80, 80))
                .margin_bottom(14.0)
                .margin_top(4.0)
        }),
        // Convert glTF/GLB to GR2 (single)
        operation_button("🔄 Convert glTF/GLB → GR2", move || {
            select_and_convert_gltf(state1.clone(), false);
        }),
        // Batch convert glTF/GLB to GR2
        operation_button("📁 Batch glTF/GLB → GR2", move || {
            select_and_convert_gltf(state2.clone(), true);
        }),
    ))
    .style(|s| card_style(s).flex_grow(1.0).flex_basis(0.0).gap(8.0))
}

fn gr2_drop_zone(state: Gr2State, config: ConfigState, use_glb: RwSignal<bool>) -> impl IntoView {
    let state_for_drop = state.clone();
    let config_for_drop = config;

    drop_zone("🦴", ".gr2, .glb, .gltf", false, move |e| {
        if let Event::DroppedFile(drop_event) = e {
            let path = drop_event.path.to_string_lossy().to_string();
            let path_lower = path.to_lowercase();

            let file_name = drop_event
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if path_lower.ends_with(".gr2") {
                // GR2 file dropped - convert using selected format
                let to_glb = use_glb.get_untracked();
                let format = if to_glb { "GLB" } else { "glTF" };
                state_for_drop.add_result(&format!("Converting to {}: {}", format, file_name));
                state_for_drop.input_file.set(Some(path));
                let game_data = config_for_drop.bg3_data_path.get_untracked();
                convert_single_with_options(state_for_drop.clone(), to_glb, game_data);
            } else if path_lower.ends_with(".glb") || path_lower.ends_with(".gltf") {
                // glTF file dropped - convert to GR2
                state_for_drop.add_result(&format!("Converting: {}", file_name));
                state_for_drop.input_file.set(Some(path));
                convert_single_with_options(state_for_drop.clone(), false, String::new()); // GR2 output (no bundle)
            } else {
                state_for_drop.add_result("⚠ Only .gr2, .glb, or .gltf files can be dropped here");
            }
        }
    })
}

/// Toggle button for format selection
fn format_toggle_button(text: &'static str, value: bool, signal: RwSignal<bool>) -> impl IntoView {
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
                s.background(Color::rgb8(66, 133, 244))
                    .color(Color::WHITE)
                    .border(1.0)
                    .border_color(Color::rgb8(66, 133, 244))
            } else {
                s.background(Color::rgb8(245, 245, 245))
                    .color(Color::rgb8(100, 100, 100))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .hover(|s| s.background(Color::rgb8(235, 235, 235)))
            }
        })
}

/// Operation button with dynamic text based on a signal
fn dynamic_operation_button(
    signal: RwSignal<bool>,
    label_fn: fn(bool) -> &'static str,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    button(label(move || label_fn(signal.get())))
        .action(on_click)
        .style(|s| {
            s.width_full()
                .padding_vert(10.0)
                .padding_horiz(16.0)
                .background(Color::rgb8(245, 245, 245))
                .border(1.0)
                .border_color(Color::rgb8(200, 200, 200))
                .border_radius(6.0)
                .hover(|s| {
                    s.background(Color::rgb8(230, 230, 230))
                        .border_color(Color::rgb8(180, 180, 180))
                })
        })
}

/// Select a GR2 file and convert it
fn select_and_convert_gr2(state: Gr2State, config: ConfigState, to_glb: bool, batch: bool) {
    let game_data = config.bg3_data_path.get_untracked();

    if batch {
        // Batch mode - select directory
        let mut dialog = rfd::FileDialog::new().set_title("Select Directory with GR2 Files");

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

            // Scan for GR2 files
            let mut files = Vec::new();
            for entry in walkdir::WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().to_lowercase() == "gr2" {
                            files.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
            files.sort();
            state.batch_files.set(files);

            // Ask for output folder
            let dest_dialog = rfd::FileDialog::new()
                .set_title("Select Output Folder for Converted Files")
                .set_directory(&dir);

            let Some(dest_dir) = dest_dialog.pick_folder() else {
                return;
            };

            convert_batch_with_options(
                state,
                to_glb,
                game_data,
                dest_dir.to_string_lossy().to_string(),
            );
        }
    } else {
        // Single file mode
        let mut dialog = rfd::FileDialog::new()
            .set_title("Select GR2 File")
            .add_filter("GR2 Files", &["gr2"]);

        if let Some(dir) = state.working_dir.get() {
            dialog = dialog.set_directory(&dir);
        }

        if let Some(file) = dialog.pick_file() {
            if let Some(parent) = file.parent() {
                state
                    .working_dir
                    .set(Some(parent.to_string_lossy().to_string()));
            }
            state
                .input_file
                .set(Some(file.to_string_lossy().to_string()));
            convert_single_with_options(state, to_glb, game_data);
        }
    }
}

/// Select a glTF/GLB file and convert it to GR2
fn select_and_convert_gltf(state: Gr2State, batch: bool) {
    if batch {
        // Batch mode - select directory
        let mut dialog = rfd::FileDialog::new().set_title("Select Directory with glTF/GLB Files");

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

            // Scan for glTF/GLB files
            let mut files = Vec::new();
            for entry in walkdir::WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_lower = ext.to_string_lossy().to_lowercase();
                        if ext_lower == "glb" || ext_lower == "gltf" {
                            files.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
            files.sort();
            state.batch_files.set(files);

            // Ask for output folder
            let dest_dialog = rfd::FileDialog::new()
                .set_title("Select Output Folder for Converted Files")
                .set_directory(&dir);

            let Some(dest_dir) = dest_dialog.pick_folder() else {
                return;
            };

            convert_batch_with_options(
                state,
                false,
                String::new(),
                dest_dir.to_string_lossy().to_string(),
            ); // to_glb is ignored for gltf->gr2
        }
    } else {
        // Single file mode
        let mut dialog = rfd::FileDialog::new()
            .set_title("Select glTF/GLB File")
            .add_filter("glTF Files", &["glb", "gltf"]);

        if let Some(dir) = state.working_dir.get() {
            dialog = dialog.set_directory(&dir);
        }

        if let Some(file) = dialog.pick_file() {
            if let Some(parent) = file.parent() {
                state
                    .working_dir
                    .set(Some(parent.to_string_lossy().to_string()));
            }
            state
                .input_file
                .set(Some(file.to_string_lossy().to_string()));
            convert_single_with_options(state, false, String::new());
        }
    }
}

/// Public function to open a GR2 file for conversion (CMD+O shortcut)
pub fn open_gr2_file(state: Gr2State, config: ConfigState) {
    select_and_convert_gr2(state, config, true, false);
}

/// Bundle options panel for texture extraction when converting GR2→GLB/glTF
fn bundle_options_panel(
    state: Gr2State,
    config: ConfigState,
    use_glb: RwSignal<bool>,
) -> impl IntoView {
    let extract_textures = state.extract_textures;
    let convert_png = state.convert_to_png;
    let keep_dds = state.keep_original_dds;
    let keep_gr2 = state.keep_original_gr2;
    let game_data_path = config.bg3_data_path;

    // Show warning only when texture options are enabled AND game data path is not set
    let needs_game_data_warning = move || {
        let textures_enabled = extract_textures.get();
        let path_missing = game_data_path.get().is_empty();
        textures_enabled && path_missing
    };

    v_stack((
        // Header - updates based on selected output format
        label(move || {
            if use_glb.get() {
                "Bundle Options (GR2 → GLB)".to_string()
            } else {
                "Bundle Options (GR2→glTF)".to_string()
            }
        })
        .style(|s| {
            s.font_size(13.0)
                .font_weight(Weight::SEMIBOLD)
                .color(Color::rgb8(80, 80, 80))
                .margin_bottom(8.0)
        }),
        // Checkboxes in a row
        h_stack((
            checkbox_option("Extract textures", extract_textures),
            checkbox_option("Convert textures DDS to PNG", convert_png),
            checkbox_option("Keep original textures DDS", keep_dds),
            checkbox_option("Keep original GR2", keep_gr2),
        ))
        .style(|s| s.gap(24.0).items_center()),
        // Warning only shown when textures enabled but path not configured
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
