//! Export functionality for the Dyes tab

mod export_mod;

use floem::prelude::*;
use floem::text::Weight;

use crate::gui::state::DyesState;
use super::shared::{
    secondary_button_style,
    collect_all_colors, reset_colors_to_default, load_colors_from_map,
    nav_row, selector_container_green, empty_state_style, selector_label_style,
};
use super::shared::constants::*;

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
                .style(|s| s.font_size(FONT_HEADER).font_weight(Weight::SEMIBOLD)),
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
        .style(|s| s.width_full().items_center().gap(GAP_STD).margin_bottom(PADDING_STD)),

        // Inner card with generated dyes list
        v_stack((
            // Generated dyes selector (only shows when there are generated dyes)
            generated_dyes_selector(state.clone()),

            // Display selected dye info
            selected_dye_display(generated_dyes, selected_index, status),
        ))
        .style(|s| {
            s.width_full()
                .padding(PADDING_BTN_H)
                .background(BG_CARD)
                .border(1.0)
                .border_color(BORDER_CARD)
                .border_radius(RADIUS_STD)
        }),
    ))
    .style(|s| {
        s.flex_grow(1.0)
            .flex_basis(0.0)
            .padding(PADDING_LG)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(BORDER_CARD)
            .border_radius(6.0)
    })
}

/// Load colors from a generated dye entry into the color pickers
fn load_generated_dye_colors(state: &DyesState) {
    let dyes = state.generated_dyes.get();
    if let Some(idx) = state.selected_generated_index.get() {
        if let Some(dye) = dyes.get(idx) {
            // Reset all colors to default before applying stored colors
            reset_colors_to_default(state);
            // Load colors from the stored HashMap
            load_colors_from_map(state, &dye.colors);
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
                    .style(empty_state_style)
                    .into_any()
            } else {
                let state_nav = state_for_selector.clone();
                let state_del = state_for_selector.clone();
                let state_clear = state_for_selector.clone();
                let state_update = state_for_selector.clone();

                // Clones for nav_row callbacks
                let generated_dyes_display = generated_dyes;
                let selected_index_display = selected_index;
                let state_on_nav = state_nav.clone();

                h_stack((
                    label(|| "Dyes:").style(selector_label_style),
                    // Navigation row
                    nav_row(
                        selected_index,
                        move || generated_dyes.get().len(),
                        move || {
                            let dyes = generated_dyes_display.get();
                            let idx = selected_index_display.get().unwrap_or(0);
                            if let Some(dye) = dyes.get(idx) {
                                format!("{} ({}/{})", dye.name, idx + 1, dyes.len())
                            } else {
                                "Select...".to_string()
                            }
                        },
                        {
                            let state = state_on_nav.clone();
                            move || load_generated_dye_colors(&state)
                        },
                        {
                            let state = state_on_nav.clone();
                            move || load_generated_dye_colors(&state)
                        },
                    ),
                    // Update button
                    {
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
                                    dyes[idx].colors = collect_all_colors(&state_update);
                                    generated_dyes.set(dyes);
                                    status.set(format!("Updated '{}'", name));
                                }
                            })
                    },
                    // Delete button
                    {
                        let selected_index = selected_index;
                        let generated_dyes = generated_dyes;
                        let status = status;
                        label(|| "Delete")
                            .style(|s| secondary_button_style(s).color(ACCENT_DANGER))
                            .on_click_stop(move |_| {
                                let idx = selected_index.get().unwrap_or(0);
                                let mut dyes = generated_dyes.get();
                                if idx < dyes.len() {
                                    let name = dyes[idx].name.clone();
                                    dyes.remove(idx);
                                    generated_dyes.set(dyes.clone());
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
                    // Clear All button
                    {
                        let selected_index = selected_index;
                        let generated_dyes = generated_dyes;
                        let status = status;
                        label(|| "Clear All")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                generated_dyes.set(Vec::new());
                                selected_index.set(None);
                                reset_colors_to_default(&state_clear);
                                status.set("Cleared all generated dyes".to_string());
                            })
                    },
                ))
                .style(selector_container_green)
                .into_any()
            }
        },
    )
}

/// Display selected dye information with editable name
fn selected_dye_display(
    generated_dyes: RwSignal<Vec<crate::gui::state::GeneratedDyeEntry>>,
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
                            .style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
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
                                    .min_width(INPUT_MIN_WIDTH)
                                    .padding(PADDING_BTN_V)
                                    .font_size(FONT_BODY)
                                    .font_family("monospace".to_string())
                                    .background(Color::WHITE)
                                    .border(1.0)
                                    .border_color(BORDER_INPUT)
                                    .border_radius(RADIUS_STD)
                            }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),

                    // Dye UUID row
                    h_stack((
                        label(|| "Dye UUID")
                            .style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
                        label(move || display_uuid.get())
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .padding(PADDING_BTN_V)
                                    .font_size(FONT_BODY)
                                    .font_family("monospace".to_string())
                                    .background(BG_INPUT_READONLY)
                                    .border(1.0)
                                    .border_color(BORDER_INPUT)
                                    .border_radius(RADIUS_STD)
                            }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),

                    // Mod UUID row - greyed out, will be generated at export
                    h_stack((
                        label(|| "Mod UUID")
                            .style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY).color(TEXT_MUTED)),
                        label(|| "Will be generated at export")
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .padding(PADDING_BTN_V)
                                    .font_size(FONT_BODY)
                                    .color(TEXT_MUTED)
                                    .background(BG_INPUT_READONLY)
                                    .border(1.0)
                                    .border_color(BORDER_INPUT)
                                    .border_radius(RADIUS_STD)
                            }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),
                ))
                .style(|s| s.width_full().gap(GAP_STD))
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
}
