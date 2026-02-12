//! UI sections for LSF/LSX/LSJ/LOCA conversion subtab

use floem::AnyView;
use floem::event::{Event, EventListener};
use floem::prelude::*;
use floem::text::Weight;

use super::conversion::{convert_batch, convert_single};
use crate::gui::shared::operation_button;
use crate::gui::state::LsfConvertState;

/// Detect format from file extension
fn detect_format(path: &str) -> String {
    let path_lower = path.to_lowercase();
    if path_lower.ends_with(".lsf") {
        "LSF".to_string()
    } else if path_lower.ends_with(".lsx") {
        "LSX".to_string()
    } else if path_lower.ends_with(".lsj") {
        "LSJ".to_string()
    } else if path_lower.ends_with(".loca") {
        "LOCA".to_string()
    } else if path_lower.ends_with(".xml") {
        "XML".to_string()
    } else {
        String::new()
    }
}

/// Get valid target formats for a detected source format.
/// LOCA only converts to XML and vice versa — never to LSF/LSX/LSJ.
fn target_formats_for(source: &str) -> Vec<&'static str> {
    match source {
        "LSF" => vec!["LSX", "LSJ"],
        "LSX" => vec!["LSF", "LSJ"],
        "LSJ" => vec!["LSX", "LSF"],
        "LOCA" => vec!["XML"],
        "XML" => vec!["LOCA"],
        _ => vec![],
    }
}

/// Main operations row with 3 columns
pub fn operations_row(state: LsfConvertState) -> impl IntoView {
    h_stack((
        // LSF/LSX/LSJ conversion group
        lsf_conversion_group(state.clone()),
        // LOCA/XML conversion group
        loca_conversion_group(state.clone()),
        // Drop zone
        drop_zone(state),
    ))
    .style(|s| s.width_full().gap(20.0).margin_bottom(20.0))
}

