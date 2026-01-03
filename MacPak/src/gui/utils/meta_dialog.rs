//! Meta.lsx Dialog UI Component
//!
//! A reusable dialog for generating meta.lsx files.

use floem::prelude::*;
use floem::text::Weight;
use floem::views::PlaceholderTextClass;

use super::{meta_generator, generate_uuid, UuidFormat};

/// Convert a string to snake_case format
fn to_snake_case(s: &str) -> String {
    let result: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    // Collapse multiple underscores and trim
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_underscore = true;
    for c in result.chars() {
        if c == '_' {
            if !prev_underscore {
                collapsed.push('_');
            }
            prev_underscore = true;
        } else {
            collapsed.push(c);
            prev_underscore = false;
        }
    }
    if collapsed.ends_with('_') {
        collapsed.pop();
    }
    collapsed
}

/// External signals for meta dialog (for integration with external state)
pub struct MetaDialogSignals {
    pub mod_name: RwSignal<String>,
    pub author: RwSignal<String>,
    pub description: RwSignal<String>,
    pub uuid: RwSignal<String>,
    pub version_major: RwSignal<u32>,
    pub version_minor: RwSignal<u32>,
    pub version_patch: RwSignal<u32>,
    pub version_build: RwSignal<u32>,
}

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
    let mod_name = RwSignal::new(prefill_name.unwrap_or_default());
    let author = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let uuid = RwSignal::new(generate_uuid(UuidFormat::Standard)); // Pre-generate UUID
    let version_major = RwSignal::new(1u32);
    let version_minor = RwSignal::new(0u32);
    let version_patch = RwSignal::new(0u32);
    let version_build = RwSignal::new(0u32);

    meta_dialog_impl(show, mod_name, author, description, uuid,
        version_major, version_minor, version_patch, version_build,
        on_create, status_message, "Generate meta.lsx", "Create", None::<fn() -> Box<dyn View>>)
}

/// Create the meta.lsx dialog UI with external signals (for dyes export)
pub fn meta_dialog_with_signals<F>(
    show: RwSignal<bool>,
    signals: MetaDialogSignals,
    on_create: F,
    status_message: Option<RwSignal<String>>,
    title: &'static str,
    button_text: &'static str,
) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
{
    // Pre-generate UUID if empty
    if signals.uuid.get().is_empty() {
        signals.uuid.set(generate_uuid(UuidFormat::Standard));
    }

    meta_dialog_impl(show, signals.mod_name, signals.author, signals.description,
        signals.uuid, signals.version_major, signals.version_minor,
        signals.version_patch, signals.version_build,
        on_create, status_message, title, button_text, None::<fn() -> Box<dyn View>>)
}

/// Create the meta.lsx dialog UI with external signals and extra content (for dyes export with vendor selection)
pub fn meta_dialog_with_signals_and_extra<F, E, V>(
    show: RwSignal<bool>,
    signals: MetaDialogSignals,
    on_create: F,
    status_message: Option<RwSignal<String>>,
    title: &'static str,
    button_text: &'static str,
    extra_content: E,
) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
    E: Fn() -> V + Clone + 'static,
    V: IntoView + 'static,
{
    // Pre-generate UUID if empty
    if signals.uuid.get().is_empty() {
        signals.uuid.set(generate_uuid(UuidFormat::Standard));
    }

    meta_dialog_impl(show, signals.mod_name, signals.author, signals.description,
        signals.uuid, signals.version_major, signals.version_minor,
        signals.version_patch, signals.version_build,
        on_create, status_message, title, button_text, Some(extra_content))
}

/// Implementation of meta dialog with configurable signals
fn meta_dialog_impl<F, E, V>(
    show: RwSignal<bool>,
    mod_name: RwSignal<String>,
    author: RwSignal<String>,
    description: RwSignal<String>,
    uuid: RwSignal<String>,
    version_major: RwSignal<u32>,
    version_minor: RwSignal<u32>,
    version_patch: RwSignal<u32>,
    version_build: RwSignal<u32>,
    on_create: F,
    status_message: Option<RwSignal<String>>,
    title: &'static str,
    button_text: &'static str,
    extra_content: Option<E>,
) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
    E: Fn() -> V + Clone + 'static,
    V: IntoView + 'static,
{
    let on_create_clone = on_create.clone();
    let extra_content_clone = extra_content.clone();

    dyn_container(
        move || show.get(),
        move |visible| {
            if !visible {
                return empty().into_any();
            }

            let on_create = on_create_clone.clone();
            let status = status_message;
            let extra = extra_content_clone.clone();

            // Build the form fields
            let form_fields = v_stack((
                // Row 1: Mod Name and Author
                h_stack((
                    meta_text_field("Mod Name", mod_name, "My Custom Mod..."),
                    meta_text_field("Author", author, "Your Name (e.g., Nexus Mods username)..."),
                ))
                .style(|s| s.width_full().gap(12.0)),

                // Row 2: Description (full width)
                meta_text_field("Description", description, "A short description..."),

                // Row 3: UUID and Version on same row
                h_stack((
                    // UUID with regenerate button
                    v_stack((
                        label(|| "UUID").style(|s| s.font_size(12.0).color(Color::rgb8(100, 100, 100))),
                        h_stack((
                            text_input(uuid)
                                .style(|s| {
                                    s.width_full()
                                        .flex_grow(1.0)
                                        .padding(8.0)
                                        .border(1.0)
                                        .border_color(Color::rgb8(200, 200, 200))
                                        .border_radius(4.0)
                                        .font_family("monospace".to_string())
                                }),
                            {
                                let uuid = uuid;
                                button("Regenerate")
                                    .style(|s| s.margin_left(8.0))
                                    .action(move || {
                                        uuid.set(generate_uuid(UuidFormat::Standard));
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
            .style(|s| s.width_full().gap(12.0));

            // Build buttons
            let buttons = h_stack((
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
                button(button_text)
                    .style(|s| {
                        s.padding(8.0)
                            .padding_horiz(16.0)
                            .background(Color::rgb8(59, 130, 246))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .font_weight(Weight::SEMIBOLD)
                    })
                    .action(move || {
                        // Generate folder as snake_case of mod name
                        let folder = to_snake_case(&mod_name.get());

                        // Generate meta.lsx content
                        let content = meta_generator::generate_meta_lsx(
                            &mod_name.get(),
                            &folder,
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
                            status.set("Exported mod".to_string());
                        }
                    }),
            ))
            .style(|s| s.width_full().justify_end().gap(8.0).margin_top(16.0));

            // Build dialog with optional extra content
            let has_extra = extra.is_some();
            let dialog_width = if has_extra { 700.0 } else { 500.0 };

            // Dialog box
            v_stack((
                // Header
                label(move || title)
                    .style(|s| s.font_size(18.0).font_weight(Weight::BOLD).margin_bottom(16.0)),
                form_fields,
                // Extra content (if provided)
                dyn_container(
                    move || has_extra,
                    {
                        let extra = extra.clone();
                        move |show_extra| {
                            if show_extra {
                                if let Some(ref content_fn) = extra {
                                    content_fn().into_any()
                                } else {
                                    empty().into_any()
                                }
                            } else {
                                empty().into_any()
                            }
                        }
                    }
                ),
                buttons,
            ))
            .style(move |s| {
                s.width(dialog_width)
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
                    .class(PlaceholderTextClass, |s| s.color(Color::rgb8(120, 120, 120)))
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
                .class(PlaceholderTextClass, |s| s.color(Color::rgb8(120, 120, 120)))
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
