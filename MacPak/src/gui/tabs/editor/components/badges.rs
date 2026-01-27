//! Badge components for format and save status display

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::EditorTabsState;

pub fn format_badge(tabs_state: EditorTabsState) -> impl IntoView {
    dyn_container(
        move || tabs_state.active_tab().map(|tab| tab.file_format.get()),
        move |maybe_format| {
            let format = maybe_format.unwrap_or_default();
            let format_text = if format.is_empty() {
                "No file".to_string()
            } else {
                format!("Format: {}", format)
            };

            let (bg, border, text_color) = match format.to_uppercase().as_str() {
                "LSX" => (
                    Color::rgb8(227, 242, 253),
                    Color::rgb8(33, 150, 243),
                    Color::rgb8(25, 118, 210),
                ),
                "LSJ" => (
                    Color::rgb8(243, 229, 245),
                    Color::rgb8(156, 39, 176),
                    Color::rgb8(123, 31, 162),
                ),
                "LSF" => (
                    Color::rgb8(255, 243, 224),
                    Color::rgb8(255, 152, 0),
                    Color::rgb8(245, 124, 0),
                ),
                "LOCA" => (
                    Color::rgb8(232, 245, 233),
                    Color::rgb8(76, 175, 80),
                    Color::rgb8(56, 142, 60),
                ),
                _ => (
                    Color::rgb8(240, 240, 240),
                    Color::rgb8(200, 200, 200),
                    Color::rgb8(100, 100, 100),
                ),
            };

            label(move || format_text.clone())
                .style(move |s| {
                    s.padding_horiz(12.0)
                        .padding_vert(4.0)
                        .background(bg)
                        .border(1.0)
                        .border_color(border)
                        .border_radius(4.0)
                        .color(text_color)
                        .font_weight(Weight::SEMIBOLD)
                })
                .into_any()
        },
    )
}

/// Save status badge - shown after file conversion/output (Dyes tab style)
pub fn save_status_badge(tabs_state: EditorTabsState) -> impl IntoView {
    dyn_container(
        move || tabs_state.active_tab().map(|tab| tab.save_status.get()),
        move |maybe_status| {
            let status = maybe_status.unwrap_or_default();
            if status.is_empty() {
                empty().into_any()
            } else {
                label(move || status.clone())
                    .style(|s| {
                        s.padding_horiz(12.0)
                            .padding_vert(6.0)
                            .border_radius(4.0)
                            .font_size(12.0)
                            .background(Color::rgb8(232, 245, 233)) // BG_SUCCESS
                            .color(Color::rgb8(46, 125, 50)) // TEXT_SUCCESS
                    })
                    .into_any()
            }
        },
    )
}
