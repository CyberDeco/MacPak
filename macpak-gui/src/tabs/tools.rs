//! Tools Tab
//!
//! Utilities for BG3 modding:
//! - UUID Generator
//! - Handle Generator
//! - Color Picker
//! - Version Calculator

use floem::prelude::*;
use floem::text::Weight;
use floem::unit::Pct;
use floem::views::slider;
use rand::Rng;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::state::{AppState, ToolsState, UuidFormat};

pub fn tools_tab(_app_state: AppState, tools_state: ToolsState) -> impl IntoView {
    let state_export = tools_state.clone();
    let state_clear = tools_state.clone();

    scroll(
        v_stack((
            // Title and actions
            h_stack((
                label(|| "Modding Tools")
                    .style(|s| s.font_size(24.0).font_weight(Weight::BOLD)),
                empty().style(|s| s.flex_grow(1.0)),
                button("Export History").action(move || {
                    export_history(state_export.clone());
                }),
                button("Clear All").action(move || {
                    clear_all(state_clear.clone());
                }),
            ))
            .style(|s| s.width_full().gap(8.0).items_center().margin_bottom(16.0)),

            // Status message
            status_bar(tools_state.status_message),

            // Row 1: UUID and Handle generators
            h_stack((
                uuid_section(tools_state.clone()),
                handle_section(tools_state.clone()),
            ))
            .style(|s| s.width_full().gap(16.0)),

            // Row 2: Color Picker and Version Calculator
            h_stack((
                color_picker_section(tools_state.clone()),
                version_calculator_section(tools_state.clone()),
            ))
            .style(|s| s.width_full().gap(16.0).margin_top(16.0)),

            // Row 3: History sections
            h_stack((
                uuid_history_section(tools_state.clone()),
                handle_history_section(tools_state.clone()),
            ))
            .style(|s| s.width_full().gap(16.0).margin_top(16.0)),

            // Row 4: Color history
            color_history_section(tools_state),
        ))
        .style(|s| s.width_full().padding(24.0)),
    )
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(250, 250, 250))
    })
}

fn status_bar(status: RwSignal<String>) -> impl IntoView {
    dyn_container(
        move || status.get(),
        move |msg| {
            if msg.is_empty() {
                empty().into_any()
            } else {
                label(move || msg.clone())
                    .style(|s| {
                        s.width_full()
                            .padding(8.0)
                            .margin_bottom(12.0)
                            .background(Color::rgb8(232, 245, 233))
                            .border_radius(4.0)
                            .color(Color::rgb8(46, 125, 50))
                            .font_size(12.0)
                    })
                    .into_any()
            }
        },
    )
}

// ============================================================================
// UUID Section
// ============================================================================

fn uuid_section(state: ToolsState) -> impl IntoView {
    let uuid = state.generated_uuid;
    let format = state.uuid_format;
    let history = state.uuid_history;
    let status = state.status_message;

    v_stack((
        label(|| "UUID Generator").style(|s| s.font_size(16.0).font_weight(Weight::BOLD).margin_bottom(12.0)),

        // Format selection
        h_stack((
            label(|| "Format:").style(|s| s.margin_right(8.0).font_size(12.0)),
            format_button("Standard", UuidFormat::Standard, format),
            format_button("Compact", UuidFormat::Compact, format),
            format_button("Larian", UuidFormat::Larian, format),
        ))
        .style(|s| s.gap(4.0).items_center().margin_bottom(12.0)),

        // Generated UUID display
        h_stack((
            label(move || {
                let u = uuid.get();
                if u.is_empty() {
                    "Click Generate".to_string()
                } else {
                    u
                }
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(10.0)
                    .font_size(13.0)
                    .font_family("monospace".to_string())
                    .background(Color::rgb8(245, 245, 245))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
            copy_button(uuid, status),
        ))
        .style(|s| s.width_full().gap(6.0)),

        // Generate button
        button("Generate UUID")
            .style(|s| {
                s.width_full()
                    .padding_vert(10.0)
                    .margin_top(10.0)
                    .font_size(13.0)
                    .background(Color::rgb8(33, 150, 243))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(25, 118, 210)))
            })
            .action(move || {
                let new_uuid = generate_uuid(format.get());
                uuid.set(new_uuid.clone());

                let mut hist = history.get();
                hist.insert(0, new_uuid);
                if hist.len() > 20 {
                    hist.truncate(20);
                }
                history.set(hist);
            }),
    ))
    .style(|s| tool_card_style(s))
}

