//! Export functionality for the Dyes tab

mod export_mod;
mod generators;

use floem::prelude::*;
use floem::text::Weight;

use crate::state::DyesState;
use super::shared::{secondary_button_style, nav_button_style, selector_display_style};

pub use export_mod::check_required_colors_at_default;
use export_mod::export_dye_mod;

/// Export section for generating mod files
pub fn export_section(state: DyesState) -> impl IntoView {
    let mod_name = state.mod_name;
    let generated_dyes = state.generated_dyes;
    let selected_index = state.selected_generated_index;
    let status = state.status_message;

    v_stack((
        // Section header with Export button
        h_stack((
            label(|| "Export")
                .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD)),
            empty().style(|s| s.flex_grow(1.0)),
            {
                let state_export = state.clone();
                let mod_name = mod_name;
                let generated_dyes = generated_dyes;
                let status = status;
                label(|| "Export Mod...")
                    .style(secondary_button_style)
                    .on_click_stop(move |_| {
                        let mod_name_str = mod_name.get();
                        if mod_name_str.is_empty() {
                            status.set("Error: Mod name is required".to_string());
                            return;
                        }

                        let dyes = generated_dyes.get();
                        if dyes.is_empty() {
                            status.set("Error: No dyes generated. Use 'Generate Dye' first.".to_string());
                            return;
                        }

                        // Open folder picker dialog
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select output folder for mod")
                            .pick_folder()
                        {
                            let message = export_dye_mod(&state_export, &path, &mod_name_str);
                            status.set(message);
                        }
                    })
            },
        ))
        .style(|s| s.width_full().items_center().gap(8.0).margin_bottom(8.0)),

        // Inner card with generated dyes list
        v_stack((
            // Generated dyes selector (only shows when there are generated dyes)
            generated_dyes_selector(state.clone()),

            // Display selected dye info
            selected_dye_display(generated_dyes, selected_index, status),
        ))
        .style(|s| {
            s.width_full()
                .padding(12.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.flex_grow(1.0)
            .flex_basis(0.0)
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
    })
}

/// Load colors from a generated dye entry into the color pickers
fn load_generated_dye_colors(state: &DyesState) {
    let dyes = state.generated_dyes.get();
    if let Some(idx) = state.selected_generated_index.get() {
        if let Some(dye) = dyes.get(idx) {
            // Load colors from the stored HashMap into the color pickers
            for (param_name, hex_color) in &dye.colors {
                match param_name.as_str() {
                    "Cloth_Primary" => state.cloth_primary.hex.set(hex_color.clone()),
                    "Cloth_Secondary" => state.cloth_secondary.hex.set(hex_color.clone()),
                    "Cloth_Tertiary" => state.cloth_tertiary.hex.set(hex_color.clone()),
                    "Leather_Primary" => state.leather_primary.hex.set(hex_color.clone()),
                    "Leather_Secondary" => state.leather_secondary.hex.set(hex_color.clone()),
                    "Leather_Tertiary" => state.leather_tertiary.hex.set(hex_color.clone()),
                    "Metal_Primary" => state.metal_primary.hex.set(hex_color.clone()),
                    "Metal_Secondary" => state.metal_secondary.hex.set(hex_color.clone()),
                    "Metal_Tertiary" => state.metal_tertiary.hex.set(hex_color.clone()),
                    "Accent_Color" => state.accent_color.hex.set(hex_color.clone()),
                    "Color_01" => state.color_01.hex.set(hex_color.clone()),
                    "Color_02" => state.color_02.hex.set(hex_color.clone()),
                    "Color_03" => state.color_03.hex.set(hex_color.clone()),
                    "Custom_1" => state.custom_1.hex.set(hex_color.clone()),
                    "Custom_2" => state.custom_2.hex.set(hex_color.clone()),
                    // Recommended colors
                    "GlowColor" => state.glow_color.hex.set(hex_color.clone()),
                    "GlowColour" => state.glow_colour.hex.set(hex_color.clone()),
                    // Common colors
                    "AddedColor" => state.added_color.hex.set(hex_color.clone()),
                    "Highlight_Color" => state.highlight_color.hex.set(hex_color.clone()),
                    "BaseColor" => state.base_color.hex.set(hex_color.clone()),
                    "InnerColor" => state.inner_color.hex.set(hex_color.clone()),
                    "OuterColor" => state.outer_color.hex.set(hex_color.clone()),
                    "PrimaryColor" => state.primary_color.hex.set(hex_color.clone()),
                    "SecondaryColor" => state.secondary_color.hex.set(hex_color.clone()),
                    "TetriaryColor" => state.tetriary_color.hex.set(hex_color.clone()),
                    "Primary" => state.primary.hex.set(hex_color.clone()),
                    "Secondary" => state.secondary.hex.set(hex_color.clone()),
                    "Tertiary" => state.tertiary.hex.set(hex_color.clone()),
                    "Primary_Color" => state.primary_color_underscore.hex.set(hex_color.clone()),
                    "Secondary_Color" => state.secondary_color_underscore.hex.set(hex_color.clone()),
                    "Tertiary_Color" => state.tertiary_color_underscore.hex.set(hex_color.clone()),
                    _ => {} // Unknown parameter, ignore
                }
            }
            state.status_message.set(format!("Loaded: {}", dye.name));
        }
    }
}

/// Generated dyes selector component
fn generated_dyes_selector(state: DyesState) -> impl IntoView {
    let state_for_selector = state.clone();
    let generated_dyes = state.generated_dyes;
    let selected_index = state.selected_generated_index;
    let status = state.status_message;

    dyn_container(
        move || generated_dyes.get().len(),
        move |count| {
            if count == 0 {
                label(|| "No dyes generated yet")
                    .style(|s| s.font_size(11.0).color(Color::rgb8(150, 150, 150)).padding(8.0))
                    .into_any()
            } else {
                let state_prev = state_for_selector.clone();
                let state_next = state_for_selector.clone();
                let state_del = state_for_selector.clone();
                let state_clear = state_for_selector.clone();

                h_stack((
                    label(|| "Dyes:").style(|s| s.font_size(11.0).font_weight(Weight::SEMIBOLD)),
                    // Navigation controls
                    h_stack((
                        // Previous button
                        {
                            let selected_index = selected_index;
                            let generated_dyes = generated_dyes;
                            label(|| "<")
                                .style(nav_button_style)
                                .on_click_stop(move |_| {
                                    let len = generated_dyes.get().len();
                                    if len > 0 {
                                        let current = selected_index.get().unwrap_or(0);
                                        let new_idx = if current == 0 { len - 1 } else { current - 1 };
                                        selected_index.set(Some(new_idx));
                                        load_generated_dye_colors(&state_prev);
                                    }
                                })
                        },
                        // Current selection display
                        {
                            let selected_index = selected_index;
                            let generated_dyes = generated_dyes;
                            label(move || {
                                let dyes = generated_dyes.get();
                                let idx = selected_index.get().unwrap_or(0);
                                if let Some(dye) = dyes.get(idx) {
                                    format!("{} ({}/{})", dye.name, idx + 1, dyes.len())
                                } else {
                                    "Select...".to_string()
                                }
                            })
                            .style(selector_display_style)
                        },
                        // Next button
                        {
                            let selected_index = selected_index;
                            let generated_dyes = generated_dyes;
                            label(|| ">")
                                .style(nav_button_style)
                                .on_click_stop(move |_| {
                                    let len = generated_dyes.get().len();
                                    if len > 0 {
                                        let current = selected_index.get().unwrap_or(0);
                                        let new_idx = (current + 1) % len;
                                        selected_index.set(Some(new_idx));
                                        load_generated_dye_colors(&state_next);
                                    }
                                })
                        },
                    ))
                    .style(|s| s.flex_grow(1.0).gap(4.0).items_center()),
                    // Update button - saves current colors to selected dye
                    {
                        let state_update = state_for_selector.clone();
                        let selected_index = selected_index;
                        let generated_dyes = generated_dyes;
                        let status = status;
                        label(|| "Update")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                let idx = selected_index.get().unwrap_or(0);
                                let mut dyes = generated_dyes.get();
                                if idx < dyes.len() {
                                    let name = dyes[idx].name.clone();
                                    dyes[idx].colors = collect_current_colors(&state_update);
                                    generated_dyes.set(dyes);
                                    status.set(format!("Updated '{}'", name));
                                }
                            })
                    },
                    // Delete selected button
                    {
                        let selected_index = selected_index;
                        let generated_dyes = generated_dyes;
                        let status = status;
                        label(|| "Delete")
                            .style(|s| {
                                secondary_button_style(s)
                                    .color(Color::rgb8(220, 38, 38))
                            })
                            .on_click_stop(move |_| {
                                let idx = selected_index.get().unwrap_or(0);
                                let mut dyes = generated_dyes.get();
                                if idx < dyes.len() {
                                    let name = dyes[idx].name.clone();
                                    dyes.remove(idx);
                                    generated_dyes.set(dyes.clone());
                                    // Adjust selection and load new colors
                                    if dyes.is_empty() {
                                        selected_index.set(None);
                                    } else if idx >= dyes.len() {
                                        selected_index.set(Some(dyes.len() - 1));
                                        load_generated_dye_colors(&state_del);
                                    } else {
                                        load_generated_dye_colors(&state_del);
                                    }
                                    status.set(format!("Deleted '{}'", name));
                                }
                            })
                    },
                    // Clear all button
                    {
                        let selected_index = selected_index;
                        let generated_dyes = generated_dyes;
                        let status = status;
                        label(|| "Clear All")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                generated_dyes.set(Vec::new());
                                selected_index.set(None);
                                // Reset colors to default
                                reset_colors_to_default(&state_clear);
                                status.set("Cleared all generated dyes".to_string());
                            })
                    },
                ))
                .style(|s| {
                    s.width_full()
                        .padding(8.0)
                        .margin_bottom(8.0)
                        .gap(8.0)
                        .items_center()
                        .background(Color::rgb8(232, 245, 233))
                        .border(1.0)
                        .border_color(Color::rgb8(129, 199, 132))
                        .border_radius(4.0)
                })
                .into_any()
            }
        },
    )
}

