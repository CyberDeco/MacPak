//! Handle Generator

use floem::prelude::*;
use floem::text::Weight;
use rand::Rng;

use crate::state::ToolsState;
use super::helpers::copy_to_clipboard;
use super::widgets::tool_card_style;

pub fn handle_section(state: ToolsState) -> impl IntoView {
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

fn generate_handle() -> String {
    let handle: u64 = rand::thread_rng().gen();
    handle.to_string()
}
