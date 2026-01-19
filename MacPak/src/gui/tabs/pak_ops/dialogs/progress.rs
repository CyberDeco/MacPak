//! Progress dialog content

use floem::prelude::*;

pub fn progress_content(
    polled_pct: RwSignal<u32>,
    polled_current: RwSignal<u32>,
    polled_total: RwSignal<u32>,
    polled_msg: RwSignal<String>,
) -> impl IntoView {
    container(
        v_stack((
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
            label(move || polled_msg.get())
                .style(|s| s.font_size(14.0).margin_bottom(12.0)),
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
}
