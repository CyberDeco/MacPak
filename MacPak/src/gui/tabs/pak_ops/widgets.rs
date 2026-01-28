//! Reusable UI widgets for PAK operations

use floem::prelude::*;

use crate::gui::state::PakCompression;

/// Compression option button
fn compression_button(compression: RwSignal<PakCompression>, opt: PakCompression) -> impl IntoView {
    let is_selected = move || compression.get() == opt;

    button(opt.as_str())
        .action(move || {
            compression.set(opt);
        })
        .style(move |s| {
            let mut s = s
                .padding_vert(6.0)
                .padding_horiz(10.0)
                .font_size(12.0)
                .border_radius(4.0);

            if is_selected() {
                s = s.background(Color::rgb8(33, 150, 243)).color(Color::WHITE);
            } else {
                s = s
                    .background(Color::rgb8(240, 240, 240))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .hover(|s| s.background(Color::rgb8(230, 230, 230)));
            }
            s
        })
}

/// Compression selector with all options
pub fn compression_selector(compression: RwSignal<PakCompression>) -> impl IntoView {
    h_stack((
        compression_button(compression, PakCompression::Lz4Hc),
        compression_button(compression, PakCompression::Lz4),
        compression_button(compression, PakCompression::None),
    ))
    .style(|s| s.gap(4.0))
}

/// Priority input with increment/decrement buttons
pub fn priority_input(priority: RwSignal<i32>) -> impl IntoView {
    h_stack((
        button("-")
            .action(move || {
                let val = priority.get();
                if val > 0 {
                    priority.set(val - 1);
                }
            })
            .style(|s| {
                s.width(30.0)
                    .height(30.0)
                    .items_center()
                    .justify_center()
                    .background(Color::rgb8(240, 240, 240))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
        label(move || format!("{}", priority.get())).style(|s| {
            s.width(50.0)
                .height(30.0)
                .items_center()
                .justify_center()
                .background(Color::WHITE)
                .border(1.0)
                .border_color(Color::rgb8(200, 200, 200))
        }),
        button("+")
            .action(move || {
                let val = priority.get();
                if val < 100 {
                    priority.set(val + 1);
                }
            })
            .style(|s| {
                s.width(30.0)
                    .height(30.0)
                    .items_center()
                    .justify_center()
                    .background(Color::rgb8(240, 240, 240))
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
        label(|| "(0-100)".to_string()).style(|s| {
            s.font_size(11.0)
                .color(Color::rgb8(120, 120, 120))
                .margin_left(8.0)
        }),
    ))
    .style(|s| s.items_center())
}
