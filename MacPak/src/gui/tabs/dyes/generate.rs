//! Generate Dye section - create new dyes from current color settings

use std::collections::HashMap;

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::{DyesState, GeneratedDyeEntry};
use crate::gui::utils::{generate_uuid, UuidFormat};
use super::export::check_required_colors_at_default;
use super::shared::{button_style, input_style};

/// Generate Dye section for creating new dye entries
pub fn generate_dye_section(state: DyesState) -> impl IntoView {
    let individual_dye_name = state.individual_dye_name;
    let generated_dyes = state.generated_dyes;
    let status = state.status_message;

    v_stack((
        // Section header
        label(|| "Generate Dye")
            .style(|s| s.font_size(14.0).font_weight(Weight::SEMIBOLD).margin_bottom(8.0)),

        // Inner card
        v_stack((
            // Dye Name row
            h_stack((
                label(|| "Dye Name")
                    .style(|s| s.width(80.0).font_size(11.0)),
                text_input(individual_dye_name)
                    .placeholder("e.g. MyMod_Dye_Crimson")
                    .style(input_style),
            ))
            .style(|s| s.width_full().items_center().gap(8.0)),

            // Generate button
            h_stack((
                {
                    let state_gen = state.clone();
                    let dye_name = individual_dye_name;
                    let generated_dyes = generated_dyes;
                    let status = status;
                    label(|| "Generate Dye")
                        .style(|s| {
                            button_style(s)
                                .background(Color::rgb8(76, 175, 80))
                                .color(Color::WHITE)
                                .font_weight(Weight::SEMIBOLD)
                        })
                        .on_click_stop(move |_| {
                            let name = dye_name.get();
                            if name.is_empty() {
                                status.set("Error: Dye name is required".to_string());
                                return;
                            }

                            // Check for required colors at default
                            let defaults = check_required_colors_at_default(&state_gen);
                            if !defaults.is_empty() {
                                let msg = format!(
                                    "The following required colors are still at default (#808080):\n\n{}\n\nPlease set these colors before generating.",
                                    defaults.join(", ")
                                );
                                rfd::MessageDialog::new()
                                    .set_title("Required Colors Missing")
                                    .set_description(&msg)
                                    .set_level(rfd::MessageLevel::Warning)
                                    .show();
                                return;
                            }

                            // Generate UUIDs
                            let preset_uuid = generate_uuid(UuidFormat::Larian);
                            let template_uuid = generate_uuid(UuidFormat::Larian);
                            let name_handle = generate_uuid(UuidFormat::Larian);
                            let desc_handle = generate_uuid(UuidFormat::Larian);

                            // Collect current colors
                            let colors = collect_colors(&state_gen);

                            // Create new dye entry
                            let entry = GeneratedDyeEntry {
                                name: name.clone(),
                                preset_uuid,
                                template_uuid,
                                name_handle,
                                desc_handle,
                                colors,
                            };

                            // Add to list and select the new entry
                            generated_dyes.update(|dyes| {
                                dyes.push(entry);
                            });
                            let count = generated_dyes.get().len();
                            state_gen.selected_generated_index.set(Some(count - 1));

                            // Clear the input and show success
                            dye_name.set(String::new());
                            status.set(format!("Generated dye '{}' ({} total)", name, count));
                        })
                },
            ))
            .style(|s| s.width_full().margin_top(8.0).gap(8.0)),
        ))
        .style(|s| {
            s.width_full()
                .padding(12.0)
                .gap(8.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_full()
            .padding(16.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
    })
}

/// Collect all color values from state into a HashMap
fn collect_colors(state: &DyesState) -> HashMap<String, String> {
    let mut colors = HashMap::new();
    let default = "808080";

    // Required colors (always included)
    colors.insert("Cloth_Primary".to_string(), state.cloth_primary.hex.get());
    colors.insert("Cloth_Secondary".to_string(), state.cloth_secondary.hex.get());
    colors.insert("Cloth_Tertiary".to_string(), state.cloth_tertiary.hex.get());
    colors.insert("Leather_Primary".to_string(), state.leather_primary.hex.get());
    colors.insert("Leather_Secondary".to_string(), state.leather_secondary.hex.get());
    colors.insert("Leather_Tertiary".to_string(), state.leather_tertiary.hex.get());
    colors.insert("Metal_Primary".to_string(), state.metal_primary.hex.get());
    colors.insert("Metal_Secondary".to_string(), state.metal_secondary.hex.get());
    colors.insert("Metal_Tertiary".to_string(), state.metal_tertiary.hex.get());
    colors.insert("Color_01".to_string(), state.color_01.hex.get());
    colors.insert("Color_02".to_string(), state.color_02.hex.get());
    colors.insert("Color_03".to_string(), state.color_03.hex.get());
    colors.insert("Custom_1".to_string(), state.custom_1.hex.get());
    colors.insert("Custom_2".to_string(), state.custom_2.hex.get());

    // Helper to add color only if not default
    let mut add_if_not_default = |name: &str, hex: String| {
        if hex.to_lowercase() != default {
            colors.insert(name.to_string(), hex);
        }
    };

    // Recommended colors (only if not default)
    add_if_not_default("Accent_Color", state.accent_color.hex.get());
    add_if_not_default("GlowColor", state.glow_color.hex.get());
    add_if_not_default("GlowColour", state.glow_colour.hex.get());

    // Common colors (only if not default)
    add_if_not_default("AddedColor", state.added_color.hex.get());
    add_if_not_default("Highlight_Color", state.highlight_color.hex.get());
    add_if_not_default("BaseColor", state.base_color.hex.get());
    add_if_not_default("InnerColor", state.inner_color.hex.get());
    add_if_not_default("OuterColor", state.outer_color.hex.get());
    add_if_not_default("PrimaryColor", state.primary_color.hex.get());
    add_if_not_default("SecondaryColor", state.secondary_color.hex.get());
    add_if_not_default("TetriaryColor", state.tetriary_color.hex.get());
    add_if_not_default("Primary", state.primary.hex.get());
    add_if_not_default("Secondary", state.secondary.hex.get());
    add_if_not_default("Tertiary", state.tertiary.hex.get());
    add_if_not_default("Primary_Color", state.primary_color_underscore.hex.get());
    add_if_not_default("Secondary_Color", state.secondary_color_underscore.hex.get());
    add_if_not_default("Tertiary_Color", state.tertiary_color_underscore.hex.get());

    colors
}