// ============================================================================
// Handle Section
// ============================================================================

fn handle_section(state: ToolsState) -> impl IntoView {
    let handle = state.generated_handle;
    let history = state.handle_history;
    let status = state.status_message;

    v_stack((
        label(|| "Handle Generator").style(|s| s.font_size(16.0).font_weight(Weight::BOLD).margin_bottom(12.0)),

        label(|| "Random u64 handles for TranslatedStrings")
            .style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100)).margin_bottom(12.0)),

        // Generated handle display
        h_stack((
            label(move || {
                let h = handle.get();
                if h.is_empty() {
                    "Click Generate".to_string()
                } else {
                    format!("h{}", h)
                }
            })
            .style(|s| {
                s.flex_grow(1.0)
                    .padding(10.0)
                    .font_size(13.0)
                    .font_family("monospace".to_string())
                    .background(Color::rgb8(245, 245, 245))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
            {
                let handle_copy = handle;
                let status_copy = status;
                button("Copy")
                    .style(|s| s.padding_horiz(12.0).padding_vert(8.0).font_size(12.0))
                    .action(move || {
                        let h = handle_copy.get();
                        if !h.is_empty() {
                            copy_to_clipboard(&format!("h{}", h));
                            status_copy.set("Copied!".to_string());
                        }
                    })
            },
        ))
        .style(|s| s.width_full().gap(6.0)),

        // Generate button
        button("Generate Handle")
            .style(|s| {
                s.width_full()
                    .padding_vert(10.0)
                    .margin_top(10.0)
                    .font_size(13.0)
                    .background(Color::rgb8(156, 39, 176))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(123, 31, 162)))
            })
            .action(move || {
                let new_handle = generate_handle();
                handle.set(new_handle.clone());

                let mut hist = history.get();
                hist.insert(0, new_handle);
                if hist.len() > 20 {
                    hist.truncate(20);
                }
                history.set(hist);
            }),
    ))
    .style(|s| tool_card_style(s))
}

// ============================================================================
// Color Picker Section
// ============================================================================