/// Collect current colors from the color pickers into a HashMap
fn collect_current_colors(state: &DyesState) -> std::collections::HashMap<String, String> {
    let mut colors = std::collections::HashMap::new();

    // Required colors
    colors.insert("Cloth_Primary".to_string(), state.cloth_primary.hex.get());
    colors.insert("Cloth_Secondary".to_string(), state.cloth_secondary.hex.get());
    colors.insert("Cloth_Tertiary".to_string(), state.cloth_tertiary.hex.get());
    colors.insert("Leather_Primary".to_string(), state.leather_primary.hex.get());
    colors.insert("Leather_Secondary".to_string(), state.leather_secondary.hex.get());
    colors.insert("Leather_Tertiary".to_string(), state.leather_tertiary.hex.get());
    colors.insert("Metal_Primary".to_string(), state.metal_primary.hex.get());
    colors.insert("Metal_Secondary".to_string(), state.metal_secondary.hex.get());
    colors.insert("Metal_Tertiary".to_string(), state.metal_tertiary.hex.get());
    colors.insert("Accent_Color".to_string(), state.accent_color.hex.get());
    colors.insert("Color_01".to_string(), state.color_01.hex.get());
    colors.insert("Color_02".to_string(), state.color_02.hex.get());
    colors.insert("Color_03".to_string(), state.color_03.hex.get());
    colors.insert("Custom_1".to_string(), state.custom_1.hex.get());
    colors.insert("Custom_2".to_string(), state.custom_2.hex.get());

    // Recommended colors
    colors.insert("GlowColor".to_string(), state.glow_color.hex.get());
    colors.insert("GlowColour".to_string(), state.glow_colour.hex.get());

    // Common colors
    colors.insert("AddedColor".to_string(), state.added_color.hex.get());
    colors.insert("Highlight_Color".to_string(), state.highlight_color.hex.get());
    colors.insert("BaseColor".to_string(), state.base_color.hex.get());
    colors.insert("InnerColor".to_string(), state.inner_color.hex.get());
    colors.insert("OuterColor".to_string(), state.outer_color.hex.get());
    colors.insert("PrimaryColor".to_string(), state.primary_color.hex.get());
    colors.insert("SecondaryColor".to_string(), state.secondary_color.hex.get());
    colors.insert("TetriaryColor".to_string(), state.tetriary_color.hex.get());
    colors.insert("Primary".to_string(), state.primary.hex.get());
    colors.insert("Secondary".to_string(), state.secondary.hex.get());
    colors.insert("Tertiary".to_string(), state.tertiary.hex.get());
    colors.insert("Primary_Color".to_string(), state.primary_color_underscore.hex.get());
    colors.insert("Secondary_Color".to_string(), state.secondary_color_underscore.hex.get());
    colors.insert("Tertiary_Color".to_string(), state.tertiary_color_underscore.hex.get());

    colors
}

