//! UI components for the import section

use floem::prelude::*;
use floem::text::Weight;

use crate::state::DyesState;
use super::super::shared::{secondary_button_style, nav_button_style, selector_display_style};
use super::operations::{load_selected_entry, load_lsf_entry, update_lsf_entry, reexport_lsf};

/// Display imported fields with editable name
pub fn imported_fields_display(
    state: DyesState,
    dye_name: RwSignal<String>,
    preset_uuid: RwSignal<String>,
    template_uuid: RwSignal<String>,
) -> impl IntoView {
    let status = state.status_message;
    let imported_lsf = state.imported_lsf_entries;
    let selected_lsf = state.selected_lsf_index;

    // Track visibility separately to avoid re-renders when typing
    let has_data: RwSignal<bool> = RwSignal::new(false);
    let _ = floem::reactive::create_effect(move |_| {
        // Only track TXT entries and LSF entries, not dye_name itself
        let txt_has_data = !state.imported_entries.get().is_empty();
        let lsf_has_data = !state.imported_lsf_entries.get().is_empty();
        has_data.set(txt_has_data || lsf_has_data);
    });

    dyn_container(
        move || has_data.get(),
        move |show| {
            if !show {
                label(|| "No data imported")
                    .style(|s| s.font_size(11.0).color(Color::rgb8(150, 150, 150)).padding(8.0))
                    .into_any()
            } else {
                v_stack((
                    // Dye Name row - editable
                    h_stack((
                        label(|| "Dye Name")
                            .style(|s| s.width(90.0).font_size(11.0)),
                        text_input(dye_name)
                            .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                                // Update the name in imported_lsf_entries when focus is lost
                                let new_name = dye_name.get();
                                if let Some(idx) = selected_lsf.get() {
                                    let mut entries = imported_lsf.get();
                                    if idx < entries.len() && !new_name.is_empty() {
                                        entries[idx].name = new_name.clone();
                                        imported_lsf.set(entries);
                                        status.set(format!("Renamed to '{}'", new_name));
                                    }
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

                    // Dye UUID row (formerly Preset UUID)
                    h_stack((
                        label(|| "Dye UUID")
                            .style(|s| s.width(90.0).font_size(11.0)),
                        label(move || {
                            let uuid = preset_uuid.get();
                            if uuid.is_empty() { "(not found)".to_string() } else { uuid }
                        })
                        .style(move |s| {
                            let s = s.flex_grow(1.0)
                                .padding(6.0)
                                .font_size(11.0)
                                .font_family("monospace".to_string())
                                .background(Color::rgb8(245, 245, 245))
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0);
                            if preset_uuid.get().is_empty() {
                                s.color(Color::rgb8(150, 150, 150))
                            } else {
                                s
                            }
                        }),
                    ))
                    .style(|s| s.width_full().items_center().gap(8.0)),

                    // Mod UUID row (formerly Template UUID)
                    h_stack((
                        label(|| "Mod UUID")
                            .style(|s| s.width(90.0).font_size(11.0)),
                        label(move || {
                            let uuid = template_uuid.get();
                            if uuid.is_empty() { "(not found)".to_string() } else { uuid }
                        })
                        .style(move |s| {
                            let s = s.flex_grow(1.0)
                                .padding(6.0)
                                .font_size(11.0)
                                .font_family("monospace".to_string())
                                .background(Color::rgb8(245, 245, 245))
                                .border(1.0)
                                .border_color(Color::rgb8(200, 200, 200))
                                .border_radius(4.0);
                            if template_uuid.get().is_empty() {
                                s.color(Color::rgb8(150, 150, 150))
                            } else {
                                s
                            }
                        }),
                    ))
                    .style(|s| s.width_full().items_center().gap(8.0)),
                ))
                .style(|s| s.width_full().gap(8.0))
                .into_any()
            }
        },
    )
}

/// TXT import selector component
pub fn txt_import_selector(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_preset_uuid: RwSignal<String>,
    imported_template_uuid: RwSignal<String>,
) -> impl IntoView {
    let state_for_selector = state.clone();
    let imported_entries = state.imported_entries;
    let selected_index = state.selected_import_index;

    dyn_container(
        move || imported_entries.get().len(),
        move |count| {
            if count == 0 {
                empty().into_any()
            } else {
                let state_load = state_for_selector.clone();

                h_stack((
                    label(|| "TXT:").style(|s| s.font_size(11.0).font_weight(Weight::SEMIBOLD)),
                    // Dropdown-style selector
                    h_stack((
                        // Previous button
                        {
                            let selected_index = selected_index;
                            let imported_entries = imported_entries;
                            let state_prev = state_for_selector.clone();
                            let imported_dye_name = imported_dye_name;
                            let imported_preset_uuid = imported_preset_uuid;
                            let imported_template_uuid = imported_template_uuid;
                            label(|| "<")
                                .style(nav_button_style)
                                .on_click_stop(move |_| {
                                    let len = imported_entries.get().len();
                                    if len > 0 {
                                        let current = selected_index.get().unwrap_or(0);
                                        let new_idx = if current == 0 { len - 1 } else { current - 1 };
                                        selected_index.set(Some(new_idx));
                                        load_selected_entry(state_prev.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);
                                    }
                                })
                        },
                        // Current selection display
                        {
                            let selected_index = selected_index;
                            let imported_entries = imported_entries;
                            label(move || {
                                let entries = imported_entries.get();
                                let idx = selected_index.get().unwrap_or(0);
                                if let Some((name, _, _)) = entries.get(idx) {
                                    format!("{} ({}/{})", name, idx + 1, entries.len())
                                } else {
                                    "Select...".to_string()
                                }
                            })
                            .style(selector_display_style)
                        },
                        // Next button
                        {
                            let selected_index = selected_index;
                            let imported_entries = imported_entries;
                            let state_next = state_for_selector.clone();
                            let imported_dye_name = imported_dye_name;
                            let imported_preset_uuid = imported_preset_uuid;
                            let imported_template_uuid = imported_template_uuid;
                            label(|| ">")
                                .style(nav_button_style)
                                .on_click_stop(move |_| {
                                    let len = imported_entries.get().len();
                                    if len > 0 {
                                        let current = selected_index.get().unwrap_or(0);
                                        let new_idx = (current + 1) % len;
                                        selected_index.set(Some(new_idx));
                                        load_selected_entry(state_next.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);
                                    }
                                })
                        },
                    ))
                    .style(|s| s.flex_grow(1.0).gap(4.0).items_center()),
                    // Clear button
                    {
                        let state_clear = state_load.clone();
                        let imported_entries = imported_entries;
                        let selected_index = selected_index;
                        let imported_dye_name = imported_dye_name;
                        let imported_preset_uuid = imported_preset_uuid;
                        let imported_template_uuid = imported_template_uuid;
                        label(|| "Clear")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                imported_entries.set(Vec::new());
                                selected_index.set(None);
                                imported_dye_name.set(String::new());
                                imported_preset_uuid.set(String::new());
                                imported_template_uuid.set(String::new());
                                state_clear.status_message.set("Cleared".to_string());
                            })
                    },
                ))
                .style(|s| {
                    s.width_full()
                        .padding(8.0)
                        .margin_bottom(8.0)
                        .gap(8.0)
                        .items_center()
                        .background(Color::rgb8(243, 244, 246))
                        .border(1.0)
                        .border_color(Color::rgb8(209, 213, 219))
                        .border_radius(4.0)
                })
                .into_any()
            }
        },
    )
}