fn color_picker_section(state: ToolsState) -> impl IntoView {
    let hex = state.color_hex;
    let r = state.color_r;
    let g = state.color_g;
    let b = state.color_b;
    let a = state.color_a;
    let history = state.color_history;
    let status = state.status_message;

    v_stack((
        label(|| "Color Picker").style(|s| s.font_size(16.0).font_weight(Weight::BOLD).margin_bottom(12.0)),

        // Color preview box
        h_stack((
            // Color swatch with eyedropper button
            v_stack((
                empty()
                    .style(move |s| {
                        s.width(60.0)
                            .height(48.0)
                            .border_radius(4.0)
                            .border(1.0)
                            .border_color(Color::rgb8(180, 180, 180))
                            .background(Color::rgba8(r.get(), g.get(), b.get(), a.get()))
                    }),
                {
                    let status_copy = status;
                    button("💧")
                        .style(|s| {
                            s.width(60.0)
                                .padding_vert(2.0)
                                .font_size(14.0)
                                .margin_top(4.0)
                                .background(Color::rgb8(100, 100, 100))
                                .color(Color::WHITE)
                                .border_radius(4.0)
                                .hover(|s| s.background(Color::rgb8(80, 80, 80)))
                        })
                        .action(move || {
                            if let Some((nr, ng, nb)) = pick_color_from_screen() {
                                r.set(nr);
                                g.set(ng);
                                b.set(nb);
                                hex.set(format!("{:02X}{:02X}{:02X}", nr, ng, nb));
                                status_copy.set("Color sampled!".to_string());
                            }
                        })
                },
            )),

            // Color format fields in a row
            h_stack((
                // Hex field
                v_stack((
                    label(|| "Hex").style(|s| s.font_size(10.0).color(Color::rgb8(100, 100, 100))),
                    h_stack((
                        label(|| "#").style(|s| s.font_size(13.0).font_family("monospace".to_string())),
                        text_input(hex)
                            .style(|s| {
                                s.width(72.0)
                                    .padding(6.0)
                                    .font_size(13.0)
                                    .font_family("monospace".to_string())
                                    .background(Color::rgb8(245, 245, 245))
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                            })
                            .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                                let hex_val = hex.get();
                                if let Some((nr, ng, nb)) = parse_hex_color(&hex_val) {
                                    r.set(nr);
                                    g.set(ng);
                                    b.set(nb);
                                }
                            }),
                    ))
                    .style(|s| s.items_center().gap(2.0)),
                )),

                // RGB field
                v_stack((
                    label(|| "RGB").style(|s| s.font_size(10.0).color(Color::rgb8(100, 100, 100))),
                    {
                        let status_copy = status;
                        label(move || format!("{}, {}, {}", r.get(), g.get(), b.get()))
                            .style(|s| {
                                s.padding(6.0)
                                    .font_size(13.0)
                                    .font_family("monospace".to_string())
                                    .background(Color::rgb8(245, 245, 245))
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                                    .cursor(floem::style::CursorStyle::Pointer)
                            })
                            .on_click_stop(move |_| {
                                copy_to_clipboard(&format!("{}, {}, {}", r.get(), g.get(), b.get()));
                                status_copy.set("Copied RGB!".to_string());
                            })
                    },
                )),

                // sRGB field
                v_stack((
                    label(|| "sRGB").style(|s| s.font_size(10.0).color(Color::rgb8(100, 100, 100))),
                    {
                        let status_copy = status;
                        label(move || {
                            let sr = r.get() as f32 / 255.0;
                            let sg = g.get() as f32 / 255.0;
                            let sb = b.get() as f32 / 255.0;
                            format!("{:.2} {:.2} {:.2}", sr, sg, sb)
                        })
                            .style(|s| {
                                s.padding(6.0)
                                    .font_size(13.0)
                                    .font_family("monospace".to_string())
                                    .background(Color::rgb8(245, 245, 245))
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                                    .cursor(floem::style::CursorStyle::Pointer)
                            })
                            .on_click_stop(move |_| {
                                let sr = r.get() as f32 / 255.0;
                                let sg = g.get() as f32 / 255.0;
                                let sb = b.get() as f32 / 255.0;
                                copy_to_clipboard(&format!("{:.4} {:.4} {:.4}", sr, sg, sb));
                                status_copy.set("Copied sRGB!".to_string());
                            })
                    },
                )),
            ))
            .style(|s| s.gap(12.0).margin_left(16.0)),
        ))
        .style(|s| s.items_start().margin_bottom(12.0)),

        // RGBA sliders
        color_slider("R", r, Color::rgb8(220, 50, 50), hex, g, b),
        color_slider("G", g, Color::rgb8(50, 180, 50), hex, r, b),
        color_slider("B", b, Color::rgb8(50, 100, 220), hex, r, g),
        color_slider_alpha("A", a),

        // Add to history button
        button("Save to History")
            .style(|s| {
                s.width_full()
                    .padding_vert(8.0)
                    .margin_top(8.0)
                    .font_size(12.0)
                    .background(Color::rgb8(76, 175, 80))
                    .color(Color::WHITE)
                    .border_radius(4.0)
                    .hover(|s| s.background(Color::rgb8(56, 142, 60)))
            })
            .action(move || {
                let color = format!("#{}", hex.get());
                let mut hist = history.get();
                if !hist.contains(&color) {
                    hist.insert(0, color);
                    if hist.len() > 16 {
                        hist.truncate(16);
                    }
                    history.set(hist);
                }
            }),
    ))
    .style(|s| tool_card_style(s))
}

