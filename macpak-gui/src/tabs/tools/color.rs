//! Color Picker

use floem::prelude::*;
use floem::text::Weight;
use floem::unit::Pct;
use floem::views::slider;
use std::process::Command;

use crate::state::ToolsState;
use super::helpers::copy_to_clipboard;
use super::widgets::tool_card_style;

pub fn color_picker_section(state: ToolsState) -> impl IntoView {
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
                {
                    let status_copy = status;
                    let hex_copy = hex;
                    empty()
                        .style(move |s| {
                            s.width(60.0)
                                .height(48.0)
                                .border_radius(4.0)
                                .border(1.0)
                                .border_color(Color::rgb8(180, 180, 180))
                                .background(Color::rgba8(r.get(), g.get(), b.get(), a.get()))
                                .cursor(floem::style::CursorStyle::Pointer)
                        })
                        .on_click_stop(move |_| {
                            let hex_val = format!("#{}", hex_copy.get());
                            copy_to_clipboard(&hex_val);
                            status_copy.set("Copied hex!".to_string());
                        })
                },
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
        color_slider("R", r, hex, g, b),
        color_slider("G", g, hex, r, b),
        color_slider("B", b, hex, r, g),
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
                    if hist.len() > 43 {
                        hist.truncate(43);
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

pub fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
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
