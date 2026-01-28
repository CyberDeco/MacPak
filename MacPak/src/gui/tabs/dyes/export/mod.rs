//! Export functionality for the Dyes tab

mod export_mod;

pub use export_mod::export_dye_mod;

use floem::prelude::*;
use floem::text::Weight;

use super::shared::constants::*;
use super::shared::{
    collect_all_colors, empty_state_style, load_colors_from_map, nav_row, reset_colors_to_default,
    secondary_button_style, selector_container_green, selector_label_style,
};
use crate::gui::state::DyesState;

pub use export_mod::check_required_colors_at_default;

/// Export section for generating mod files
pub fn export_section(state: DyesState) -> impl IntoView {
    let generated_dyes = state.generated_dyes;
    let selected_index = state.selected_generated_index;
    let status = state.status_message;

    v_stack((
        // Section header with Export button
        h_stack((
            label(|| "Export").style(|s| s.font_size(FONT_HEADER).font_weight(Weight::SEMIBOLD)),
            empty().style(|s| s.flex_grow(1.0)),
            {
                let generated_dyes = generated_dyes;
                let status = status;
                let show_meta = state.show_meta_dialog;
                label(|| "Export Mod...")
                    .style(secondary_button_style)
                    .on_click_stop(move |_| {
                        let dyes = generated_dyes.get();
                        if dyes.is_empty() {
                            status.set(
                                "Error: No dyes generated. Use 'Generate Dye' first.".to_string(),
                            );
                            return;
                        }

                        // Show meta dialog for export
                        show_meta.set(true);
                    })
            },
        ))
        .style(|s| {
            s.width_full()
                .items_center()
                .gap(GAP_STD)
                .margin_bottom(PADDING_STD)
        }),
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
                                let count = generated_dyes.get().len();
                                if count == 0 {
                                    return;
                                }

                                // Show confirmation dialog
                                let result = rfd::MessageDialog::new()
                                    .set_title("Clear All Dyes")
                                    .set_description(&format!(
                                        "Delete all {} generated dye{}? This cannot be undone.",
                                        count,
                                        if count == 1 { "" } else { "s" }
                                    ))
                                    .set_buttons(rfd::MessageButtons::OkCancel)
                                    .show();

                                if result == rfd::MessageDialogResult::Ok {
                                    generated_dyes.set(Vec::new());
                                    selected_index.set(None);
                                    reset_colors_to_default(&state_clear);
                                    status.set("Cleared all generated dyes".to_string());
                                }
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
    let display_name: RwSignal<String> = RwSignal::new(String::new());
    let display_description: RwSignal<String> = RwSignal::new(String::new());
    let display_uuid: RwSignal<String> = RwSignal::new(String::new());

    // Sync display values when selection or dyes change
    let _ = floem::reactive::create_effect(move |prev_idx: Option<Option<usize>>| {
        let dyes = generated_dyes.get();
        let idx = selected_index.get();

        if let Some(i) = idx {
            if let Some(dye) = dyes.get(i) {
                // Only update editable fields if selection changed (to avoid overwriting user edits)
                if prev_idx != Some(idx) {
                    edit_name.set(dye.name.clone());
                    display_name.set(dye.display_name.clone());
                    display_description.set(dye.description.clone());
                }
                // Always update UUID (read-only, in case dye was just generated)
                display_uuid.set(dye.preset_uuid.clone());
            }
        }
        idx
    });

    // Track only if boolean, not the full entry
    dyn_container(
        move || selected_index.get().is_some() && !generated_dyes.get().is_empty(),
        move |has_selection| {
            if has_selection {
                v_stack((
                    // Dye Name row - editable (stable, won't re-render on typing)
                    h_stack((
                        label(|| "Dye Name").style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
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
                    // Display Name row - editable
                    h_stack((
                        label(|| "Display Name")
                            .style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
                        text_input(display_name)
                            .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                                let new_display_name = display_name.get();
                                let idx = selected_index.get().unwrap_or(0);
                                let mut dyes = generated_dyes.get();
                                if idx < dyes.len() {
                                    dyes[idx].display_name = new_display_name;
                                    generated_dyes.set(dyes);
                                }
                            })
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .flex_basis(0.0)
                                    .width_full()
                                    .min_width(INPUT_MIN_WIDTH)
                                    .padding(PADDING_BTN_V)
                                    .font_size(FONT_BODY)
                                    .background(Color::WHITE)
                                    .border(1.0)
                                    .border_color(BORDER_INPUT)
                                    .border_radius(RADIUS_STD)
                            }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),
                    // Description row - editable
                    h_stack((
                        label(|| "Description")
                            .style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
                        text_input(display_description)
                            .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                                let new_description = display_description.get();
                                let idx = selected_index.get().unwrap_or(0);
                                let mut dyes = generated_dyes.get();
                                if idx < dyes.len() {
                                    dyes[idx].description = new_description;
                                    generated_dyes.set(dyes);
                                }
                            })
                            .style(|s| {
                                s.flex_grow(1.0)
                                    .flex_basis(0.0)
                                    .width_full()
                                    .min_width(INPUT_MIN_WIDTH)
                                    .padding(PADDING_BTN_V)
                                    .font_size(FONT_BODY)
                                    .background(Color::WHITE)
                                    .border(1.0)
                                    .border_color(BORDER_INPUT)
                                    .border_radius(RADIUS_STD)
                            }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),
                    // Dye UUID row (read-only)
                    h_stack((
                        label(|| "Dye UUID").style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
                        label(move || display_uuid.get()).style(|s| {
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
                ))
                .style(|s| s.width_full().gap(GAP_STD))
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
}
