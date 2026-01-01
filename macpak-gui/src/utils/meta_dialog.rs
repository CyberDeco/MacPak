//! Meta.lsx Dialog UI Component
//!
//! A reusable dialog for generating meta.lsx files.

use floem::prelude::*;
use floem::text::Weight;

use super::{meta_generator, generate_uuid, UuidFormat};

/// Create the meta.lsx dialog UI with its own internal state
///
/// # Arguments
/// * `show` - Signal controlling dialog visibility
/// * `prefill_name` - Optional prefill value for mod name/folder
/// * `on_create` - Callback when Create is clicked, receives the generated content
/// * `status_message` - Optional status message signal to update
pub fn meta_dialog<F>(
    show: RwSignal<bool>,
    prefill_name: Option<String>,
    on_create: F,
    status_message: Option<RwSignal<String>>,
) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
{
    // Internal state for the dialog form
    let mod_name = RwSignal::new(prefill_name.clone().unwrap_or_default());
    let folder = RwSignal::new(prefill_name.unwrap_or_default());
    let author = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let uuid = RwSignal::new(String::new());
    let version_major = RwSignal::new(1u32);
    let version_minor = RwSignal::new(0u32);
    let version_patch = RwSignal::new(0u32);
    let version_build = RwSignal::new(0u32);

    let on_create_clone = on_create.clone();

    dyn_container(
        move || show.get(),
        move |visible| {
            if !visible {
                return empty().into_any();
            }

            let on_create = on_create_clone.clone();
            let status = status_message;

            // Dialog box
            v_stack((
                    // Header
                    label(|| "Generate meta.lsx")
                        .style(|s| s.font_size(18.0).font_weight(Weight::BOLD).margin_bottom(16.0)),

                    // Form fields
                    v_stack((
                        // Row 1: Mod Name and Folder
                        h_stack((
                            meta_text_field("Mod Name", mod_name, "My Awesome Mod"),
                            meta_text_field("Folder", folder, "MyAwesomeMod"),
                        ))
                        .style(|s| s.width_full().gap(12.0)),

                        // Row 2: Author and Description
                        h_stack((
                            meta_text_field("Author", author, "Your Name"),
                            meta_text_field("Description", description, "A short description..."),
                        ))
                        .style(|s| s.width_full().gap(12.0)),

                        // Row 3: UUID with generate button
                        h_stack((
                            v_stack((
                                label(|| "UUID").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                                h_stack((
                                    text_input(uuid)
                                        .placeholder("Click Generate")
                                        .style(|s| {
                                            s.flex_grow(1.0)
                                                .padding(8.0)
                                                .border(1.0)
                                                .border_color(Color::rgb8(200, 200, 200))
                                                .border_radius(4.0)
                                                .font_family("monospace".to_string())
                                        }),
                                    {
                                        let uuid = uuid;
                                        button("Generate")
                                            .style(|s| s.margin_left(8.0))
                                            .action(move || {
                                                uuid.set(generate_uuid(UuidFormat::Larian));
                                            })
                                    },
                                ))
                                .style(|s| s.width_full().items_center()),
                            ))
                            .style(|s| s.flex_grow(1.0).gap(4.0)),

                            // Version fields
                            v_stack((
                                label(|| "Version").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                                h_stack((
                                    meta_version_field(version_major, "Major"),
                                    label(|| ".").style(|s| s.font_size(16.0).margin_horiz(2.0)),
                                    meta_version_field(version_minor, "Minor"),
                                    label(|| ".").style(|s| s.font_size(16.0).margin_horiz(2.0)),
                                    meta_version_field(version_patch, "Patch"),
                                    label(|| ".").style(|s| s.font_size(16.0).margin_horiz(2.0)),
                                    meta_version_field(version_build, "Build"),
                                ))
                                .style(|s| s.items_center()),
                            ))
                            .style(|s| s.gap(4.0)),
                        ))
                        .style(|s| s.width_full().gap(12.0)),
                    ))
                    .style(|s| s.width_full().gap(12.0)),

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
                        button("Create")
                            .style(|s| {
                                s.padding(8.0)
                                    .padding_horiz(16.0)
                                    .background(Color::rgb8(59, 130, 246))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                                    .font_weight(Weight::SEMIBOLD)
                            })
                            .action(move || {
                                // Generate meta.lsx content
                                let content = meta_generator::generate_meta_lsx(
                                    &mod_name.get(),
                                    &folder.get(),
                                    &author.get(),
                                    &description.get(),
                                    &uuid.get(),
                                    version_major.get(),
                                    version_minor.get(),
                                    version_patch.get(),
                                    version_build.get(),
                                );

                                // Call the callback with generated content
                                on_create(content);

                                // Close dialog
                                show.set(false);

                                // Update status if provided
                                if let Some(status) = status {
                                    status.set("Generated meta.lsx".to_string());
                                }
                            }),
                    ))
                    .style(|s| s.width_full().justify_end().gap(8.0).margin_top(16.0)),
            ))
            .style(|s| {
                s.width(500.0)
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
}

fn meta_text_field(
    label_text: &'static str,
    signal: RwSignal<String>,
    placeholder: &'static str,
) -> impl IntoView {
    v_stack((
        label(move || label_text).style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
        text_input(signal)
            .placeholder(placeholder)
            .style(|s| {
                s.width_full()
                    .padding(8.0)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(4.0)
            }),
    ))
    .style(|s| s.flex_grow(1.0).gap(4.0))
}

fn meta_version_field(signal: RwSignal<u32>, placeholder: &'static str) -> impl IntoView {
    let text_signal = RwSignal::new(signal.get().to_string());

    text_input(text_signal)
        .placeholder(placeholder)
        .style(|s| {
            s.width(50.0)
                .padding(8.0)
                .border(1.0)
                .border_color(Color::rgb8(200, 200, 200))
                .border_radius(4.0)
        })
        .on_event(floem::event::EventListener::FocusLost, move |_| {
            // Parse and update the u32 signal when focus is lost
            if let Ok(value) = text_signal.get().parse::<u32>() {
                signal.set(value);
            } else {
                // Reset to current value if parse fails
                text_signal.set(signal.get().to_string());
            }
            floem::event::EventPropagation::Continue
        })
}
