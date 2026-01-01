//! Generate Dye section - create new dyes from current color settings

use std::collections::HashMap;

use floem::prelude::*;
use floem::text::Weight;

use crate::state::{DyesState, GeneratedDyeEntry, UuidFormat};
use crate::utils::generate_uuid;
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
                            let preset_uuid = generate_uuid(UuidFormat::Standard);
                            let template_uuid = generate_uuid(UuidFormat::Standard);
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
    colors.insert("Color_01".to_string(), state.color_01.hex.get());
    colors.insert("Color_02".to_string(), state.color_02.hex.get());
    colors.insert("Color_03".to_string(), state.color_03.hex.get());
    colors.insert("Custom_1".to_string(), state.custom_1.hex.get());
    colors.insert("Custom_2".to_string(), state.custom_2.hex.get());

    // Recommended colors (only if not default)
    let accent = state.accent_color.hex.get();
    if accent.to_lowercase() != "808080" {
        colors.insert("Accent_Color".to_string(), accent);
    }

    colors
}
