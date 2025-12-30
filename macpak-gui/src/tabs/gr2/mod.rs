//! GR2 Conversion Tab
//!
//! Convert between GR2 (Granny 3D) and glTF/GLB formats:
//! - Single file conversion
//! - Batch conversion of directories

mod conversion;
pub mod types;

use floem::action::exec_after;
use floem::prelude::*;
use floem::style::Position;
use floem::text::Weight;
use floem::views::{VirtualDirection, VirtualItemSize, virtual_list};
use floem_reactive::create_effect;
use im::Vector as ImVector;
use std::path::Path;
use std::time::Duration;
use walkdir::WalkDir;

use crate::state::{AppState, Gr2ConversionDirection, Gr2OutputFormat, Gr2State};
use conversion::{convert_batch, convert_single};
use types::get_shared_progress;

pub fn gr2_tab(_app_state: AppState, gr2_state: Gr2State) -> impl IntoView {
    let state = gr2_state.clone();

    // Main content with dialog overlay using absolute positioning
    // The v_stack has position: Relative so absolutely positioned children
    // are positioned relative to it
    v_stack((
        // Header
        header_section(gr2_state.clone()),
        // Main content area with form sections and results
        v_stack((
            // Form sections (fixed height content)
            v_stack((
                // Direction and format selector
                direction_section(gr2_state.clone()),
                // Single file conversion
                single_file_section(gr2_state.clone()),
                // Batch conversion
                batch_section(gr2_state.clone()),
            ))
            .style(|s| s.width_full().gap(16.0)),
            // Results log (fills remaining space)
            results_section(gr2_state.clone()),
        ))
        .style(|s| {
            s.width_full()
                .height_full()
                .min_height(0.0)
                .flex_grow(1.0)
                .flex_basis(0.0)
                .padding(24.0)
                .gap(16.0)
        }),
        // Progress overlay (shown when converting) - absolutely positioned
        progress_overlay(state),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(250, 250, 250))
            .position(Position::Relative)
    })
}