/// Reset all colors to default gray
fn reset_colors_to_default(state: &DyesState) {
    let default = "808080".to_string();
    state.cloth_primary.hex.set(default.clone());
    state.cloth_secondary.hex.set(default.clone());
    state.cloth_tertiary.hex.set(default.clone());
    state.leather_primary.hex.set(default.clone());
    state.leather_secondary.hex.set(default.clone());
    state.leather_tertiary.hex.set(default.clone());
    state.metal_primary.hex.set(default.clone());
    state.metal_secondary.hex.set(default.clone());
    state.metal_tertiary.hex.set(default.clone());
    state.color_01.hex.set(default.clone());
    state.color_02.hex.set(default.clone());
    state.color_03.hex.set(default.clone());
    state.custom_1.hex.set(default.clone());
    state.custom_2.hex.set(default.clone());
    state.accent_color.hex.set(default.clone());
    state.glow_color.hex.set(default.clone());
    state.glow_colour.hex.set(default.clone());
    state.added_color.hex.set(default.clone());
    state.highlight_color.hex.set(default.clone());
    state.base_color.hex.set(default.clone());
    state.inner_color.hex.set(default.clone());
    state.outer_color.hex.set(default.clone());
    state.primary_color.hex.set(default.clone());
    state.secondary_color.hex.set(default.clone());
    state.tetriary_color.hex.set(default.clone());
    state.primary.hex.set(default.clone());
    state.secondary.hex.set(default.clone());
    state.tertiary.hex.set(default.clone());
    state.primary_color_underscore.hex.set(default.clone());
    state.secondary_color_underscore.hex.set(default.clone());
    state.tertiary_color_underscore.hex.set(default);
}

