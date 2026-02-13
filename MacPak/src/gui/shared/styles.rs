//! Shared styles and UI components for batch operation tabs

use floem::prelude::*;
use floem::text::Weight;

use super::BatchOperationState;

/// Standard card style used for sections in batch operation tabs.
///
/// Provides consistent styling: white background, subtle border, rounded corners.
pub fn card_style(s: floem::style::Style) -> floem::style::Style {
    s.width_full()
        .padding(16.0)
        .background(Color::WHITE)
        .border(1.0)
        .border_color(Color::rgb8(220, 220, 220))
        .border_radius(8.0)
}

/// Generic header section that works with any BatchOperationState.
///
/// Displays the tab title and a status message badge (success/error styled).
pub fn header_section<S: BatchOperationState>(title: &'static str, state: S) -> impl IntoView {
    h_stack((
        label(move || title).style(|s| s.font_size(18.0).font_weight(Weight::BOLD)),
        empty().style(|s| s.flex_grow(1.0)),
        // Status message
        dyn_container(
            move || state.status_message().get(),
            move |msg| {
                if msg.is_empty() {
                    empty().into_any()
                } else {
                    let is_error = msg.contains("Failed") || msg.contains("Error");
                    label(move || msg.clone())
                        .style(move |s| {
                            let s = s
                                .padding_horiz(12.0)
                                .padding_vert(6.0)
                                .border_radius(4.0)
                                .font_size(12.0);
                            if is_error {
                                s.background(Color::rgb8(255, 235, 235))
                                    .color(Color::rgb8(180, 30, 30))
                            } else {
                                s.background(Color::rgb8(232, 245, 233))
                                    .color(Color::rgb8(46, 125, 50))
                            }
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

/// Standard operation button style used throughout batch operation tabs.
///
/// Provides a consistent button appearance with hover effects.
pub fn operation_button(text: &'static str, on_click: impl Fn() + 'static) -> impl IntoView {
    button(text).action(on_click).style(|s| {
        s.width_full()
            .padding_vert(10.0)
            .padding_horiz(16.0)
            .background(Color::rgb8(245, 245, 245))
            .border(1.0)
            .border_color(Color::rgb8(200, 200, 200))
            .border_radius(6.0)
            .hover(|s| {
                s.background(Color::rgb8(230, 230, 230))
                    .border_color(Color::rgb8(180, 180, 180))
            })
    })
}

/// Checkbox with label for toggle options.
///
/// Clicking anywhere on the row toggles the boolean signal.
pub fn checkbox_option(label_text: &'static str, signal: RwSignal<bool>) -> impl IntoView {
    h_stack((
        checkbox(move || signal.get())
            .on_update(move |checked| signal.set(checked))
            .style(|s| s.cursor(floem::style::CursorStyle::Pointer)),
        label(move || label_text).style(|s| {
            s.font_size(12.0)
                .margin_left(6.0)
                .cursor(floem::style::CursorStyle::Pointer)
        }),
    ))
    .on_click_stop(move |_| signal.update(|v| *v = !*v))
    .style(|s| s.items_center())
}
