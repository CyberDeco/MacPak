//! Dialog overlays for PAK operations

use floem::action::exec_after;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem::text::Weight;
use floem::views::{text_input, virtual_list, VirtualDirection, VirtualItemSize};
use floem_reactive::create_effect;
use im::Vector as ImVector;
use std::path::Path;
use std::time::Duration;

use crate::gui::state::PakOpsState;
use super::operations::{
    create_pak_from_dropped_folder, execute_create_pak, execute_individual_extract,
    extract_dropped_file, list_dropped_file, rebuild_pak_from_dropped_folder,
    validate_dropped_folder,
};
use super::types::get_shared_progress;
use super::widgets::{compression_selector, priority_input};

/// Progress overlay shown during long-running operations
/// Uses a polling timer to read shared atomic state updated by background threads
pub fn progress_overlay(state: PakOpsState) -> impl IntoView {
    let show = state.show_progress;

    // Local signals for polled values - updated by timer
    let polled_pct = RwSignal::new(0u32);
    let polled_current = RwSignal::new(0u32);
    let polled_total = RwSignal::new(0u32);
    let polled_msg = RwSignal::new(String::new());
    let timer_active = RwSignal::new(false);

    // Polling function
    fn poll_and_schedule(
        polled_pct: RwSignal<u32>,
        polled_current: RwSignal<u32>,
        polled_total: RwSignal<u32>,
        polled_msg: RwSignal<String>,
        show: RwSignal<bool>,
        timer_active: RwSignal<bool>,
    ) {
        // Read from shared atomic state
        let shared = get_shared_progress();
        let pct = shared.get_pct();
        let (current, total) = shared.get_counts();
        let msg = shared.get_message();

        // Update local signals
        polled_pct.set(pct);
        polled_current.set(current);
        polled_total.set(total);
        if !msg.is_empty() {
            polled_msg.set(msg);
        }

        // Schedule next poll if still active
        if show.get_untracked() && timer_active.get_untracked() {
            exec_after(Duration::from_millis(50), move |_| {
                if show.get_untracked() && timer_active.get_untracked() {
                    poll_and_schedule(polled_pct, polled_current, polled_total, polled_msg, show, timer_active);
                }
            });
        }
    }

    // Start/stop polling based on visibility
    create_effect(move |_| {
        let visible = show.get();
        if visible {
            // Reset and start polling
            get_shared_progress().reset();
            polled_pct.set(0);
            polled_current.set(0);
            polled_total.set(0);
            polled_msg.set("Starting...".to_string());
            timer_active.set(true);

            exec_after(Duration::from_millis(50), move |_| {
                if show.get_untracked() {
                    poll_and_schedule(polled_pct, polled_current, polled_total, polled_msg, show, timer_active);
                }
            });
        } else {
            timer_active.set(false);
        }
    });

    dyn_container(
        move || show.get(),
        move |is_visible| {
            if is_visible {
                container(
                    v_stack((
                        // Count display (e.g., "1/5")
                        label(move || {
                            let total = polled_total.get();
                            let current = polled_current.get();
                            if total > 0 {
                                format!("{}/{}", current, total)
                            } else {
                                String::new()
                            }
                        })
                        .style(|s| {
                            s.font_size(13.0)
                                .color(Color::rgb8(100, 100, 100))
                                .margin_bottom(4.0)
                        }),
                        // Filename
                        label(move || polled_msg.get())
                            .style(|s| s.font_size(14.0).margin_bottom(12.0)),
                        // Progress bar - full width
                        container(
                            container(empty())
                                .style(move |s| {
                                    let pct = polled_pct.get();
                                    s.height_full()
                                        .width_pct(pct as f64)
                                        .background(Color::rgb8(76, 175, 80))
                                        .border_radius(4.0)
                                }),
                        )
                        .style(|s| {
                            s.width_full()
                                .height(8.0)
                                .background(Color::rgb8(220, 220, 220))
                                .border_radius(4.0)
                        }),
                        label(move || format!("{}%", polled_pct.get()))
                            .style(|s| s.font_size(12.0).margin_top(8.0).color(Color::rgb8(100, 100, 100))),
                    ))
                    .style(|s| {
                        s.padding(24.0)
                            .background(Color::WHITE)
                            .border(1.0)
                            .border_color(Color::rgb8(200, 200, 200))
                            .border_radius(8.0)
                            .width(500.0)
                    }),
                )
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
}

/// Dialog for PAK creation options (compression, priority, info.json)
pub fn create_options_dialog(state: PakOpsState) -> impl IntoView {
    let show = state.show_create_options;
    let compression = state.compression;
    let priority = state.priority;
    let generate_info_json = state.generate_info_json;
    let pending = state.pending_create;
    let state_confirm = state.clone();
    let state_cancel = state.clone();

    dyn_container(
        move || show.get(),
        move |visible| {
            if visible {
                let compression = compression;
                let priority = priority;
                let generate_info_json = generate_info_json;
                let state_confirm = state_confirm.clone();
                let state_cancel = state_cancel.clone();

                v_stack((
                    // Title
                    label(|| "PAK Creation Options".to_string()).style(|s| {
                        s.font_size(18.0)
                            .font_weight(Weight::BOLD)
                            .margin_bottom(16.0)
                    }),
                    // Compression selector
                    h_stack((
                        label(|| "Compression:".to_string()).style(|s| s.width(120.0)),
                        compression_selector(compression),
                    ))
                    .style(|s| s.width_full().items_center().margin_bottom(12.0)),
                    // Priority input
                    h_stack((
                        label(|| "Load Priority:".to_string()).style(|s| s.width(120.0)),
                        priority_input(priority),
                    ))
                    .style(|s| s.width_full().items_center().margin_bottom(12.0)),
                    // Generate info.json checkbox
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
                    // Help text
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
                    // Buttons
                    h_stack((
                        empty().style(|s| s.flex_grow(1.0)),
                        button("Cancel")
                            .action(move || {
                                state_cancel.show_create_options.set(false);
                                state_cancel.pending_create.set(None);
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
                                    state_confirm.show_create_options.set(false);
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
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
    .on_event_stop(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(key_event) = e {
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                state.show_create_options.set(false);
                state.pending_create.set(None);
            }
        }
    })
    .keyboard_navigable()
}

/// Dialog shown when a file is dropped, asking what action to take
pub fn drop_action_dialog(state: PakOpsState) -> impl IntoView {
    let show = state.show_drop_dialog;
    let dropped_file = state.dropped_file;

    let state_extract = state.clone();
    let state_list = state.clone();
    let state_cancel = state.clone();

    dyn_container(
        move || show.get(),
        move |visible| {
            if visible {
                let file_path = dropped_file.get().unwrap_or_default();
                let file_name = Path::new(&file_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "PAK file".to_string());

                let state_extract = state_extract.clone();
                let state_list = state_list.clone();
                let state_cancel = state_cancel.clone();
                let file_path_extract = file_path.clone();
                let file_path_list = file_path.clone();

                v_stack((
                    // Title
                    label(move || format!("Dropped: {}", file_name)).style(|s| {
                        s.font_size(16.0)
                            .font_weight(Weight::BOLD)
                            .margin_bottom(16.0)
                    }),
                    // Action buttons
                    label(|| "What would you like to do?".to_string())
                        .style(|s| s.margin_bottom(12.0)),
                    // Extract button
                    button("📦 Extract PAK")
                        .action(move || {
                            state_extract.show_drop_dialog.set(false);
                            extract_dropped_file(state_extract.clone(), file_path_extract.clone());
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .margin_bottom(8.0)
                                .background(Color::rgb8(33, 150, 243))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                        }),
                    // List button
                    button("📋 List Contents")
                        .action(move || {
                            state_list.show_drop_dialog.set(false);
                            list_dropped_file(state_list.clone(), file_path_list.clone());
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .margin_bottom(8.0)
                                .background(Color::rgb8(76, 175, 80))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(56, 142, 60)))
                        }),
                    // Cancel button
                    button("Cancel")
                        .action(move || {
                            state_cancel.show_drop_dialog.set(false);
                            state_cancel.dropped_file.set(None);
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .background(Color::rgb8(240, 240, 240))
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0)
                        }),
                ))
                .style(|s| {
                    s.padding(24.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .width(320.0)
                })
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
    .on_event_stop(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(key_event) = e {
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                state.show_drop_dialog.set(false);
                state.dropped_file.set(None);
            }
        }
    })
    .keyboard_navigable()
}

/// Dialog for selecting individual files to extract from a PAK
pub fn file_select_dialog(state: PakOpsState) -> impl IntoView {
    let show = state.show_file_select;
    let files = state.file_select_list;
    let selected = state.file_select_selected;
    let pak_path = state.file_select_pak;

    // Local filter signal
    let ext_filter = RwSignal::new(String::new());

    let state_extract = state.clone();
    let state_cancel = state.clone();
    let state_select_all = state.clone();
    let state_deselect = state.clone();
    let state_escape = state.clone();

    // Filtered file list based on extension
    let filtered_files = move || {
        let filter = ext_filter.get().to_lowercase();
        let all_files = files.get();
        if filter.is_empty() {
            all_files.into_iter().collect::<ImVector<_>>()
        } else {
            all_files
                .into_iter()
                .filter(|f| f.to_lowercase().contains(&filter))
                .collect::<ImVector<_>>()
        }
    };

    dyn_container(
        move || show.get(),
        move |visible| {
            if visible {
                let state_extract = state_extract.clone();
                let state_cancel = state_cancel.clone();
                let state_select_all = state_select_all.clone();
                let state_deselect = state_deselect.clone();

                let pak_name = pak_path.get()
                    .map(|p| Path::new(&p).file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "PAK".to_string()))
                    .unwrap_or_else(|| "PAK".to_string());

                v_stack((
                    // Title
                    label(move || format!("Select files from {}", pak_name)).style(|s| {
                        s.font_size(16.0)
                            .font_weight(Weight::BOLD)
                            .margin_bottom(8.0)
                    }),
                    // Selection info and filter
                    h_stack((
                        label(move || {
                            let sel_count = selected.get().len();
                            let filtered = filtered_files().len();
                            let total = files.get().len();
                            if filtered == total {
                                format!("{} of {} files selected", sel_count, total)
                            } else {
                                format!("{} selected, {} of {} shown", sel_count, filtered, total)
                            }
                        })
                        .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                        empty().style(|s| s.flex_grow(1.0)),
                        text_input(ext_filter)
                            .placeholder("Filter (e.g. .lsf, .xml)")
                            .style(|s| {
                                s.width(160.0)
                                    .height(26.0)
                                    .padding_horiz(8.0)
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                                    .font_size(12.0)
                                    .background(Color::WHITE)
                            }),
                    ))
                    .style(|s| s.width_full().margin_bottom(12.0).items_center()),
                    // Select/Deselect buttons
                    h_stack((
                        button("Select All Visible")
                            .action(move || {
                                let visible_files: std::collections::HashSet<String> =
                                    filtered_files().into_iter().collect();
                                state_select_all.file_select_selected.update(|set| {
                                    for f in visible_files {
                                        set.insert(f);
                                    }
                                });
                            })
                            .style(|s| {
                                s.padding_vert(6.0)
                                    .padding_horiz(12.0)
                                    .font_size(12.0)
                                    .background(Color::rgb8(240, 240, 240))
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                            }),
                        button("Deselect All")
                            .action(move || {
                                state_deselect.file_select_selected.set(std::collections::HashSet::new());
                            })
                            .style(|s| {
                                s.padding_vert(6.0)
                                    .padding_horiz(12.0)
                                    .font_size(12.0)
                                    .margin_left(8.0)
                                    .background(Color::rgb8(240, 240, 240))
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                            }),
                    ))
                    .style(|s| s.margin_bottom(12.0)),
                    // File list with checkboxes using virtual_list
                    scroll(
                        virtual_list(
                            VirtualDirection::Vertical,
                            VirtualItemSize::Fixed(Box::new(|| 28.0)),
                            filtered_files,
                            |file_path: &String| file_path.clone(),
                            move |file_path| {
                                let file_path_clone = file_path.clone();
                                let file_path_for_check = file_path.clone();
                                let file_path_for_update = file_path.clone();
                                let file_path_for_label = file_path.clone();

                                h_stack((
                                    checkbox(move || selected.get().contains(&file_path_for_check))
                                        .on_update(move |checked| {
                                            let fp = file_path_for_update.clone();
                                            selected.update(|set| {
                                                if checked {
                                                    set.insert(fp);
                                                } else {
                                                    set.remove(&fp);
                                                }
                                            });
                                        })
                                        .style(|s| s.margin_right(8.0)),
                                    label(move || file_path_clone.clone())
                                        .on_click_stop(move |_| {
                                            let fp = file_path_for_label.clone();
                                            selected.update(|set| {
                                                if set.contains(&fp) {
                                                    set.remove(&fp);
                                                } else {
                                                    set.insert(fp);
                                                }
                                            });
                                        })
                                        .style(|s| {
                                            s.font_size(12.0)
                                                .text_overflow(floem::style::TextOverflow::Clip)
                                                .cursor(floem::style::CursorStyle::Pointer)
                                        }),
                                ))
                                .style(|s| {
                                    s.height(28.0)
                                        .padding_vert(4.0)
                                        .padding_horiz(8.0)
                                        .items_center()
                                        .flex_shrink(0.0)
                                        .hover(|s| s.background(Color::rgb8(245, 245, 245)))
                                })
                            },
                        )
                        .style(|s| s.flex_col())
                    )
                    .scroll_style(|s| s.handle_thickness(8.0))
                    .style(|s| {
                        s.width_full()
                            .height(300.0)
                            .border(1.0)
                            .border_color(Color::rgb8(220, 220, 220))
                            .border_radius(4.0)
                            .background(Color::WHITE)
                    }),
                    // Action buttons
                    h_stack((
                        empty().style(|s| s.flex_grow(1.0)),
                        button("Cancel")
                            .action(move || {
                                state_cancel.show_file_select.set(false);
                                state_cancel.file_select_pak.set(None);
                                state_cancel.file_select_list.set(Vec::new());
                                state_cancel.file_select_selected.set(std::collections::HashSet::new());
                                state_cancel.clear_results();
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
                        button("Extract Selected")
                            .action(move || {
                                execute_individual_extract(state_extract.clone());
                            })
                            .disabled(move || selected.get().is_empty())
                            .style(move |s| {
                                let disabled = selected.get().is_empty();
                                let s = s
                                    .padding_vert(8.0)
                                    .padding_horiz(20.0)
                                    .border_radius(4.0);
                                if disabled {
                                    s.background(Color::rgb8(200, 200, 200))
                                        .color(Color::rgb8(150, 150, 150))
                                } else {
                                    s.background(Color::rgb8(33, 150, 243))
                                        .color(Color::WHITE)
                                        .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                                }
                            }),
                    ))
                    .style(|s| s.width_full().margin_top(16.0)),
                ))
                .style(|s| {
                    s.padding(24.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .width(800.0)
                        .max_height(500.0)
                })
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
    .on_event_stop(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(key_event) = e {
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                state_escape.show_file_select.set(false);
                state_escape.file_select_pak.set(None);
                state_escape.file_select_list.set(Vec::new());
                state_escape.file_select_selected.set(std::collections::HashSet::new());
                state_escape.clear_results();
            }
        }
    })
    .keyboard_navigable()
}

/// Dialog shown when a folder is dropped, asking what action to take
pub fn folder_drop_action_dialog(state: PakOpsState) -> impl IntoView {
    let show = state.show_folder_drop_dialog;
    let dropped_folder = state.dropped_folder;

    let state_create = state.clone();
    let state_rebuild = state.clone();
    let state_validate = state.clone();
    let state_cancel = state.clone();

    dyn_container(
        move || show.get(),
        move |visible| {
            if visible {
                let folder_path = dropped_folder.get().unwrap_or_default();
                let folder_name = Path::new(&folder_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "folder".to_string());

                let state_create = state_create.clone();
                let state_rebuild = state_rebuild.clone();
                let state_validate = state_validate.clone();
                let state_cancel = state_cancel.clone();
                let folder_path_create = folder_path.clone();
                let folder_path_rebuild = folder_path.clone();
                let folder_path_validate = folder_path.clone();

                v_stack((
                    // Title
                    label(move || format!("Dropped: {}", folder_name)).style(|s| {
                        s.font_size(16.0)
                            .font_weight(Weight::BOLD)
                            .margin_bottom(16.0)
                    }),
                    // Action buttons
                    label(|| "What would you like to do?".to_string())
                        .style(|s| s.margin_bottom(12.0)),
                    // Create PAK button
                    button("🔧 Create PAK from Folder")
                        .action(move || {
                            state_create.show_folder_drop_dialog.set(false);
                            create_pak_from_dropped_folder(state_create.clone(), folder_path_create.clone());
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .margin_bottom(8.0)
                                .background(Color::rgb8(33, 150, 243))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                        }),
                    // Rebuild PAK button
                    button("🔧 Rebuild Modified PAK")
                        .action(move || {
                            state_rebuild.show_folder_drop_dialog.set(false);
                            rebuild_pak_from_dropped_folder(state_rebuild.clone(), folder_path_rebuild.clone());
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .margin_bottom(8.0)
                                .background(Color::rgb8(76, 175, 80))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(56, 142, 60)))
                        }),
                    // Validate button
                    button("✓ Validate Mod Structure")
                        .action(move || {
                            state_validate.show_folder_drop_dialog.set(false);
                            validate_dropped_folder(state_validate.clone(), folder_path_validate.clone());
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .margin_bottom(8.0)
                                .background(Color::rgb8(255, 152, 0))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(245, 124, 0)))
                        }),
                    // Cancel button
                    button("Cancel")
                        .action(move || {
                            state_cancel.show_folder_drop_dialog.set(false);
                            state_cancel.dropped_folder.set(None);
                        })
                        .style(|s| {
                            s.width_full()
                                .padding_vert(10.0)
                                .background(Color::rgb8(240, 240, 240))
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0)
                        }),
                ))
                .style(|s| {
                    s.padding(24.0)
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(8.0)
                        .width(320.0)
                })
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
    .on_event_stop(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(key_event) = e {
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                state.show_folder_drop_dialog.set(false);
                state.dropped_folder.set(None);
            }
        }
    })
    .keyboard_navigable()
}
