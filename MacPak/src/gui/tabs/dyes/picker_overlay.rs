//! Color picker overlay dialog
//!
//! Shows a `solid_picker` widget in a modal overlay, following the same
//! pattern as the meta.lsx export dialog in `meta_dialog.rs`.

use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem::text::Weight;
use floem_picker::{SolidColor, solid_picker};
use floem_reactive::create_effect;

use super::shared::constants::*;
use crate::gui::state::DyesState;

/// Creates the color picker overlay.
///
/// When `active_picker_color` is `Some(name)`, the overlay is visible with
/// the picker pre-loaded to that color row's hex value. Picking a color
/// updates the hex signal in real-time. "Done" or Escape dismisses it.
pub fn color_picker_overlay(state: DyesState) -> impl IntoView {
    let active = state.active_picker_color;
    let picker_color = state.picker_color;

    // --- Effect: when the active color name changes, seed the picker ---
    // Subscribes to `active` only; reads hex untracked to avoid a cycle.
    let state_for_open = state.clone();
    create_effect(move |_| {
        if let Some(name) = active.get() {
            if let Some(hex_signal) = state_for_open.color_hex(name) {
                let hex = hex_signal.get_untracked();
                if let Some(c) = SolidColor::from_hex(&hex) {
                    // Always open with brightness at 100%
                    let (h, s, _) = c.to_hsb();
                    picker_color.set(SolidColor::from_hsb(h, s, 1.0, 1.0));
                }
            }
        }
    });

    // --- Effect: when the picker color changes, push back to the hex signal ---
    // Subscribes to `picker_color` only; reads active untracked to avoid a cycle.
    let state_for_sync = state.clone();
    create_effect(move |prev: Option<String>| {
        // Truncate to 6 chars (RGB only) — BG3 dyes are opaque
        let full_hex = picker_color.get().to_hex();
        let new_hex = full_hex[..6.min(full_hex.len())].to_string();
        if prev.as_ref() != Some(&new_hex) {
            if let Some(name) = active.get_untracked() {
                if let Some(hex_signal) = state_for_sync.color_hex(name) {
                    hex_signal.set(new_hex.clone());
                }
            }
        }
        new_hex
    });

    // --- UI ---
    dyn_container(
        move || active.get(),
        move |visible_name| {
            if visible_name.is_none() {
                return empty().into_any();
            }
            let name = visible_name.unwrap();

            v_stack((
                // Header row: close button + color name
                h_stack((
                    h_stack((label(|| "\u{2716}").style(|s| {
                        s.font_size(16.0)
                            .line_height(16.0)
                            .color(Color::rgb8(255, 92, 95))
                            .hover(|s| s.color(Color::rgb8(128, 47, 48)))
                    }),))
                    .style(|s| {
                        s.width(16.0)
                            .height(16.0)
                            .border_radius(8.0)
                            .background(Color::rgb8(255, 92, 95))
                            .justify_center()
                            .items_center()
                            .cursor(floem::style::CursorStyle::Pointer)
                    })
                    .on_click_stop(move |_| {
                        active.set(None);
                    }),
                    label(move || name).style(|s| {
                        s.font_size(FONT_HEADER)
                            .font_weight(Weight::SEMIBOLD)
                            .margin_left(PADDING_STD)
                    }),
                ))
                .style(|s| s.width_full().items_center().margin_bottom(PADDING_STD)),
                // The picker widget
                solid_picker(picker_color),
            ))
            .style(|s| {
                s.width(320.0)
                    .min_height(500.0)
                    .padding(12.0)
                    .background(Color::WHITE)
                    .border_radius(8.0)
                    .box_shadow_blur(20.0)
                    .box_shadow_color(Color::rgba8(0, 0, 0, 50))
            })
            .into_any()
        },
    )
    .style(move |s| {
        if active.get().is_some() {
            s.position(floem::style::Position::Absolute)
                .inset_top(0.0)
                .inset_left(0.0)
                .inset_bottom(0.0)
                .inset_right(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 100))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
    .on_event_stop(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(key_event) = e {
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                active.set(None);
            }
        }
    })
    .keyboard_navigable()
}
