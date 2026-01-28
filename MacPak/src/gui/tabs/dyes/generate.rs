//! Generate Dye section - create new dyes from current color settings

use floem::prelude::*;
use floem::text::Weight;
use floem::views::PlaceholderTextClass;

use super::export::check_required_colors_at_default;
use super::shared::constants::*;
use super::shared::{button_style, collect_colors_skip_defaults, input_style};
use crate::gui::state::{DyesState, GeneratedDyeEntry};
use crate::gui::utils::{UuidFormat, generate_uuid};

/// Convert a string to snake_case format suitable for dye names
/// - Replaces spaces and hyphens with underscores
/// - Removes non-alphanumeric characters (except underscores)
/// - Collapses multiple underscores
/// - Trims leading/trailing underscores
fn to_snake_case(s: &str) -> String {
    let result: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    // Collapse multiple underscores and trim
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_underscore = true; // Start true to trim leading underscores
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

    // Trim trailing underscore
    if collapsed.ends_with('_') {
        collapsed.pop();
    }

    collapsed
}

/// Generate Dye section for creating new dye entries
pub fn generate_dye_section(state: DyesState) -> impl IntoView {
    let individual_dye_name = state.individual_dye_name;
    let individual_display_name = state.individual_display_name;
    let individual_description = state.individual_description;
    let generated_dyes = state.generated_dyes;
    let status = state.status_message;

    // Live validation: track which required colors are at default
    let missing_required: RwSignal<Vec<&'static str>> = RwSignal::new(Vec::new());
    let state_for_validation = state.clone();
    let _ = floem::reactive::create_effect(move |_| {
        // Re-check whenever any color changes
        let defaults = check_required_colors_at_default(&state_for_validation);
        missing_required.set(defaults);
    });

    v_stack((
        // Section header
        label(|| "Generate Dye").style(|s| {
            s.font_size(FONT_HEADER)
                .font_weight(Weight::SEMIBOLD)
                .margin_bottom(PADDING_STD)
        }),
        // Inner card
        v_stack((
            // Dye Name row
            h_stack((
                label(|| "Dye Name").style(|s| s.width(LABEL_WIDTH_SM).font_size(FONT_BODY)),
                text_input(individual_dye_name)
                    .placeholder("e.g. MyMod_Dye_Crimson")
                    .style(|s| {
                        input_style(s).class(PlaceholderTextClass, |s| {
                            s.color(Color::rgb8(120, 120, 120))
                        })
                    }),
            ))
            .style(|s| s.width_full().items_center().gap(GAP_STD)),
            // Display Name row
            h_stack((
                label(|| "Display Name").style(|s| s.width(LABEL_WIDTH_SM).font_size(FONT_BODY)),
                text_input(individual_display_name)
                    .placeholder("e.g. Crimson Dye")
                    .style(|s| {
                        input_style(s).class(PlaceholderTextClass, |s| {
                            s.color(Color::rgb8(120, 120, 120))
                        })
                    }),
            ))
            .style(|s| s.width_full().items_center().gap(GAP_STD)),
            // Description row
            h_stack((
                label(|| "Description").style(|s| s.width(LABEL_WIDTH_SM).font_size(FONT_BODY)),
                text_input(individual_description)
                    .placeholder("e.g. A deep crimson dye")
                    .style(|s| {
                        input_style(s).class(PlaceholderTextClass, |s| {
                            s.color(Color::rgb8(120, 120, 120))
                        })
                    }),
            ))
            .style(|s| s.width_full().items_center().gap(GAP_STD)),
            // Generate button with live validation (button dims when required colors missing)
            h_stack(({
                let state_gen = state.clone();
                let dye_name = individual_dye_name;
                let display_name = individual_display_name;
                let description = individual_description;
                let generated_dyes = generated_dyes;
                let status = status;
                let missing = missing_required;
                label(|| "Generate Dye")
                    .style(move |s| {
                        let base = button_style(s)
                            .color(Color::WHITE)
                            .font_weight(Weight::SEMIBOLD);
                        // Dim the button if validation fails
                        if missing.get().is_empty() {
                            base.background(ACCENT_SUCCESS)
                        } else {
                            base.background(Color::rgb8(150, 150, 150))
                        }
                    })
                    .on_click_stop(move |_| {
                        let raw_name = dye_name.get();
                        if raw_name.is_empty() {
                            status.set("Error: Dye name is required".to_string());
                            return;
                        }

                        // Normalize to snake_case
                        let name = to_snake_case(&raw_name);
                        if name.is_empty() {
                            status.set(
                                "Error: Dye name must contain alphanumeric characters".to_string(),
                            );
                            return;
                        }

                        // Check for required colors at default (safety check)
                        let defaults = check_required_colors_at_default(&state_gen);
                        if !defaults.is_empty() {
                            status.set(format!("Missing: {}", defaults.join(", ")));
                            return;
                        }

                        // Get display name and description (use defaults if empty)
                        let display_name_val = display_name.get();
                        let display_name_final = if display_name_val.is_empty() {
                            name.replace('_', " ")
                        } else {
                            display_name_val
                        };
                        let description_final = description.get();

                        // Generate UUIDs (Standard for resource IDs, Larian for localization handles)
                        let preset_uuid = generate_uuid(UuidFormat::Standard);
                        let template_uuid = generate_uuid(UuidFormat::Standard);
                        let name_handle = generate_uuid(UuidFormat::Larian);
                        let desc_handle = generate_uuid(UuidFormat::Larian);

                        // Collect current colors
                        let colors = collect_colors_skip_defaults(&state_gen);

                        // Create new dye entry
                        let entry = GeneratedDyeEntry {
                            name: name.clone(),
                            display_name: display_name_final,
                            description: description_final,
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

                        // Clear all inputs and show success
                        dye_name.set(String::new());
                        display_name.set(String::new());
                        description.set(String::new());
                        status.set(format!("Generated dye '{}' ({} total)", name, count));
                    })
            },))
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
