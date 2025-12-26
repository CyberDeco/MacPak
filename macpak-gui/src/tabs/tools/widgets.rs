//! Shared Widgets

use floem::prelude::*;

use crate::state::UuidFormat;
use super::helpers::copy_to_clipboard;

pub fn tool_card_style(s: floem::style::Style) -> floem::style::Style {
    s.width_pct(50.0)
        .padding(16.0)
        .background(Color::WHITE)
        .border(1.0)
        .border_color(Color::rgb8(220, 220, 220))
        .border_radius(6.0)
}

pub fn format_button(
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

pub fn copy_button(value: RwSignal<String>, status: RwSignal<String>) -> impl IntoView {
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