/// Display selected dye information with editable name
fn selected_dye_display(
    generated_dyes: RwSignal<Vec<crate::state::GeneratedDyeEntry>>,
    selected_index: RwSignal<Option<usize>>,
    status: RwSignal<String>,
) -> impl IntoView {
    // Create local signals for display values
    let edit_name: RwSignal<String> = RwSignal::new(String::new());
    let display_uuid: RwSignal<String> = RwSignal::new(String::new());

    // Sync display values when selection or dyes change
    let _ = floem::reactive::create_effect(move |prev_idx: Option<Option<usize>>| {
        let dyes = generated_dyes.get();
        let idx = selected_index.get();

        if let Some(i) = idx {
            if let Some(dye) = dyes.get(i) {
                // Only update edit_name if selection changed (to avoid overwriting user edits)
                if prev_idx != Some(idx) {
                    edit_name.set(dye.name.clone());
                }
                // Always update UUID (in case dye was just generated or data changed)
                display_uuid.set(dye.preset_uuid.clone());
            }
        }
        idx
    });

    // Track only whether we have a selection (boolean), not the full entry
    dyn_container(
        move || selected_index.get().is_some() && !generated_dyes.get().is_empty(),
        move |has_selection| {
            if has_selection {
                v_stack((
                    // Dye Name row - editable (stable, won't re-render on typing)
                    h_stack((
                        label(|| "Dye Name")
                            .style(|s| s.width(90.0).font_size(11.0)),
                        text_input(edit_name)
                            .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                                // Update the name in generated_dyes when focus is lost
                                let new_name = edit_name.get();
                                let idx = selected_index.get().unwrap_or(0);
                                let mut dyes = generated_dyes.get();
                                if idx < dyes.len() && !new_name.is_empty() {
                                    dyes[idx].name = new_name.clone();
                                    generated_dyes.set(dyes);
                                    status.set(format!("Renamed to '{}'", new_name));
                                }
                            })
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .flex_basis(0.0)
                                    .width_full()
                                    .min_width(100.0)
                                    .padding(6.0)
                                    .font_size(11.0)
                                    .font_family("monospace".to_string())
                                    .background(Color::WHITE)
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                            }),
                    ))
                    .style(|s| s.width_full().items_center().gap(8.0)),

                    // Dye UUID row
                    h_stack((
                        label(|| "Dye UUID")
                            .style(|s| s.width(90.0).font_size(11.0)),
                        label(move || display_uuid.get())
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .padding(6.0)
                                    .font_size(11.0)
                                    .font_family("monospace".to_string())
                                    .background(Color::rgb8(245, 245, 245))
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                            }),
                    ))
                    .style(|s| s.width_full().items_center().gap(8.0)),

                    // Mod UUID row - greyed out, will be generated at export
                    h_stack((
                        label(|| "Mod UUID")
                            .style(|s| s.width(90.0).font_size(11.0).color(Color::rgb8(150, 150, 150))),
                        label(|| "Will be generated at export")
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .padding(6.0)
                                    .font_size(11.0)
                                    .color(Color::rgb8(150, 150, 150))
                                    .background(Color::rgb8(245, 245, 245))
                                    .border(1.0)
                                    .border_color(Color::rgb8(200, 200, 200))
                                    .border_radius(4.0)
                            }),
                    ))
                    .style(|s| s.width_full().items_center().gap(8.0)),
                ))
                .style(|s| s.width_full().gap(8.0))
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
}
