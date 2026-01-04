//! UI sections for GR2 conversion tab

use floem::event::{Event, EventListener};
use floem::prelude::*;
use floem::text::Weight;

use crate::gui::shared::operation_button;
use crate::gui::state::Gr2State;
use super::conversion::{convert_batch_with_options, convert_single_with_options};

/// Main operations row with 3 columns
pub fn operations_row(state: Gr2State) -> impl IntoView {
    h_stack((
        // GR2 -> glTF operations
        gr2_to_gltf_group(state.clone()),
        // glTF -> GR2 operations
        gltf_to_gr2_group(state.clone()),
        // Drop zone
        drop_zone(state),
    ))
    .style(|s| s.width_full().gap(20.0).margin_bottom(20.0))
}

fn gr2_to_gltf_group(state: Gr2State) -> impl IntoView {
    let state1 = state.clone();
    let state2 = state.clone();

    // Toggle state: true = GLB, false = glTF
    let use_glb = RwSignal::new(true);

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
            |glb| if glb { "🔄 Convert GR2 → GLB" } else { "🔄 Convert GR2 → glTF" },
            move || {
                let to_glb = use_glb.get_untracked();
                select_and_convert_gr2(state1.clone(), to_glb, false);
            },
        ),
        // Batch convert
        dynamic_operation_button(
            use_glb,
            |glb| if glb { "📁 Batch GR2 → GLB" } else { "📁 Batch GR2 → glTF" },
            move || {
                let to_glb = use_glb.get_untracked();
                select_and_convert_gr2(state2.clone(), to_glb, true);
            },
        ),
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

fn drop_zone(state: Gr2State) -> impl IntoView {
    let state_for_drop = state.clone();

    container(
        v_stack((
            label(|| "🦴".to_string()).style(|s| s.font_size(32.0)),
            label(|| "Drag files here".to_string()).style(|s| {
                s.font_size(14.0)
                    .color(Color::rgb8(100, 100, 100))
                    .margin_top(8.0)
            }),
            label(|| ".gr2, .glb, .gltf".to_string())
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

            if path_lower.ends_with(".gr2") {
                // GR2 file dropped - convert to GLB by default
                state_for_drop.add_result(&format!("Converting: {}", file_name));
                state_for_drop.input_file.set(Some(path));
                convert_single_with_options(state_for_drop.clone(), true); // GLB output
            } else if path_lower.ends_with(".glb") || path_lower.ends_with(".gltf") {
                // glTF file dropped - convert to GR2
                state_for_drop.add_result(&format!("Converting: {}", file_name));
                state_for_drop.input_file.set(Some(path));
                convert_single_with_options(state_for_drop.clone(), false); // GR2 output (glb param ignored)
            } else {
                state_for_drop.add_result("⚠ Only .gr2, .glb, or .gltf files can be dropped here");
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

/// Toggle button for format selection
fn format_toggle_button(
    text: &'static str,
    value: bool,
    signal: RwSignal<bool>,
) -> impl IntoView {
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
fn select_and_convert_gr2(state: Gr2State, to_glb: bool, batch: bool) {
    if batch {
        // Batch mode - select directory
        let mut dialog = rfd::FileDialog::new()
            .set_title("Select Directory with GR2 Files");

        if let Some(dir) = state.working_dir.get() {
            dialog = dialog.set_directory(&dir);
        }

        if let Some(dir) = dialog.pick_folder() {
            state.working_dir.set(Some(dir.to_string_lossy().to_string()));
            state.batch_input_dir.set(Some(dir.to_string_lossy().to_string()));

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

            convert_batch_with_options(state, to_glb);
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
                state.working_dir.set(Some(parent.to_string_lossy().to_string()));
            }
            state.input_file.set(Some(file.to_string_lossy().to_string()));
            convert_single_with_options(state, to_glb);
        }
    }
}

/// Select a glTF/GLB file and convert it to GR2
fn select_and_convert_gltf(state: Gr2State, batch: bool) {
    if batch {
        // Batch mode - select directory
        let mut dialog = rfd::FileDialog::new()
            .set_title("Select Directory with glTF/GLB Files");

        if let Some(dir) = state.working_dir.get() {
            dialog = dialog.set_directory(&dir);
        }

        if let Some(dir) = dialog.pick_folder() {
            state.working_dir.set(Some(dir.to_string_lossy().to_string()));
            state.batch_input_dir.set(Some(dir.to_string_lossy().to_string()));

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

            convert_batch_with_options(state, false); // to_glb is ignored for gltf->gr2
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
                state.working_dir.set(Some(parent.to_string_lossy().to_string()));
            }
            state.input_file.set(Some(file.to_string_lossy().to_string()));
            convert_single_with_options(state, false);
        }
    }
}