/// LSF/LSX/LSJ conversion group — single + batch
fn lsf_conversion_group(state: LsfConvertState) -> impl IntoView {
    let detected = state.detected_format;
    let target = state.target_format;
    let state_for_select = state.clone();
    let state_for_convert = state.clone();

    v_stack((
        // Header row with title and target format toggle
        h_stack((
            label(|| "LSF / LSX / LSJ").style(|s| {
                s.font_size(13.0)
                    .font_weight(Weight::SEMIBOLD)
                    .color(Color::rgb8(80, 80, 80))
            }),
            empty().style(|s| s.flex_grow(1.0)),
            // Target format buttons - shown when a format is detected
            dyn_container(
                move || detected.get(),
                move |fmt| {
                    let targets = target_formats_for(&fmt);
                    // Only show toggles for LSF/LSX/LSJ sources
                    if targets.is_empty() || fmt == "LOCA" || fmt == "XML" {
                        empty().into_any()
                    } else {
                        let mut views: Vec<AnyView> = Vec::new();
                        for t in targets {
                            views.push(format_select_button(t, target).into_any());
                        }
                        h_stack_from_iter(views)
                            .style(|s| s.gap(4.0).items_center())
                            .into_any()
                    }
                },
            ),
        ))
        .style(|s| s.width_full().gap(4.0).items_center().margin_bottom(8.0)),
        // Select + convert single file
        operation_button("🔄 Convert File", move || {
            select_and_convert_single_lsf(state_for_select.clone());
        }),
        // Batch convert
        operation_button("📁 Batch Convert Directory", move || {
            select_and_convert_batch_lsf(state_for_convert.clone());
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

/// LOCA <-> XML conversion group — single + batch
fn loca_conversion_group(state: LsfConvertState) -> impl IntoView {
    let state_for_loca = state.clone();
    let state_for_xml = state.clone();
    let state_for_batch_loca = state.clone();
    let state_for_batch_xml = state.clone();

    v_stack((
        label(|| "LOCA / XML").style(|s| {
            s.font_size(13.0)
                .font_weight(Weight::SEMIBOLD)
                .color(Color::rgb8(80, 80, 80))
                .margin_bottom(14.0)
                .margin_top(4.0)
        }),
        // LOCA -> XML
        operation_button("🔄 Convert LOCA → XML", move || {
            select_and_convert_single_fixed(state_for_loca.clone(), "LOCA", "XML", &["loca"]);
        }),
        // XML -> LOCA
        operation_button("🔄 Convert XML → LOCA", move || {
            select_and_convert_single_fixed(state_for_xml.clone(), "XML", "LOCA", &["xml"]);
        }),
        // Batch LOCA -> XML
        operation_button("📁 Batch LOCA → XML", move || {
            select_and_convert_batch_fixed(state_for_batch_loca.clone(), "LOCA", "XML");
        }),
        // Batch XML -> LOCA
        operation_button("📁 Batch XML → LOCA", move || {
            select_and_convert_batch_fixed(state_for_batch_xml.clone(), "XML", "LOCA");
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

/// Drop zone for drag & drop conversion
fn drop_zone(state: LsfConvertState) -> impl IntoView {
    let state_for_drop = state.clone();

    container(
        v_stack((
            label(|| "📄".to_string()).style(|s| s.font_size(32.0)),
            label(|| "Drag files here".to_string()).style(|s| {
                s.font_size(14.0)
                    .color(Color::rgb8(100, 100, 100))
                    .margin_top(8.0)
            }),
            label(|| ".lsf, .lsx, .lsj, .loca, .xml".to_string())
                .style(|s| s.font_size(12.0).color(Color::rgb8(150, 150, 150))),
        ))
        .style(|s| s.items_center()),
    )
    .on_event_cont(EventListener::DroppedFile, move |e| {
        if let Event::DroppedFile(drop_event) = e {
            let path = drop_event.path.to_string_lossy().to_string();
            let format = detect_format(&path);

            if format.is_empty() {
                state_for_drop
                    .add_result("⚠ Only .lsf, .lsx, .lsj, .loca, or .xml files can be dropped here");
                return;
            }

            // Auto-detect format and set up for conversion
            state_for_drop.input_file.set(Some(path));
            state_for_drop.detected_format.set(format.clone());

            // Auto-select first valid target
            let targets = target_formats_for(&format);
            if let Some(first) = targets.first() {
                state_for_drop.target_format.set(first.to_string());
                select_output_and_convert(state_for_drop.clone());
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

/// Toggle button for format selection (matches GR2 format_toggle_button style)
fn format_select_button(text: &'static str, signal: RwSignal<String>) -> impl IntoView {
    button(text)
        .action(move || signal.set(text.to_string()))
        .style(move |s| {
            let is_selected = signal.get() == text;
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

// ─── File dialog functions ───────────────────────────────────────────────────

/// Select an LSF/LSX/LSJ file, pick target format, then pick output folder and convert
fn select_and_convert_single_lsf(state: LsfConvertState) {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Select LSF/LSX/LSJ File")
        .add_filter("LSF/LSX/LSJ Files", &["lsf", "lsx", "lsj"]);

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    let Some(file) = dialog.pick_file() else {
        return;
    };

    if let Some(parent) = file.parent() {
        state
            .working_dir
            .set(Some(parent.to_string_lossy().to_string()));
    }

    let path_str = file.to_string_lossy().to_string();
    let format = detect_format(&path_str);

    state.input_file.set(Some(path_str));
    state.detected_format.set(format.clone());

    // Use the currently selected target, or auto-select first valid target
    let targets = target_formats_for(&format);
    let current_target = state.target_format.get();
    if !targets.contains(&current_target.as_str()) {
        if let Some(first) = targets.first() {
            state.target_format.set(first.to_string());
        }
    }

    select_output_and_convert(state);
}

/// Select a file with a fixed source/target format, then pick output folder and convert
fn select_and_convert_single_fixed(
    state: LsfConvertState,
    source_format: &str,
    target_format: &str,
    extensions: &[&str],
) {
    let title = format!("Select {} File", source_format);
    let filter_label = format!("{} Files", source_format);
    let mut dialog = rfd::FileDialog::new()
        .set_title(&title)
        .add_filter(&filter_label, extensions);

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    let Some(file) = dialog.pick_file() else {
        return;
    };

    if let Some(parent) = file.parent() {
        state
            .working_dir
            .set(Some(parent.to_string_lossy().to_string()));
    }

    state
        .input_file
        .set(Some(file.to_string_lossy().to_string()));
    state.detected_format.set(source_format.to_string());
    state.target_format.set(target_format.to_string());

    select_output_and_convert(state);
}

/// Prompt for output folder, then convert the currently selected single file
fn select_output_and_convert(state: LsfConvertState) {
    if state.input_file.get().is_none() {
        state
            .status_message
            .set("No input file selected".to_string());
        return;
    }

    let mut dialog = rfd::FileDialog::new().set_title("Select Output Folder for Converted File");

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    let Some(output_dir) = dialog.pick_folder() else {
        return;
    };

    convert_single(state, output_dir.to_string_lossy().to_string());
}

/// Select input directory for LSF/LSX/LSJ batch, then output directory, then convert
fn select_and_convert_batch_lsf(state: LsfConvertState) {
    // Use the currently selected target from the format toggle in the header
    let target = state.target_format.get();
    let detected = state.detected_format.get();

    // Determine source format from detected format, or default to LSF
    let source_format = if !detected.is_empty()
        && detected != "LOCA"
        && detected != "XML"
    {
        detected.clone()
    } else {
        "LSF".to_string()
    };

    // Determine target format
    let target_format = if !target.is_empty() {
        target
    } else {
        let targets = target_formats_for(&source_format);
        targets.first().unwrap_or(&"LSX").to_string()
    };

    state.batch_source_format.set(source_format);
    state.batch_target_format.set(target_format);

    select_and_convert_batch_common(state);
}

/// Select input directory for LOCA/XML batch with fixed formats, then output directory
fn select_and_convert_batch_fixed(
    state: LsfConvertState,
    source_format: &str,
    target_format: &str,
) {
    state.batch_source_format.set(source_format.to_string());
    state.batch_target_format.set(target_format.to_string());

    select_and_convert_batch_common(state);
}

/// Common batch conversion: pick input dir, scan files, pick output dir, convert
fn select_and_convert_batch_common(state: LsfConvertState) {
    let source_ext = state.batch_source_format.get().to_lowercase();

    let mut dialog = rfd::FileDialog::new().set_title(&format!(
        "Select Directory with .{} Files",
        source_ext
    ));

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    let Some(input_dir) = dialog.pick_folder() else {
        return;
    };

    state
        .working_dir
        .set(Some(input_dir.to_string_lossy().to_string()));
    state
        .batch_input_dir
        .set(Some(input_dir.to_string_lossy().to_string()));

    // Scan for files matching the source format
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(&input_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == source_ext {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    files.sort();

    if files.is_empty() {
        state.add_result(&format!(
            "No .{} files found in {}",
            source_ext,
            input_dir.display()
        ));
        state.status_message.set(format!(
            "No .{} files found in selected directory",
            source_ext
        ));
        return;
    }

    state.add_result(&format!("Found {} .{} files", files.len(), source_ext));
    state.batch_files.set(files);

    // Ask for output folder
    let dest_dialog = rfd::FileDialog::new()
        .set_title("Select Output Folder for Converted Files")
        .set_directory(&input_dir);

    let Some(dest_dir) = dest_dialog.pick_folder() else {
        return;
    };

    convert_batch(state, dest_dir.to_string_lossy().to_string());
}

/// Public function to open a file for LSF conversion (CMD+O shortcut)
pub fn open_lsf_file(state: LsfConvertState) {
    select_and_convert_single_lsf(state);
}
