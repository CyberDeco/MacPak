//! Generate Dye section - create new dyes from current color settings

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::{DyesState, GeneratedDyeEntry};
use crate::gui::utils::{generate_uuid, UuidFormat};
use super::export::check_required_colors_at_default;
use super::shared::{button_style, input_style, collect_colors_skip_defaults};
use super::shared::constants::*;

/// Generate Dye section for creating new dye entries
pub fn generate_dye_section(state: DyesState) -> impl IntoView {
    let individual_dye_name = state.individual_dye_name;
    let generated_dyes = state.generated_dyes;
    let status = state.status_message;

    v_stack((
        // Section header
        label(|| "Generate Dye")
            .style(|s| s.font_size(FONT_HEADER).font_weight(Weight::SEMIBOLD).margin_bottom(PADDING_STD)),

        // Inner card
        v_stack((
            // Dye Name row
            h_stack((
                label(|| "Dye Name")
                    .style(|s| s.width(LABEL_WIDTH_SM).font_size(FONT_BODY)),
                text_input(individual_dye_name)
                    .placeholder("e.g. MyMod_Dye_Crimson")
                    .style(input_style),
            ))
            .style(|s| s.width_full().items_center().gap(GAP_STD)),

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
                                .background(ACCENT_SUCCESS)
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
                            let colors = collect_colors_skip_defaults(&state_gen);

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
            .style(|s| s.width_full().margin_top(PADDING_STD).gap(GAP_STD)),
        ))
        .style(|s| {
            s.width_full()
                .padding(PADDING_BTN_H)
                .gap(GAP_STD)
                .background(BG_CARD)
                .border(1.0)
                .border_color(BORDER_CARD)
                .border_radius(RADIUS_STD)
        }),
    ))
    .style(|s| {
        s.width_full()
            .padding(PADDING_LG)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(BORDER_CARD)
            .border_radius(6.0)
    })
}