fn header_section(state: Gr2State) -> impl IntoView {
    h_stack((
        label(|| "GR2 Conversion")
            .style(|s| s.font_size(18.0).font_weight(Weight::BOLD)),
        empty().style(|s| s.flex_grow(1.0)),
        // Status message
        dyn_container(
            move || state.status_message.get(),
            move |msg| {
                if msg.is_empty() {
                    empty().into_any()
                } else {
                    label(move || msg.clone())
                        .style(|s| {
                            s.padding_horiz(12.0)
                                .padding_vert(6.0)
                                .background(Color::rgb8(232, 245, 233))
                                .border_radius(4.0)
                                .color(Color::rgb8(46, 125, 50))
                                .font_size(12.0)
                        })
                        .into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(16.0)
            .gap(8.0)
            .items_center()
            .background(Color::WHITE)
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn direction_section(state: Gr2State) -> impl IntoView {
    let state_for_gr2 = state.clone();
    let state_for_gltf = state.clone();
    let state_for_glb = state.clone();
    let state_for_gltf_fmt = state.clone();

    v_stack((
        label(|| "Conversion Settings")
            .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD).margin_bottom(8.0)),
        h_stack((
            // Direction buttons
            v_stack((
                label(|| "Direction").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                h_stack((
                    direction_button("GR2 -> glTF", Gr2ConversionDirection::Gr2ToGltf, state_for_gr2),
                    direction_button("glTF -> GR2", Gr2ConversionDirection::GltfToGr2, state_for_gltf),
                ))
                .style(|s| s.gap(8.0)),
            ))
            .style(|s| s.gap(4.0)),
            // Format buttons (only shown for GR2 -> glTF)
            dyn_container(
                move || state.direction.get(),
                move |dir| {
                    if dir == Gr2ConversionDirection::Gr2ToGltf {
                        v_stack((
                            label(|| "Output Format")
                                .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                            h_stack((
                                format_button("GLB", Gr2OutputFormat::Glb, state_for_glb.clone()),
                                format_button("glTF", Gr2OutputFormat::Gltf, state_for_gltf_fmt.clone()),
                            ))
                            .style(|s| s.gap(8.0)),
                        ))
                        .style(|s| s.gap(4.0).margin_left(24.0))
                        .into_any()
                    } else {
                        v_stack((
                            label(|| "Note")
                                .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                            label(|| "Output GR2 will be uncompressed")
                                .style(|s| {
                                    s.font_size(11.0)
                                        .color(Color::rgb8(150, 100, 50))
                                        .padding(6.0)
                                        .background(Color::rgb8(255, 248, 225))
                                        .border_radius(4.0)
                                }),
                        ))
                        .style(|s| s.gap(4.0).margin_left(24.0))
                        .into_any()
                    }
                },
            ),
        ))
        .style(|s| s.gap(16.0).items_end()),
    ))
    .style(|s| card_style(s))
}

fn direction_button(
    label_text: &'static str,
    btn_dir: Gr2ConversionDirection,
    state: Gr2State,
) -> impl IntoView {
    button(label_text)
        .style(move |s| {
            let is_selected = state.direction.get() == btn_dir;
            let s = s
                .padding_horiz(12.0)
                .padding_vert(6.0)
                .border_radius(4.0)
                .font_size(12.0);

            if is_selected {
                s.background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
            } else {
                s.background(Color::rgb8(240, 240, 240))
                    .color(Color::rgb8(60, 60, 60))
                    .hover(|s| s.background(Color::rgb8(220, 220, 220)))
            }
        })
        .action(move || {
            state.direction.set(btn_dir);
            // Clear file selections when direction changes
            state.input_file.set(None);
            state.output_file.set(None);
            state.batch_input_dir.set(None);
            state.batch_output_dir.set(None);
            state.batch_files.set(Vec::new());
        })
}

fn format_button(
    label_text: &'static str,
    btn_format: Gr2OutputFormat,
    state: Gr2State,
) -> impl IntoView {
    button(label_text)
        .style(move |s| {
            let is_selected = state.output_format.get() == btn_format;
            let s = s
                .padding_horiz(12.0)
                .padding_vert(6.0)
                .border_radius(4.0)
                .font_size(12.0);

            if is_selected {
                s.background(Color::rgb8(76, 175, 80))
                    .color(Color::WHITE)
            } else {
                s.background(Color::rgb8(240, 240, 240))
                    .color(Color::rgb8(60, 60, 60))
                    .hover(|s| s.background(Color::rgb8(220, 220, 220)))
            }
        })
        .action(move || {
            state.output_format.set(btn_format);
        })
}

fn single_file_section(state: Gr2State) -> impl IntoView {
    let state_select = state.clone();
    let state_convert = state.clone();

    v_stack((
        label(|| "Single File Conversion")
            .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD).margin_bottom(8.0)),
        h_stack((
            // Input file display
            v_stack((
                label(|| "Input File").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                dyn_container(
                    move || state.input_file.get(),
                    move |path| {
                        let display = path
                            .as_ref()
                            .and_then(|p| Path::new(p).file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "No file selected".to_string());
                        label(move || display.clone())
                            .style(|s| {
                                s.padding(8.0)
                                    .background(Color::rgb8(245, 245, 245))
                                    .border_radius(4.0)
                                    .min_width(200.0)
                                    .font_size(12.0)
                            })
                            .into_any()
                    },
                ),
            ))
            .style(|s| s.gap(4.0)),
            // Select button
            button("Select File")
                .style(|s| {
                    s.padding_horiz(16.0)
                        .padding_vert(8.0)
                        .background(Color::rgb8(33, 150, 243))
                        .color(Color::WHITE)
                        .border_radius(4.0)
                        .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                })
                .action(move || {
                    select_input_file(state_select.clone());
                }),
            empty().style(|s| s.flex_grow(1.0)),
            // Convert button
            {
                let state_btn = state_convert.clone();
                button("Convert")
                    .disabled(move || state_btn.input_file.get().is_none() || state_btn.is_converting.get())
                    .style(move |s| {
                        let disabled = state_convert.input_file.get().is_none() || state_convert.is_converting.get();
                        let s = s
                            .padding_horiz(20.0)
                            .padding_vert(10.0)
                            .border_radius(4.0)
                            .font_weight(Weight::SEMIBOLD);

                        if disabled {
                            s.background(Color::rgb8(200, 200, 200))
                                .color(Color::rgb8(150, 150, 150))
                        } else {
                            s.background(Color::rgb8(76, 175, 80))
                                .color(Color::WHITE)
                                .hover(|s| s.background(Color::rgb8(67, 160, 71)))
                        }
                    })
                    .action(move || {
                        convert_single(state.clone());
                    })
            },
        ))
        .style(|s| s.gap(12.0).items_end()),
    ))
    .style(|s| card_style(s))
}

fn batch_section(state: Gr2State) -> impl IntoView {
    let state_select = state.clone();
    let state_convert = state.clone();
    let state_out = state.clone();

    v_stack((
        label(|| "Batch Conversion")
            .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD).margin_bottom(8.0)),
        h_stack((
            // Input directory
            v_stack((
                label(|| "Input Directory").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                dyn_container(
                    move || state.batch_input_dir.get(),
                    move |path| {
                        let display = path
                            .as_ref()
                            .and_then(|p| Path::new(p).file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "No directory selected".to_string());
                        label(move || display.clone())
                            .style(|s| {
                                s.padding(8.0)
                                    .background(Color::rgb8(245, 245, 245))
                                    .border_radius(4.0)
                                    .min_width(150.0)
                                    .font_size(12.0)
                            })
                            .into_any()
                    },
                ),
            ))
            .style(|s| s.gap(4.0)),
            button("Select")
                .style(|s| {
                    s.padding_horiz(12.0)
                        .padding_vert(8.0)
                        .background(Color::rgb8(33, 150, 243))
                        .color(Color::WHITE)
                        .border_radius(4.0)
                        .hover(|s| s.background(Color::rgb8(25, 118, 210)))
                })
                .action(move || {
                    select_batch_input(state_select.clone());
                }),
            // Output directory
            v_stack((
                label(|| "Output Directory").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                dyn_container(
                    move || state_out.batch_output_dir.get(),
                    move |path| {
                        let display = path
                            .as_ref()
                            .and_then(|p| Path::new(p).file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Same as input".to_string());
                        label(move || display.clone())
                            .style(|s| {
                                s.padding(8.0)
                                    .background(Color::rgb8(245, 245, 245))
                                    .border_radius(4.0)
                                    .min_width(150.0)
                                    .font_size(12.0)
                            })
                            .into_any()
                    },
                ),
            ))
            .style(|s| s.gap(4.0).margin_left(12.0)),
            {
                let state_out_btn = state.clone();
                button("Select")
                    .style(|s| {
                        s.padding_horiz(12.0)
                            .padding_vert(8.0)
                            .background(Color::rgb8(100, 100, 100))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .hover(|s| s.background(Color::rgb8(80, 80, 80)))
                    })
                    .action(move || {
                        select_batch_output(state_out_btn.clone());
                    })
            },
            empty().style(|s| s.flex_grow(1.0)),
            // File count and convert button
            dyn_container(
                move || state.batch_files.get().len(),
                move |count| {
                    if count > 0 {
                        label(move || format!("{} files", count))
                            .style(|s| {
                                s.padding_horiz(8.0)
                                    .padding_vert(4.0)
                                    .background(Color::rgb8(232, 245, 233))
                                    .border_radius(4.0)
                                    .color(Color::rgb8(46, 125, 50))
                                    .font_size(12.0)
                            })
                            .into_any()
                    } else {
                        empty().into_any()
                    }
                },
            ),
            {
                let state_conv = state_convert.clone();
                button("Convert All")
                    .disabled(move || state_conv.batch_files.get().is_empty() || state_conv.is_converting.get())
                    .style(move |s| {
                        let disabled = state_convert.batch_files.get().is_empty() || state_convert.is_converting.get();
                        let s = s
                            .padding_horiz(20.0)
                            .padding_vert(10.0)
                            .border_radius(4.0)
                            .font_weight(Weight::SEMIBOLD);

                        if disabled {
                            s.background(Color::rgb8(200, 200, 200))
                                .color(Color::rgb8(150, 150, 150))
                        } else {
                            s.background(Color::rgb8(76, 175, 80))
                                .color(Color::WHITE)
                                .hover(|s| s.background(Color::rgb8(67, 160, 71)))
                        }
                    })
                    .action(move || {
                        convert_batch(state.clone());
                    })
            },
        ))
        .style(|s| s.gap(12.0).items_end()),
    ))
    .style(|s| card_style(s))
}

fn results_section(state: Gr2State) -> impl IntoView {
    let state_clear = state.clone();
    let show_failures_only = RwSignal::new(false);

    // Filtered results based on toggle
    let filtered_results = move || {
        let log = state.results_log.get();
        let filter = show_failures_only.get();
        if filter {
            log.into_iter()
                .filter(|msg| msg.starts_with("Error") || msg.starts_with("Failed"))
                .collect::<ImVector<_>>()
        } else {
            // Already an ImVector, return directly
            log
        }
    };

    // Count failures for badge
    let failure_count = move || {
        state.results_log.get()
            .iter()
            .filter(|msg| msg.starts_with("Error") || msg.starts_with("Failed"))
            .count()
    };

    v_stack((
        h_stack((
            label(|| "Results Log")
                .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD)),
            empty().style(|s| s.flex_grow(1.0)),
            // Show Failures Only toggle button
            button(label(move || {
                let count = failure_count();
                if show_failures_only.get() {
                    "Show All".to_string()
                } else if count > 0 {
                    format!("Failures ({})", count)
                } else {
                    "Failures".to_string()
                }
            }))
            .style(move |s| {
                let is_active = show_failures_only.get();
                let has_failures = failure_count() > 0;
                let s = s
                    .padding_horiz(10.0)
                    .padding_vert(4.0)
                    .font_size(11.0)
                    .border_radius(4.0)
                    .margin_right(8.0);

                if is_active {
                    s.background(Color::rgb8(211, 47, 47))
                        .color(Color::WHITE)
                } else if has_failures {
                    s.background(Color::rgb8(255, 235, 235))
                        .color(Color::rgb8(180, 30, 30))
                        .hover(|s| s.background(Color::rgb8(255, 220, 220)))
                } else {
                    s.background(Color::rgb8(240, 240, 240))
                        .color(Color::rgb8(150, 150, 150))
                }
            })
            .action(move || {
                show_failures_only.set(!show_failures_only.get());
            }),
            button("Clear")
                .style(|s| {
                    s.padding_horiz(12.0)
                        .padding_vert(4.0)
                        .font_size(11.0)
                        .background(Color::rgb8(240, 240, 240))
                        .border_radius(4.0)
                        .hover(|s| s.background(Color::rgb8(220, 220, 220)))
                })
                .action(move || {
                    state_clear.clear_results();
                    show_failures_only.set(false);
                }),
        ))
        .style(|s| s.width_full().margin_bottom(8.0)),
        scroll(
            virtual_list(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| 22.0)),
                filtered_results,
                |msg: &String| msg.clone(),
                |msg| {
                    let is_error = msg.starts_with("Error") || msg.starts_with("Failed");
                    container(
                        label(move || msg.clone())
                            .style(move |s| {
                                let s = s.font_size(11.0)
                                    .font_family("monospace".to_string());
                                if is_error {
                                    s.color(Color::rgb8(180, 30, 30))
                                } else {
                                    s.color(Color::rgb8(46, 125, 50))
                                }
                            }),
                    )
                    .style(move |s| {
                        let s = s.width_full()
                            .height(22.0)
                            .padding_vert(2.0)
                            .padding_horiz(4.0);
                        if is_error {
                            s.background(Color::rgb8(255, 235, 235))
                        } else {
                            s
                        }
                    })
                },
            )
            .style(|s| s.flex_col().width_full()),
        )
        .style(|s| {
            s.width_full()
                .height_full()
                .min_height(0.0)
                .flex_grow(1.0)
                .flex_basis(0.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        card_style(s)
            .height_full()
            .min_height(0.0)
            .flex_grow(1.0)
            .flex_basis(0.0)
    })
}

