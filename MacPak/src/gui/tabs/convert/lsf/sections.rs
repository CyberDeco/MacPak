//! UI sections for LSF/LSX/LSJ/LOCA conversion subtab

use floem::event::Event;
use floem::prelude::*;
use floem::text::Weight;

use super::conversion::{convert_batch, convert_single};
use crate::gui::shared::{card_style, drop_zone};
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
        lsf_drop_zone(state),
    ))
    .style(|s| s.width_full().gap(20.0).margin_bottom(20.0))
}

/// LSF/LSX/LSJ conversion group — source → target toggle layout
fn lsf_conversion_group(state: LsfConvertState) -> impl IntoView {
    let source = state.detected_format;
    let target = state.target_format;
    let state_for_select = state.clone();
    let state_for_convert = state.clone();

    v_stack((
        // Title
        label(|| "LSF / LSX / LSJ").style(|s| {
            s.font_size(13.0)
                .font_weight(Weight::SEMIBOLD)
                .color(Color::rgb8(80, 80, 80))
                .margin_bottom(8.0)
        }),
        // Source → Target format selector row
        h_stack((
            // Source format buttons
            h_stack((
                format_source_button("LSF", source, target),
                format_source_button("LSX", source, target),
                format_source_button("LSJ", source, target),
            ))
            .style(|s| s.gap(4.0).items_center()),
            // Arrow
            label(|| "→").style(|s| {
                s.font_size(16.0)
                    .font_weight(Weight::BOLD)
                    .color(Color::rgb8(120, 120, 120))
                    .padding_horiz(12.0)
            }),
            // Target format buttons
            h_stack((
                format_target_button("LSF", target, source),
                format_target_button("LSX", target, source),
                format_target_button("LSJ", target, source),
            ))
            .style(|s| s.gap(4.0).items_center()),
        ))
        .style(|s| {
            s.width_full()
                .items_center()
                .justify_center()
                .margin_bottom(12.0)
        }),
        // Select + convert single file
        lsf_dynamic_button(source, target, "🔄", "Convert", move || {
            select_and_convert_single_lsf(state_for_select.clone());
        }),
        // Batch convert
        lsf_dynamic_button(source, target, "📁", "Batch", move || {
            select_and_convert_batch_lsf(state_for_convert.clone());
        }),
    ))
    .style(|s| card_style(s).flex_grow(1.0).flex_basis(0.0).gap(8.0))
}

/// LOCA <-> XML conversion group — source → target toggle layout
fn loca_conversion_group(state: LsfConvertState) -> impl IntoView {
    let source = state.loca_source_format;
    let target = state.loca_target_format;
    let state_for_select = state.clone();
    let state_for_batch = state.clone();

    v_stack((
        // Title
        label(|| "LOCA / XML").style(|s| {
            s.font_size(13.0)
                .font_weight(Weight::SEMIBOLD)
                .color(Color::rgb8(80, 80, 80))
                .margin_bottom(8.0)
        }),
        // Source → Target format selector row
        h_stack((
            // Source format buttons
            h_stack((
                format_source_button("LOCA", source, target),
                format_source_button("XML", source, target),
            ))
            .style(|s| s.gap(4.0).items_center()),
            // Arrow
            label(|| "→").style(|s| {
                s.font_size(16.0)
                    .font_weight(Weight::BOLD)
                    .color(Color::rgb8(120, 120, 120))
                    .padding_horiz(12.0)
            }),
            // Target format buttons
            h_stack((
                format_target_button("LOCA", target, source),
                format_target_button("XML", target, source),
            ))
            .style(|s| s.gap(4.0).items_center()),
        ))
        .style(|s| {
            s.width_full()
                .items_center()
                .justify_center()
                .margin_bottom(12.0)
        }),
        // Convert single file
        lsf_dynamic_button(source, target, "🔄", "Convert", move || {
            select_and_convert_single_loca(state_for_select.clone());
        }),
        // Batch convert
        lsf_dynamic_button(source, target, "📁", "Batch", move || {
            select_and_convert_batch_loca(state_for_batch.clone());
        }),
    ))
    .style(|s| card_style(s).flex_grow(1.0).flex_basis(0.0).gap(8.0))
}

