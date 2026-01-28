//! Color row component for dye entries

use floem::prelude::*;

use super::shared::constants::*;
use super::shared::{
    copy_to_clipboard, normalize_hex, parse_hex_color, parse_hex_to_color, pick_color_from_screen,
};
use crate::gui::state::DyeColorEntry;

/// Creates a single color row with label, color swatch (clickable for eyedropper), hex input, and sRGB
pub fn color_row(entry: DyeColorEntry, status: RwSignal<String>) -> impl IntoView {
    let hex = entry.hex;
    let name = entry.name;

    h_stack((
        // Category name (first column)
        label(move || name).style(|s| {
            s.width(140.0)
                .font_size(FONT_BODY)
                .font_family("monospace".to_string())
        }),
        // Color picker section (second column)
        h_stack((
            // Color swatch - click to open eyedropper
            {
                let hex_copy = hex;
                empty()
                    .style(move |s| {
                        let color = parse_hex_to_color(&hex_copy.get());
                        s.width(24.0)
                            .height(18.0)
                            .border_radius(RADIUS_SM)
                            .border(1.0)
                            .border_color(BORDER_DARK)
                            .background(color)
                            .cursor(floem::style::CursorStyle::Pointer)
                    })
                    .on_click_stop(move |_| {
                        if let Some((r, g, b)) = pick_color_from_screen() {
                            hex_copy.set(format!("{:02X}{:02X}{:02X}", r, g, b));
                        }
                    })
            },
            // Hash symbol
            label(|| "#").style(|s| {
                s.font_size(FONT_SMALL)
                    .font_family("monospace".to_string())
                    .margin_left(4.0)
            }),
            // Hex input
            text_input(hex)
                .style(|s| {
                    s.width(56.0)
                        .padding(3.0)
                        .font_size(FONT_SMALL)
                        .font_family("monospace".to_string())
                        .background(Color::WHITE)
                        .border(1.0)
                        .border_color(BORDER_INPUT)
                        .border_radius(RADIUS_SM)
                })
                .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                    // Validate and normalize hex on focus lost
                    let val = hex.get();
                    let normalized = normalize_hex(&val);
                    if normalized != val {
                        hex.set(normalized);
                    }
                }),
            // sRGB preview (space-separated floats for BG3)
            {
                let hex_copy = hex;
                let status_copy = status;
                label(move || {
                    let (r, g, b) = parse_hex_color(&hex_copy.get()).unwrap_or((128, 128, 128));
                    let sr = r as f32 / 255.0;
                    let sg = g as f32 / 255.0;
                    let sb = b as f32 / 255.0;
                    format!("{:.2} {:.2} {:.2}", sr, sg, sb)
                })
                .style(|s| {
                    s.margin_left(PADDING_BTN_V)
                        .padding(3.0)
                        .font_size(FONT_TINY)
                        .font_family("monospace".to_string())
                        .background(BG_DISABLED)
                        .border(1.0)
                        .border_color(BORDER_INPUT)
                        .border_radius(RADIUS_SM)
                        .cursor(floem::style::CursorStyle::Pointer)
                        .color(TEXT_DARK)
                })
                .on_click_stop(move |_| {
                    let (r, g, b) = parse_hex_color(&hex_copy.get()).unwrap_or((128, 128, 128));
                    let sr = r as f32 / 255.0;
                    let sg = g as f32 / 255.0;
                    let sb = b as f32 / 255.0;
                    copy_to_clipboard(&format!("{:.4} {:.4} {:.4}", sr, sg, sb));
                    status_copy.set("Copied sRGB".to_string());
                })
            },
        ))
        .style(|s| s.items_center().gap(2.0)),
    ))
    .style(|s| s.padding_vert(2.0).padding_horiz(4.0).items_center())
}
