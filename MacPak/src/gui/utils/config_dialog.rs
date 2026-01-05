//! Configuration Dialog UI Component
//!
//! A dialog for configuring MacPak settings.

use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem::text::Weight;
use floem::views::PlaceholderTextClass;

use crate::gui::state::ConfigState;

/// Create the configuration dialog UI
pub fn config_dialog(config_state: ConfigState) -> impl IntoView {
    let show = config_state.show_dialog;
    let bg3_path = config_state.bg3_data_path;
    let path_warning = config_state.path_warning;
    let config_for_save = config_state.clone();

    // Local edit signal (so we can cancel without saving)
    let edit_path = RwSignal::new(bg3_path.get());

    // Sync when dialog opens
    let show_for_sync = show;
    let bg3_path_for_sync = bg3_path;
    let edit_path_for_sync = edit_path;

    dyn_container(
        move || show_for_sync.get(),
        move |visible| {
            if !visible {
                return empty().into_any();
            }

            // Sync edit_path with current bg3_path when dialog opens
            edit_path_for_sync.set(bg3_path_for_sync.get());

            let config_for_save = config_for_save.clone();

            // Dialog content
            v_stack((
                // Header
                label(|| "Preferences")
                    .style(|s| s.font_size(18.0).font_weight(Weight::BOLD).margin_bottom(16.0)),

                // Warning message if path is invalid
                dyn_container(
                    move || path_warning.get(),
                    move |warning| {
                        if let Some(msg) = warning {
                            label(move || msg.clone())
                                .style(|s| {
                                    s.width_full()
                                        .padding(8.0)
                                        .margin_bottom(12.0)
                                        .background(Color::rgb8(255, 243, 205))
                                        .border(1.0)
                                        .border_color(Color::rgb8(255, 193, 7))
                                        .border_radius(4.0)
                                        .color(Color::rgb8(133, 100, 4))
                                        .font_size(12.0)
                                })
                                .into_any()
                        } else {
                            empty().into_any()
                        }
                    }
                ),

                // BG3 Path field
                v_stack((
                    label(|| "Baldur's Gate 3 Data Path")
                        .style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                    h_stack((
                        {
                            let show_for_input = show;
                            text_input(edit_path)
                                .placeholder("Path to BG3 Data folder...")
                                .style(|s| {
                                    s.flex_grow(1.0)
                                        .flex_basis(0.0)
                                        .width_full()
                                        .min_width(100.0)
                                        .padding(8.0)
                                        .font_size(13.0)
                                        .background(Color::WHITE)
                                        .border(1.0)
                                        .border_color(Color::rgb8(200, 200, 200))
                                        .border_radius(4.0)
                                        .class(PlaceholderTextClass, |s| s.color(Color::rgb8(120, 120, 120)))
                                })
                                .on_event_cont(EventListener::KeyDown, move |e| {
                                    if let Event::KeyDown(key_event) = e {
                                        // ESC closes dialog
                                        if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                                            show_for_input.set(false);
                                        }
                                        // CMD+, closes dialog
                                        let is_cmd = key_event.modifiers.contains(floem::keyboard::Modifiers::META)
                                            || key_event.modifiers.contains(floem::keyboard::Modifiers::CONTROL);
                                        let is_comma = matches!(
                                            &key_event.key.logical_key,
                                            Key::Character(c) if c.as_str() == ","
                                        );
                                        if is_cmd && is_comma {
                                            show_for_input.set(false);
                                        }
                                    }
                                })
                        },
                        {
                            let edit_path = edit_path;
                            button("Browse...")
                                .style(|s| s.margin_left(8.0).flex_shrink(0.0))
                                .action(move || {
                                    if let Some(folder) = rfd::FileDialog::new()
                                        .set_title("Select BG3 Data Folder")
                                        .pick_folder()
                                    {
                                        edit_path.set(folder.display().to_string());
                                    }
                                })
                        },
                    ))
                    .style(|s| s.width_full().items_center()),
                ))
                .style(|s| s.width_full().gap(4.0)),

                // Help text
                label(|| "This should point to the Data folder containing .pak files")
                    .style(|s| {
                        s.font_size(11.0)
                            .color(Color::rgb8(128, 128, 128))
                            .margin_top(4.0)
                    }),

                // Buttons
                h_stack((
                    {
                        let show = show;
                        button("Cancel")
                            .style(|s| {
                                s.padding(8.0)
                                    .padding_horiz(16.0)
                                    .background(Color::rgb8(240, 240, 240))
                                    .border_radius(4.0)
                            })
                            .action(move || {
                                show.set(false);
                            })
                    },
                    {
                        let show = show;
                        let bg3_path = bg3_path;
                        let edit_path = edit_path;
                        button("Save")
                            .style(|s| {
                                s.padding(8.0)
                                    .padding_horiz(16.0)
                                    .background(Color::rgb8(59, 130, 246))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                                    .font_weight(Weight::SEMIBOLD)
                            })
                            .action(move || {
                                // Save the edited path
                                bg3_path.set(edit_path.get());
                                // Validate the new path
                                config_for_save.validate_path();
                                show.set(false);
                            })
                    },
                ))
                .style(|s| s.width_full().justify_end().gap(8.0).margin_top(16.0)),
            ))
            .style(|s| {
                s.width(600.0)
                    .padding(24.0)
                    .background(Color::WHITE)
                    .border_radius(8.0)
                    .box_shadow_blur(20.0)
                    .box_shadow_color(Color::rgba8(0, 0, 0, 50))
            })
            .into_any()
        },
    )
    .style(move |s| {
        if show.get() {
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
        // Close dialog on Escape or CMD+,
        if let Event::KeyDown(key_event) = e {
            // Escape key
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                show.set(false);
            }
            // CMD+, (Preferences shortcut toggles/closes)
            let is_cmd = key_event.modifiers.contains(floem::keyboard::Modifiers::META)
                || key_event.modifiers.contains(floem::keyboard::Modifiers::CONTROL);
            let is_comma = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str() == ","
            );
            if is_cmd && is_comma {
                show.set(false);
            }
        }
    })
    .keyboard_navigable()
}