/// Drop zone for drag & drop conversion
fn lsf_drop_zone(state: LsfConvertState) -> impl IntoView {
    let state_for_drop = state.clone();

    drop_zone("📄", ".lsf, .lsx, .lsj, .loca, .xml", false, move |e| {
        if let Event::DroppedFile(drop_event) = e {
            let path = drop_event.path.to_string_lossy().to_string();
            let format = detect_format(&path);

            if format.is_empty() {
                state_for_drop.add_result(
                    "⚠ Only .lsf, .lsx, .lsj, .loca, or .xml files can be dropped here",
                );
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
}

/// Source format toggle button — selects the source format and auto-fixes target if needed
fn format_source_button(
    text: &'static str,
    source: RwSignal<String>,
    target: RwSignal<String>,
) -> impl IntoView {
    button(text)
        .action(move || {
            source.set(text.to_string());
            // If the target is now the same as source, pick the first valid alternative
            if target.get() == text {
                let targets = target_formats_for(text);
                if let Some(first) = targets.first() {
                    target.set(first.to_string());
                }
            }
        })
        .style(move |s| {
            let is_selected = source.get() == text;
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

/// Target format toggle button — grayed out/disabled when it matches the current source
fn format_target_button(
    text: &'static str,
    target: RwSignal<String>,
    source: RwSignal<String>,
) -> impl IntoView {
    button(text)
        .action(move || {
            // Only allow selection if not same as source
            if source.get() != text {
                target.set(text.to_string());
            }
        })
        .style(move |s| {
            let is_disabled = source.get() == text;
            let is_selected = target.get() == text;
            let s = s
                .padding_vert(4.0)
                .padding_horiz(12.0)
                .border_radius(4.0)
                .font_size(12.0);
            if is_disabled {
                // Grayed out — same as source format
                s.background(Color::rgb8(240, 240, 240))
                    .color(Color::rgb8(190, 190, 190))
                    .border(1.0)
                    .border_color(Color::rgb8(220, 220, 220))
                    .cursor(floem::style::CursorStyle::Default)
            } else if is_selected {
                s.background(Color::rgb8(66, 133, 244))
                    .color(Color::WHITE)
                    .border(1.0)
                    .border_color(Color::rgb8(66, 133, 244))
                    .cursor(floem::style::CursorStyle::Pointer)
            } else {
                s.background(Color::rgb8(245, 245, 245))
                    .color(Color::rgb8(100, 100, 100))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .cursor(floem::style::CursorStyle::Pointer)
                    .hover(|s| s.background(Color::rgb8(235, 235, 235)))
            }
        })
}

/// Dynamic button whose label reflects the current source → target selection
fn lsf_dynamic_button(
    source: RwSignal<String>,
    target: RwSignal<String>,
    emoji: &'static str,
    verb: &'static str,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    button(label(move || {
        format!("{} {} {} → {}", emoji, verb, source.get(), target.get())
    }))
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

// ─── File dialog functions ───────────────────────────────────────────────────

/// Select an LSF/LSX/LSJ file filtered by the chosen source format, then convert
fn select_and_convert_single_lsf(state: LsfConvertState) {
    let source = state.detected_format.get();
    let source_ext = source.to_lowercase();
    let title = format!("Select {} File", source);
    let filter_label = format!("{} Files", source);

    let mut dialog = rfd::FileDialog::new()
        .set_title(&title)
        .add_filter(&filter_label, &[&source_ext]);

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

    select_output_and_convert(state);
}

/// Select a LOCA/XML file filtered by the chosen source format, then convert
fn select_and_convert_single_loca(state: LsfConvertState) {
    let source = state.loca_source_format.get();
    let target = state.loca_target_format.get();
    let source_ext = source.to_lowercase();
    let title = format!("Select {} File", source);
    let filter_label = format!("{} Files", source);

    let mut dialog = rfd::FileDialog::new()
        .set_title(&title)
        .add_filter(&filter_label, &[&source_ext]);

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
    state.detected_format.set(source.clone());
    state.target_format.set(target);

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

/// Select input directory for LSF/LSX/LSJ batch using the toggle-selected source/target
fn select_and_convert_batch_lsf(state: LsfConvertState) {
    let source = state.detected_format.get();
    let target = state.target_format.get();

    state.batch_source_format.set(source);
    state.batch_target_format.set(target);

    select_and_convert_batch_common(state);
}

/// Select input directory for LOCA/XML batch using the toggle-selected source/target
fn select_and_convert_batch_loca(state: LsfConvertState) {
    let source = state.loca_source_format.get();
    let target = state.loca_target_format.get();

    state.batch_source_format.set(source);
    state.batch_target_format.set(target);

    select_and_convert_batch_common(state);
}

/// Common batch conversion: pick input dir, scan files, pick output dir, convert
fn select_and_convert_batch_common(state: LsfConvertState) {
    let source_ext = state.batch_source_format.get().to_lowercase();

    let mut dialog =
        rfd::FileDialog::new().set_title(&format!("Select Directory with .{} Files", source_ext));

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