fn color_slider(
    label_text: &'static str,
    value: RwSignal<u8>,
    _color: Color,
    hex: RwSignal<String>,
    other1: RwSignal<u8>,
    other2: RwSignal<u8>,
) -> impl IntoView {
    h_stack((
        label(move || label_text)
            .style(|s| s.width(16.0).font_size(11.0).font_weight(Weight::BOLD)),
        slider::Slider::new(move || Pct(value.get() as f64 / 255.0 * 100.0))
            .style(|s| s.flex_grow(1.0))
            .on_change_pct(move |v| {
                let new_val = (v.0 / 100.0 * 255.0).round() as u8;
                value.set(new_val);
                // Update hex
                let (r, g, b) = if label_text == "R" {
                    (new_val, other1.get(), other2.get())
                } else if label_text == "G" {
                    (other1.get(), new_val, other2.get())
                } else {
                    (other1.get(), other2.get(), new_val)
                };
                hex.set(format!("{:02X}{:02X}{:02X}", r, g, b));
            }),
        label(move || format!("{:3}", value.get()))
            .style(|s| s.width(28.0).font_size(11.0).font_family("monospace".to_string())),
    ))
    .style(|s| s.width_full().gap(6.0).items_center().margin_top(4.0))
}

fn color_slider_alpha(label_text: &'static str, value: RwSignal<u8>) -> impl IntoView {
    h_stack((
        label(move || label_text)
            .style(|s| s.width(16.0).font_size(11.0).font_weight(Weight::BOLD)),
        slider::Slider::new(move || Pct(value.get() as f64 / 255.0 * 100.0))
            .style(|s| s.flex_grow(1.0))
            .on_change_pct(move |v| {
                value.set((v.0 / 100.0 * 255.0).round() as u8);
            }),
        label(move || format!("{:3}", value.get()))
            .style(|s| s.width(28.0).font_size(11.0).font_family("monospace".to_string())),
    ))
    .style(|s| s.width_full().gap(6.0).items_center().margin_top(4.0))
}

// ============================================================================
// Version Calculator Section
// ============================================================================