fn progress_overlay(state: Gr2State) -> impl IntoView {
    let show = state.is_converting;

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
        move |is_converting| {
            if is_converting {
                container(
                    v_stack((
                        // Count display (e.g., "1/5") - only show for batch
                        label(move || {
                            let total = polled_total.get();
                            if total > 1 {
                                format!("{}/{}", polled_current.get(), total)
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
            s.position(Position::Absolute)
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

fn card_style(s: floem::style::Style) -> floem::style::Style {
    s.width_full()
        .padding(16.0)
        .background(Color::WHITE)
        .border(1.0)
        .border_color(Color::rgb8(220, 220, 220))
        .border_radius(6.0)
}

fn select_input_file(state: Gr2State) {
    let direction = state.direction.get();
    let (title, filters): (&str, &[&str]) = match direction {
        Gr2ConversionDirection::Gr2ToGltf => ("Select GR2 File", &["gr2"]),
        Gr2ConversionDirection::GltfToGr2 => ("Select glTF/GLB File", &["glb", "gltf"]),
    };

    let mut dialog = rfd::FileDialog::new().set_title(title);

    if direction == Gr2ConversionDirection::Gr2ToGltf {
        dialog = dialog.add_filter("GR2 Files", filters);
    } else {
        dialog = dialog.add_filter("glTF Files", filters);
    }

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(file) = dialog.pick_file() {
        if let Some(parent) = file.parent() {
            state.working_dir.set(Some(parent.to_string_lossy().to_string()));
        }
        state.input_file.set(Some(file.to_string_lossy().to_string()));
    }
}

fn select_batch_input(state: Gr2State) {
    let direction = state.direction.get();
    let title = match direction {
        Gr2ConversionDirection::Gr2ToGltf => "Select Directory with GR2 Files",
        Gr2ConversionDirection::GltfToGr2 => "Select Directory with glTF/GLB Files",
    };

    let mut dialog = rfd::FileDialog::new().set_title(title);

    if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(dir) = dialog.pick_folder() {
        state.working_dir.set(Some(dir.to_string_lossy().to_string()));
        state.batch_input_dir.set(Some(dir.to_string_lossy().to_string()));

        // Recursively scan for matching files using walkdir
        let mut files = Vec::new();
        for entry in WalkDir::new(&dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    let matches = match direction {
                        Gr2ConversionDirection::Gr2ToGltf => ext_lower == "gr2",
                        Gr2ConversionDirection::GltfToGr2 => ext_lower == "glb" || ext_lower == "gltf",
                    };
                    if matches {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Sort for consistent ordering
        files.sort();
        state.batch_files.set(files);
    }
}

fn select_batch_output(state: Gr2State) {
    let mut dialog = rfd::FileDialog::new().set_title("Select Output Directory");

    if let Some(dir) = state.batch_input_dir.get() {
        dialog = dialog.set_directory(&dir);
    } else if let Some(dir) = state.working_dir.get() {
        dialog = dialog.set_directory(&dir);
    }

    if let Some(dir) = dialog.pick_folder() {
        state.batch_output_dir.set(Some(dir.to_string_lossy().to_string()));
    }
}
