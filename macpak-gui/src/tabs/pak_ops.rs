//! PAK Operations Tab
//!
//! Extract and create PAK files with progress tracking.

use floem::prelude::*;
use floem::text::Weight;
use std::thread;

use crate::state::{AppState, PakOpsState};

pub fn pak_ops_tab(_app_state: AppState, pak_state: PakOpsState) -> impl IntoView {
    v_stack((
        // Extract section
        section_header("Extract PAK"),
        extract_section(pak_state.clone()),

        spacer(),

        // Create section
        section_header("Create PAK"),
        create_section(pak_state.clone()),

        spacer(),

        // List section
        section_header("List PAK Contents"),
        list_section(pak_state),

        empty().style(|s| s.flex_grow(1.0)),
    ))
    .style(|s| s.width_full().height_full().padding(20.0))
}

fn section_header(title: &'static str) -> impl IntoView {
    label(move || title.to_string()).style(|s| {
        s.font_size(18.0)
            .font_weight(Weight::BOLD)
            .margin_bottom(12.0)
    })
}

fn spacer() -> impl IntoView {
    empty().style(|s| s.height(30.0))
}

fn extract_section(state: PakOpsState) -> impl IntoView {
    let source = state.extract_source;
    let dest = state.extract_dest;
    let progress = state.extract_progress;
    let status = state.extract_status;
    let is_extracting = state.is_extracting;

    let state_browse_src = state.clone();
    let state_browse_dest = state.clone();
    let state_extract = state.clone();

    v_stack((
        // Source PAK
        h_stack((
            label(|| "Source PAK:").style(|s| s.width(120.0)),
            label(move || {
                source
                    .get()
                    .unwrap_or_else(|| "No file selected".to_string())
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(8.0)
                    .background(Color::rgb8(250, 250, 250))
                    .border(1.0)
                    .border_color(Color::rgb8(220, 220, 220))
                    .border_radius(4.0)
                    .text_ellipsis()
            }),
            button("Browse...").action(move || {
                browse_extract_source(state_browse_src.clone());
            }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),

        // Destination folder
        h_stack((
            label(|| "Extract to:").style(|s| s.width(120.0)),
            label(move || {
                dest.get()
                    .unwrap_or_else(|| "No folder selected".to_string())
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(8.0)
                    .background(Color::rgb8(250, 250, 250))
                    .border(1.0)
                    .border_color(Color::rgb8(220, 220, 220))
                    .border_radius(4.0)
                    .text_ellipsis()
            }),
            button("Browse...").action(move || {
                browse_extract_dest(state_browse_dest.clone());
            }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center().margin_top(8.0)),

        // Progress bar
        progress_bar(progress),

        // Status message
        label(move || status.get())
            .style(|s| s.color(Color::rgb8(76, 175, 80)).font_size(12.0).margin_top(4.0)),

        // Extract button
        h_stack((
            empty().style(|s| s.flex_grow(1.0)),
            dyn_container(
                move || is_extracting.get(),
                move |extracting| {
                    if extracting {
                        button("⏳ Extracting...")
                            .disabled(|| true)
                            .style(|s| {
                                s.padding_horiz(24.0)
                                    .padding_vert(8.0)
                                    .background(Color::rgb8(150, 150, 150))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                            })
                            .into_any()
                    } else {
                        let state = state_extract.clone();
                        button("🗜️ Extract PAK")
                            .style(|s| {
                                s.padding_horiz(24.0)
                                    .padding_vert(8.0)
                                    .background(Color::rgb8(33, 150, 243))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                                    .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                            })
                            .action(move || {
                                extract_pak(state.clone());
                            })
                            .into_any()
                    }
                },
            ),
        ))
        .style(|s| s.width_full().margin_top(12.0)),
    ))
    .style(|s| {
        s.width_full()
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn create_section(state: PakOpsState) -> impl IntoView {
    let source = state.create_source;
    let dest = state.create_dest;
    let progress = state.create_progress;
    let status = state.create_status;
    let is_creating = state.is_creating;

    let state_browse_src = state.clone();
    let state_browse_dest = state.clone();
    let state_create = state.clone();

    v_stack((
        // Source folder
        h_stack((
            label(|| "Source Folder:").style(|s| s.width(120.0)),
            label(move || {
                source
                    .get()
                    .unwrap_or_else(|| "No folder selected".to_string())
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(8.0)
                    .background(Color::rgb8(250, 250, 250))
                    .border(1.0)
                    .border_color(Color::rgb8(220, 220, 220))
                    .border_radius(4.0)
                    .text_ellipsis()
            }),
            button("Browse...").action(move || {
                browse_create_source(state_browse_src.clone());
            }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),

        // Output PAK
        h_stack((
            label(|| "Output PAK:").style(|s| s.width(120.0)),
            label(move || {
                dest.get()
                    .unwrap_or_else(|| "No file selected".to_string())
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(8.0)
                    .background(Color::rgb8(250, 250, 250))
                    .border(1.0)
                    .border_color(Color::rgb8(220, 220, 220))
                    .border_radius(4.0)
                    .text_ellipsis()
            }),
            button("Save As...").action(move || {
                browse_create_dest(state_browse_dest.clone());
            }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center().margin_top(8.0)),

        // Progress bar
        progress_bar(progress),

        // Status message
        label(move || status.get())
            .style(|s| s.color(Color::rgb8(76, 175, 80)).font_size(12.0).margin_top(4.0)),

        // Create button
        h_stack((
            empty().style(|s| s.flex_grow(1.0)),
            dyn_container(
                move || is_creating.get(),
                move |creating| {
                    if creating {
                        button("⏳ Creating...")
                            .disabled(|| true)
                            .style(|s| {
                                s.padding_horiz(24.0)
                                    .padding_vert(8.0)
                                    .background(Color::rgb8(150, 150, 150))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                            })
                            .into_any()
                    } else {
                        let state = state_create.clone();
                        button("📦 Create PAK")
                            .style(|s| {
                                s.padding_horiz(24.0)
                                    .padding_vert(8.0)
                                    .background(Color::rgb8(76, 175, 80))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                                    .hover(|s| s.background(Color::rgb8(56, 142, 60)))
                            })
                            .action(move || {
                                create_pak(state.clone());
                            })
                            .into_any()
                    }
                },
            ),
        ))
        .style(|s| s.width_full().margin_top(12.0)),
    ))
    .style(|s| {
        s.width_full()
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn list_section(state: PakOpsState) -> impl IntoView {
    let source = state.list_source;
    let contents = state.list_contents;
    let is_listing = state.is_listing;

    let state_browse = state.clone();
    let state_list = state.clone();

    v_stack((
        // Source PAK
        h_stack((
            label(|| "PAK File:").style(|s| s.width(120.0)),
            label(move || {
                source
                    .get()
                    .unwrap_or_else(|| "No file selected".to_string())
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(8.0)
                    .background(Color::rgb8(250, 250, 250))
                    .border(1.0)
                    .border_color(Color::rgb8(220, 220, 220))
                    .border_radius(4.0)
                    .text_ellipsis()
            }),
            button("Browse...").action(move || {
                browse_list_source(state_browse.clone());
            }),
            dyn_container(
                move || is_listing.get(),
                move |listing| {
                    if listing {
                        button("⏳ Loading...")
                            .disabled(|| true)
                            .into_any()
                    } else {
                        let state = state_list.clone();
                        button("📋 List Contents")
                            .action(move || {
                                list_pak(state.clone());
                            })
                            .into_any()
                    }
                },
            ),
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),

        // Contents list
        scroll(
            dyn_stack(
                move || contents.get(),
                |item| item.clone(),
                |item| {
                    label(move || item.clone()).style(|s| {
                        s.width_full()
                            .padding(4.0)
                            .font_size(12.0)
                            .font_family("monospace".to_string())
                            .border_bottom(1.0)
                            .border_color(Color::rgb8(245, 245, 245))
                    })
                },
            )
            .style(|s| s.width_full()),
        )
        .style(|s| {
            s.width_full()
                .height(150.0)
                .margin_top(8.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),

        // File count
        label(move || {
            let count = contents.get().len();
            if count > 0 {
                format!("{} files in archive", count)
            } else {
                String::new()
            }
        })
        .style(|s| s.color(Color::rgb8(100, 100, 100)).font_size(12.0).margin_top(4.0)),
    ))
    .style(|s| {
        s.width_full()
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(8.0)
    })
}

fn progress_bar(progress: RwSignal<f32>) -> impl IntoView {
    h_stack((
        // Progress track
        container(
            // Progress fill
            empty().style(move |s| {
                let pct = progress.get();
                s.width_pct(pct as f64 * 100.0)
                    .height_full()
                    .background(Color::rgb8(33, 150, 243))
                    .border_radius(4.0)
            }),
        )
        .style(|s| {
            s.flex_grow(1.0)
                .height(8.0)
                .background(Color::rgb8(230, 230, 230))
                .border_radius(4.0)
        }),

        // Percentage label
        label(move || format!("{:.0}%", progress.get() * 100.0))
            .style(|s| s.width(50.0).font_size(12.0)),
    ))
    .style(|s| s.width_full().gap(8.0).items_center().margin_top(12.0))
}

// ============================================================================
// File Operations
// ============================================================================

fn browse_extract_source(state: PakOpsState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Select PAK File to Extract")
        .add_filter("PAK Files", &["pak"]);

    if let Some(path) = dialog.pick_file() {
        state.extract_source.set(Some(path.to_string_lossy().to_string()));
    }
}

fn browse_extract_dest(state: PakOpsState) {
    let dialog = rfd::FileDialog::new().set_title("Select Extraction Destination");

    if let Some(path) = dialog.pick_folder() {
        state.extract_dest.set(Some(path.to_string_lossy().to_string()));
    }
}

fn extract_pak(state: PakOpsState) {
    let source = match state.extract_source.get() {
        Some(s) => s,
        None => {
            state.extract_status.set("Please select a PAK file".to_string());
            return;
        }
    };

    let dest = match state.extract_dest.get() {
        Some(d) => d,
        None => {
            state.extract_status.set("Please select a destination folder".to_string());
            return;
        }
    };

    state.is_extracting.set(true);
    state.extract_status.set("Extracting...".to_string());
    state.extract_progress.set(0.0);

    // Run extraction in background thread
    thread::spawn(move || {
        let result = MacLarian::pak::PakOperations::extract(&source, &dest);

        // Update UI (note: in Floem we can update signals from any thread)
        match result {
            Ok(_) => {
                state.extract_progress.set(1.0);
                state.extract_status.set("Extraction complete!".to_string());
            }
            Err(e) => {
                state.extract_status.set(format!("Extraction failed: {}", e));
            }
        }
        state.is_extracting.set(false);
    });
}

fn browse_create_source(state: PakOpsState) {
    let dialog = rfd::FileDialog::new().set_title("Select Mod Directory");

    if let Some(path) = dialog.pick_folder() {
        state.create_source.set(Some(path.to_string_lossy().to_string()));
    }
}

fn browse_create_dest(state: PakOpsState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Save PAK File As")
        .add_filter("PAK Files", &["pak"]);

    if let Some(path) = dialog.save_file() {
        state.create_dest.set(Some(path.to_string_lossy().to_string()));
    }
}

fn create_pak(state: PakOpsState) {
    let source = match state.create_source.get() {
        Some(s) => s,
        None => {
            state.create_status.set("Please select a source folder".to_string());
            return;
        }
    };

    let dest = match state.create_dest.get() {
        Some(d) => d,
        None => {
            state.create_status.set("Please select an output file".to_string());
            return;
        }
    };

    state.is_creating.set(true);
    state.create_status.set("Creating PAK...".to_string());
    state.create_progress.set(0.0);

    thread::spawn(move || {
        let result = MacLarian::pak::PakOperations::create(&source, &dest);

        match result {
            Ok(_) => {
                state.create_progress.set(1.0);
                state.create_status.set("PAK created successfully!".to_string());
            }
            Err(e) => {
                state.create_status.set(format!("Creation failed: {}", e));
            }
        }
        state.is_creating.set(false);
    });
}

fn browse_list_source(state: PakOpsState) {
    let dialog = rfd::FileDialog::new()
        .set_title("Select PAK File")
        .add_filter("PAK Files", &["pak"]);

    if let Some(path) = dialog.pick_file() {
        state.list_source.set(Some(path.to_string_lossy().to_string()));
        state.list_contents.set(Vec::new());
    }
}

fn list_pak(state: PakOpsState) {
    let source = match state.list_source.get() {
        Some(s) => s,
        None => return,
    };

    state.is_listing.set(true);

    thread::spawn(move || {
        let result = MacLarian::pak::PakOperations::list(&source);

        match result {
            Ok(files) => {
                state.list_contents.set(files);
            }
            Err(e) => {
                state.list_contents.set(vec![format!("Error: {}", e)]);
            }
        }
        state.is_listing.set(false);
    });
}
