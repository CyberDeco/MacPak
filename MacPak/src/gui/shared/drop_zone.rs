//! Shared drop zone component for drag-and-drop file operations

use floem::event::{Event, EventListener};
use floem::prelude::*;

/// Generic drop zone with emoji, hint text, and event handler callback.
///
/// If `stop_propagation` is true, uses `on_event_stop` (prevents further event
/// handling); otherwise uses `on_event_cont` (continues event propagation).
pub fn drop_zone(
    emoji: &'static str,
    hint: &'static str,
    stop_propagation: bool,
    on_drop: impl Fn(&Event) + 'static,
) -> impl IntoView {
    let view = container(
        v_stack((
            label(move || emoji.to_string()).style(|s| s.font_size(32.0)),
            label(|| "Drag files here".to_string()).style(|s| {
                s.font_size(14.0)
                    .color(Color::rgb8(100, 100, 100))
                    .margin_top(8.0)
            }),
            label(move || hint.to_string())
                .style(|s| s.font_size(12.0).color(Color::rgb8(150, 150, 150))),
        ))
        .style(|s| s.items_center()),
    );

    let view = if stop_propagation {
        view.on_event_stop(EventListener::DroppedFile, move |e| on_drop(e))
    } else {
        view.on_event_cont(EventListener::DroppedFile, move |e| on_drop(e))
    };

    view.style(|s| {
        s.flex_grow(2.0)
            .min_height(120.0)
            .padding(16.0)
            .items_center()
            .justify_center()
            .background(Color::rgb8(249, 249, 249))
            .border(2.0)
            .border_color(Color::rgb8(204, 204, 204))
            .border_radius(8.0)
    })
}