/// LSF import selector component
pub fn lsf_import_selector(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_preset_uuid: RwSignal<String>,
    imported_template_uuid: RwSignal<String>,
) -> impl IntoView {
    let state_for_lsf = state.clone();
    let imported_lsf = state.imported_lsf_entries;
    let selected_lsf = state.selected_lsf_index;

    dyn_container(
        move || imported_lsf.get().len(),
        move |count| {
            if count == 0 {
                empty().into_any()
            } else {
                let state_nav = state_for_lsf.clone();

                h_stack((
                    label(|| "LSF:").style(|s| s.font_size(11.0).font_weight(Weight::SEMIBOLD)),
                    // Dropdown-style selector
                    h_stack((
                        // Previous button
                        {
                            let selected_lsf = selected_lsf;
                            let imported_lsf = imported_lsf;
                            let state_prev = state_for_lsf.clone();
                            let imported_dye_name = imported_dye_name;
                            let imported_preset_uuid = imported_preset_uuid;
                            let imported_template_uuid = imported_template_uuid;
                            label(|| "<")
                                .style(nav_button_style)
                                .on_click_stop(move |_| {
                                    let len = imported_lsf.get().len();
                                    if len > 0 {
                                        let current = selected_lsf.get().unwrap_or(0);
                                        let new_idx = if current == 0 { len - 1 } else { current - 1 };
                                        selected_lsf.set(Some(new_idx));
                                        load_lsf_entry(state_prev.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);
                                    }
                                })
                        },
                        // Current selection display
                        {
                            let selected_lsf = selected_lsf;
                            let imported_lsf = imported_lsf;
                            label(move || {
                                let entries = imported_lsf.get();
                                let idx = selected_lsf.get().unwrap_or(0);
                                if let Some(entry) = entries.get(idx) {
                                    format!("{} ({}/{})", entry.name, idx + 1, entries.len())
                                } else {
                                    "Select...".to_string()
                                }
                            })
                            .style(selector_display_style)
                        },
                        // Next button
                        {
                            let selected_lsf = selected_lsf;
                            let imported_lsf = imported_lsf;
                            let state_next = state_for_lsf.clone();
                            let imported_dye_name = imported_dye_name;
                            let imported_preset_uuid = imported_preset_uuid;
                            let imported_template_uuid = imported_template_uuid;
                            label(|| ">")
                                .style(nav_button_style)
                                .on_click_stop(move |_| {
                                    let len = imported_lsf.get().len();
                                    if len > 0 {
                                        let current = selected_lsf.get().unwrap_or(0);
                                        let new_idx = (current + 1) % len;
                                        selected_lsf.set(Some(new_idx));
                                        load_lsf_entry(state_next.clone(), imported_dye_name, imported_preset_uuid, imported_template_uuid);
                                    }
                                })
                        },
                    ))
                    .style(|s| s.flex_grow(1.0).gap(4.0).items_center()),
                    // Update button - saves current colors to selected LSF entry
                    {
                        let state_update = state_nav.clone();
                        label(|| "Update")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                update_lsf_entry(state_update.clone());
                            })
                    },
                    // Re-export button - writes all entries back to the original LSF file
                    {
                        let state_reexport = state_nav.clone();
                        label(|| "Re-export")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                reexport_lsf(state_reexport.clone());
                            })
                    },
                    // Clear button
                    {
                        let state_clear = state_nav.clone();
                        let imported_lsf = imported_lsf;
                        let selected_lsf = selected_lsf;
                        let imported_dye_name = imported_dye_name;
                        let imported_preset_uuid = imported_preset_uuid;
                        let imported_template_uuid = imported_template_uuid;
                        label(|| "Clear")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                imported_lsf.set(Vec::new());
                                selected_lsf.set(None);
                                imported_dye_name.set(String::new());
                                imported_preset_uuid.set(String::new());
                                imported_template_uuid.set(String::new());
                                state_clear.status_message.set("Cleared".to_string());
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