fn version_calculator_section(state: ToolsState) -> impl IntoView {
    let version_int = state.version_int;
    let major = state.version_major;
    let minor = state.version_minor;
    let patch = state.version_patch;
    let build = state.version_build;
    let status = state.status_message;

    v_stack((
        label(|| "Version Calculator").style(|s| s.font_size(16.0).font_weight(Weight::BOLD).margin_bottom(12.0)),

        label(|| "BG3 uses Int64 version numbers (Major.Minor.Patch.Build)")
            .style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100)).margin_bottom(12.0)),

        // Int64 input
        h_stack((
            label(|| "Int64:").style(|s| s.font_size(12.0).width(50.0)),
            text_input(version_int)
                .placeholder("Enter version number...")
                .style(|s| {
                    s.flex_grow(1.0)
                        .padding(8.0)
                        .font_size(13.0)
                        .font_family("monospace".to_string())
                        .background(Color::rgb8(245, 245, 245))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                }),
            button("Parse")
                .style(|s| s.padding_horiz(12.0).padding_vert(8.0).font_size(12.0))
                .action(move || {
                    let input = version_int.get();
                    if let Ok(v) = input.trim().parse::<u64>() {
                        let (ma, mi, pa, bu) = int_to_version(v);
                        major.set(ma);
                        minor.set(mi);
                        patch.set(pa);
                        build.set(bu);
                        status.set(format!("Parsed: {}.{}.{}.{}", ma, mi, pa, bu));
                    } else {
                        status.set("Invalid number".to_string());
                    }
                }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),

        // Version components
        h_stack((
            version_field("Major", major),
            version_field("Minor", minor),
            version_field("Patch", patch),
            version_field("Build", build),
        ))
        .style(|s| s.width_full().gap(8.0).margin_top(12.0)),

        // Readable version display
        h_stack((
            label(move || format!("{}.{}.{}.{}", major.get(), minor.get(), patch.get(), build.get()))
                .style(|s| {
                    s.flex_grow(1.0)
                        .padding(10.0)
                        .font_size(14.0)
                        .font_weight(Weight::BOLD)
                        .font_family("monospace".to_string())
                        .background(Color::rgb8(245, 245, 245))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                        .justify_center()
                }),
        ))
        .style(|s| s.width_full().margin_top(12.0)),

        // Action buttons
        h_stack((
            {
                let status_copy = status;
                button("Copy Int64")
                    .style(|s| {
                        s.flex_grow(1.0)
                            .padding_vert(10.0)
                            .font_size(12.0)
                            .background(Color::rgb8(255, 152, 0))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .hover(|s| s.background(Color::rgb8(245, 124, 0)))
                    })
                    .action(move || {
                        let v = version_to_int(major.get(), minor.get(), patch.get(), build.get());
                        copy_to_clipboard(&v.to_string());
                        version_int.set(v.to_string());
                        status_copy.set("Copied Int64!".to_string());
                    })
            },
            {
                let status_copy = status;
                button("Copy Readable")
                    .style(|s| {
                        s.flex_grow(1.0)
                            .padding_vert(10.0)
                            .font_size(12.0)
                            .background(Color::rgb8(0, 150, 136))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .hover(|s| s.background(Color::rgb8(0, 121, 107)))
                    })
                    .action(move || {
                        let readable = format!("{}.{}.{}.{}", major.get(), minor.get(), patch.get(), build.get());
                        copy_to_clipboard(&readable);
                        status_copy.set("Copied version string!".to_string());
                    })
            },
        ))
        .style(|s| s.width_full().gap(8.0).margin_top(12.0)),
    ))
    .style(|s| tool_card_style(s))
}

fn version_field(label_text: &'static str, value: RwSignal<u32>) -> impl IntoView {
    v_stack((
        label(move || label_text).style(|s| s.font_size(10.0).color(Color::rgb8(100, 100, 100))),
        h_stack((
            button("-")
                .style(|s| s.padding_horiz(8.0).padding_vert(4.0).font_size(12.0))
                .action(move || {
                    let v = value.get();
                    if v > 0 {
                        value.set(v - 1);
                    }
                }),
            label(move || format!("{}", value.get()))
                .style(|s| {
                    s.min_width(30.0)
                        .padding_horiz(6.0)
                        .font_size(13.0)
                        .font_family("monospace".to_string())
                        .justify_center()
                }),
            button("+")
                .style(|s| s.padding_horiz(8.0).padding_vert(4.0).font_size(12.0))
                .action(move || {
                    value.set(value.get() + 1);
                }),
        ))
        .style(|s| s.items_center()),
    ))
    .style(|s| s.items_center())
}

// ============================================================================
// History Sections
// ============================================================================

fn uuid_history_section(state: ToolsState) -> impl IntoView {
    let history = state.uuid_history;
    let status = state.status_message;

    v_stack((
        label(|| "UUID History").style(|s| s.font_size(13.0).font_weight(Weight::BOLD).margin_bottom(6.0)),

        scroll(
            dyn_stack(
                move || history.get(),
                |uuid| uuid.clone(),
                move |uuid| {
                    history_row(uuid, status)
                },
            )
            .style(|s| s.width_full().flex_direction(floem::style::FlexDirection::Column)),
        )
        .style(|s| {
            s.width_full()
                .height(100.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_pct(50.0)
            .padding(12.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
    })
}

fn handle_history_section(state: ToolsState) -> impl IntoView {
    let history = state.handle_history;
    let status = state.status_message;

    v_stack((
        label(|| "Handle History").style(|s| s.font_size(13.0).font_weight(Weight::BOLD).margin_bottom(6.0)),

        scroll(
            dyn_stack(
                move || history.get(),
                |handle| handle.clone(),
                move |handle| {
                    let display = format!("h{}", handle);
                    history_row(display, status)
                },
            )
            .style(|s| s.width_full().flex_direction(floem::style::FlexDirection::Column)),
        )
        .style(|s| {
            s.width_full()
                .height(100.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_pct(50.0)
            .padding(12.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
    })
}

fn color_history_section(state: ToolsState) -> impl IntoView {
    let history = state.color_history;
    let status = state.status_message;

    v_stack((
        label(|| "Color History").style(|s| s.font_size(13.0).font_weight(Weight::BOLD).margin_bottom(6.0)),

        dyn_container(
            move || history.get().is_empty(),
            move |is_empty| {
                if is_empty {
                    label(|| "No colors saved yet")
                        .style(|s| s.color(Color::rgb8(150, 150, 150)).font_size(12.0))
                        .into_any()
                } else {
                    dyn_stack(
                        move || history.get(),
                        |color| color.clone(),
                        move |color| {
                            let hex = color.trim_start_matches('#').to_string();
                            let (r, g, b) = parse_hex_color(&hex).unwrap_or((128, 128, 128));
                            let color_clone = color.clone();
                            let status_copy = status;

                            empty()
                                .style(move |s| {
                                    s.width(28.0)
                                        .height(28.0)
                                        .border_radius(4.0)
                                        .border(1.0)
                                        .border_color(Color::rgb8(180, 180, 180))
                                        .background(Color::rgb8(r, g, b))
                                        .cursor(floem::style::CursorStyle::Pointer)
                                })
                                .on_click_stop(move |_| {
                                    copy_to_clipboard(&color_clone);
                                    status_copy.set(format!("Copied {}", color_clone));
                                })
                        },
                    )
                    .style(|s| s.gap(6.0).flex_wrap(floem::style::FlexWrap::Wrap).flex_direction(floem::style::FlexDirection::Row))
                    .into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
            .margin_top(16.0)
    })
}

// ============================================================================
// Shared Components
// ============================================================================

fn tool_card_style(s: floem::style::Style) -> floem::style::Style {
    s.width_pct(50.0)
        .padding(16.0)
        .background(Color::WHITE)
        .border(1.0)
        .border_color(Color::rgb8(220, 220, 220))
        .border_radius(6.0)
}

fn format_button(
    label_text: &'static str,
    btn_format: UuidFormat,
    current_format: RwSignal<UuidFormat>,
) -> impl IntoView {
    button(label_text)
        .style(move |s| {
            let is_selected = current_format.get() == btn_format;
            let s = s
                .padding_horiz(10.0)
                .padding_vert(4.0)
                .border_radius(4.0)
                .font_size(11.0);

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
            current_format.set(btn_format);
        })
}

fn copy_button(value: RwSignal<String>, status: RwSignal<String>) -> impl IntoView {
    button("Copy")
        .style(|s| s.padding_horiz(12.0).padding_vert(8.0).font_size(12.0))
        .action(move || {
            let v = value.get();
            if !v.is_empty() {
                copy_to_clipboard(&v);
                status.set("Copied!".to_string());
            }
        })
}

fn history_row(value: String, status: RwSignal<String>) -> impl IntoView {
    let value_copy = value.clone();

    h_stack((
        label(move || value.clone())
            .style(|s| {
                s.flex_grow(1.0)
                    .font_family("monospace".to_string())
                    .font_size(11.0)
            }),
        button("Copy")
            .style(|s| s.font_size(10.0).padding_horiz(8.0).padding_vert(2.0))
            .action(move || {
                copy_to_clipboard(&value_copy);
                status.set("Copied!".to_string());
            }),
    ))
    .style(|s| {
        s.width_full()
            .padding(6.0)
            .border_bottom(1.0)
            .border_color(Color::rgb8(240, 240, 240))
            .hover(|s| s.background(Color::rgb8(245, 245, 245)))
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_uuid(format: UuidFormat) -> String {
    let uuid = uuid::Uuid::new_v4();

    match format {
        UuidFormat::Standard => uuid.to_string().to_uppercase(),
        UuidFormat::Compact => uuid.simple().to_string().to_uppercase(),
        UuidFormat::Larian => {
            let simple = uuid.simple().to_string();
            format!(
                "h{}g{}g{}g{}g{}",
                &simple[0..8],
                &simple[8..12],
                &simple[12..16],
                &simple[16..20],
                &simple[20..32]
            )
        }
    }
}

fn generate_handle() -> String {
    let handle: u64 = rand::thread_rng().gen();
    handle.to_string()
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Opens the native macOS color picker and returns the selected color as (R, G, B).
/// The color picker includes an eyedropper tool for sampling screen colors.
fn pick_color_from_screen() -> Option<(u8, u8, u8)> {
    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to open the native color picker
        // The "choose color" command returns 16-bit RGB values (0-65535)
        let script = r#"choose color"#;

        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let result = String::from_utf8_lossy(&output.stdout);
        // Output format: "red, green, blue" where values are 0-65535
        // Example: "65535, 32768, 0"
        let parts: Vec<&str> = result.trim().split(", ").collect();
        if parts.len() >= 3 {
            let r16: u32 = parts[0].parse().ok()?;
            let g16: u32 = parts[1].parse().ok()?;
            let b16: u32 = parts[2].parse().ok()?;

            // Convert 16-bit to 8-bit
            let r = (r16 / 257) as u8;
            let g = (g16 / 257) as u8;
            let b = (b16 / 257) as u8;

            return Some((r, g, b));
        }
        None
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn version_to_int(major: u32, minor: u32, patch: u32, build: u32) -> u64 {
    ((major as u64) << 55)
        | ((minor as u64) << 47)
        | ((patch as u64) << 31)
        | (build as u64)
}

fn int_to_version(v: u64) -> (u32, u32, u32, u32) {
    let major = ((v >> 55) & 0x1FF) as u32;      // 9 bits
    let minor = ((v >> 47) & 0xFF) as u32;        // 8 bits
    let patch = ((v >> 31) & 0xFFFF) as u32;      // 16 bits
    let build = (v & 0x7FFFFFFF) as u32;          // 31 bits
    (major, minor, patch, build)
}

fn copy_to_clipboard(value: &str) {
    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(value.as_bytes());
            }
            let _ = child.wait();
        }
    }
}

fn export_history(state: ToolsState) {
    let uuids = state.uuid_history.get();
    let handles = state.handle_history.get();
    let colors = state.color_history.get();

    let json = serde_json::json!({
        "uuids": uuids,
        "handles": handles.iter().map(|h| format!("h{}", h)).collect::<Vec<_>>(),
        "colors": colors
    });

    let dialog = rfd::FileDialog::new()
        .set_title("Export History")
        .add_filter("JSON", &["json"])
        .set_file_name("macpak_tools_history.json");

    if let Some(path) = dialog.save_file() {
        match fs::write(&path, serde_json::to_string_pretty(&json).unwrap()) {
            Ok(_) => {
                state.status_message.set("Exported successfully!".to_string());
            }
            Err(e) => {
                state.status_message.set(format!("Export failed: {}", e));
            }
        }
    }
}

fn clear_all(state: ToolsState) {
    state.uuid_history.set(Vec::new());
    state.handle_history.set(Vec::new());
    state.color_history.set(Vec::new());
    state.generated_uuid.set(String::new());
    state.generated_handle.set(String::new());
    state.status_message.set("All history cleared".to_string());
}
