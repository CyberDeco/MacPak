//! UI components for the import section

use floem::prelude::*;

use super::super::shared::constants::*;
use super::super::shared::{
    nav_row, secondary_button_style, selector_container_gray, selector_container_green,
    selector_label_style,
};
use super::operations::{load_lsf_entry, load_selected_entry};
use crate::gui::state::DyesState;

/// Display imported fields with editable name
pub fn imported_fields_display(
    state: DyesState,
    dye_name: RwSignal<String>,
    display_name: RwSignal<String>,
    mod_name: RwSignal<String>,
    mod_author: RwSignal<String>,
) -> impl IntoView {
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
                    .style(|s| {
                        s.font_size(FONT_BODY)
                            .color(TEXT_MUTED)
                            .padding(PADDING_STD)
                    })
                    .into_any()
            } else {
                v_stack((
                    // Dye Name row
                    h_stack((
                        label(|| "Dye Name").style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
                        label(move || {
                            let new_name = dye_name.get();
                            if new_name.is_empty() {
                                "(not found)".to_string()
                            } else {
                                new_name
                            }
                        })
                        .style(move |s| {
                            let s = s
                                .flex_grow(1.0)
                                .padding(PADDING_BTN_V)
                                .font_size(FONT_BODY)
                                .font_family("monospace".to_string())
                                .background(BG_INPUT_READONLY)
                                .border(1.0)
                                .border_color(BORDER_INPUT)
                                .border_radius(RADIUS_STD);
                            if dye_name.get().is_empty() {
                                s.color(TEXT_MUTED)
                            } else {
                                s
                            }
                        }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),
                    // Display Name row
                    h_stack((
                        label(|| "Display Name")
                            .style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
                        label(move || {
                            let name = display_name.get();
                            if name.is_empty() {
                                "(not found)".to_string()
                            } else {
                                name
                            }
                        })
                        .style(move |s| {
                            let s = s
                                .flex_grow(1.0)
                                .padding(PADDING_BTN_V)
                                .font_size(FONT_BODY)
                                .background(BG_INPUT_READONLY)
                                .border(1.0)
                                .border_color(BORDER_INPUT)
                                .border_radius(RADIUS_STD);
                            if display_name.get().is_empty() {
                                s.color(TEXT_MUTED)
                            } else {
                                s
                            }
                        }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),
                    // Mod Name row
                    h_stack((
                        label(|| "Mod Name").style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
                        label(move || {
                            let name = mod_name.get();
                            if name.is_empty() {
                                "(not found)".to_string()
                            } else {
                                name
                            }
                        })
                        .style(move |s| {
                            let s = s
                                .flex_grow(1.0)
                                .padding(PADDING_BTN_V)
                                .font_size(FONT_BODY)
                                .background(BG_INPUT_READONLY)
                                .border(1.0)
                                .border_color(BORDER_INPUT)
                                .border_radius(RADIUS_STD);
                            if mod_name.get().is_empty() {
                                s.color(TEXT_MUTED)
                            } else {
                                s
                            }
                        }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),
                    // Mod Author row
                    h_stack((
                        label(|| "Mod Author").style(|s| s.width(LABEL_WIDTH).font_size(FONT_BODY)),
                        label(move || {
                            let author = mod_author.get();
                            if author.is_empty() {
                                "(not found)".to_string()
                            } else {
                                author
                            }
                        })
                        .style(move |s| {
                            let s = s
                                .flex_grow(1.0)
                                .padding(PADDING_BTN_V)
                                .font_size(FONT_BODY)
                                .background(BG_INPUT_READONLY)
                                .border(1.0)
                                .border_color(BORDER_INPUT)
                                .border_radius(RADIUS_STD);
                            if mod_author.get().is_empty() {
                                s.color(TEXT_MUTED)
                            } else {
                                s
                            }
                        }),
                    ))
                    .style(|s| s.width_full().items_center().gap(GAP_STD)),
                ))
                .style(|s| s.width_full().gap(GAP_STD))
                .into_any()
            }
        },
    )
}

/// TXT import selector component
pub fn txt_import_selector(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_display_name: RwSignal<String>,
    _imported_mod_name: RwSignal<String>,
    _imported_mod_author: RwSignal<String>,
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
                let state_on_nav = state_for_selector.clone();
                let imported_entries_display = imported_entries;
                let selected_index_display = selected_index;

                h_stack((
                    label(|| "TXT:").style(selector_label_style),
                    // Navigation row
                    nav_row(
                        selected_index,
                        move || imported_entries.get().len(),
                        move || {
                            let entries = imported_entries_display.get();
                            let idx = selected_index_display.get().unwrap_or(0);
                            if let Some((name, _, _)) = entries.get(idx) {
                                format!("{} ({}/{})", name, idx + 1, entries.len())
                            } else {
                                "Select...".to_string()
                            }
                        },
                        {
                            let state = state_on_nav.clone();
                            move || {
                                load_selected_entry(
                                    state.clone(),
                                    imported_dye_name,
                                    imported_display_name,
                                )
                            }
                        },
                        {
                            let state = state_on_nav.clone();
                            move || {
                                load_selected_entry(
                                    state.clone(),
                                    imported_dye_name,
                                    imported_display_name,
                                )
                            }
                        },
                    ),
                    // Clear button
                    {
                        let imported_entries = imported_entries;
                        let selected_index = selected_index;
                        label(|| "Clear")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                let count = imported_entries.get().len();
                                if count == 0 {
                                    return;
                                }

                                let result = rfd::MessageDialog::new()
                                    .set_title("Clear TXT Import")
                                    .set_description(&format!(
                                        "Clear {} imported TXT entr{}?",
                                        count,
                                        if count == 1 { "y" } else { "ies" }
                                    ))
                                    .set_buttons(rfd::MessageButtons::OkCancel)
                                    .show();

                                if result == rfd::MessageDialogResult::Ok {
                                    imported_entries.set(Vec::new());
                                    selected_index.set(None);
                                    imported_dye_name.set(String::new());
                                    imported_display_name.set(String::new());
                                    state_load.status_message.set("Cleared".to_string());
                                }
                            })
                    },
                ))
                .style(selector_container_gray)
                .into_any()
            }
        },
    )
}

/// LSF import selector component
pub fn lsf_import_selector(
    state: DyesState,
    imported_dye_name: RwSignal<String>,
    imported_display_name: RwSignal<String>,
    _imported_mod_name: RwSignal<String>,
    _imported_mod_author: RwSignal<String>,
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
                let state_on_nav = state_for_lsf.clone();
                let imported_lsf_display = imported_lsf;
                let selected_lsf_display = selected_lsf;

                h_stack((
                    // label(|| "LSF:").style(selector_label_style),
                    // Navigation row
                    nav_row(
                        selected_lsf,
                        move || imported_lsf.get().len(),
                        move || {
                            let entries = imported_lsf_display.get();
                            let idx = selected_lsf_display.get().unwrap_or(0);
                            if let Some(entry) = entries.get(idx) {
                                format!("{} ({}/{})", entry.name, idx + 1, entries.len())
                            } else {
                                "Select...".to_string()
                            }
                        },
                        {
                            let state = state_on_nav.clone();
                            move || {
                                load_lsf_entry(
                                    state.clone(),
                                    imported_dye_name,
                                    imported_display_name,
                                )
                            }
                        },
                        {
                            let state = state_on_nav.clone();
                            move || {
                                load_lsf_entry(
                                    state.clone(),
                                    imported_dye_name,
                                    imported_display_name,
                                )
                            }
                        },
                    ),
                    // Clear button
                    {
                        let state_clear = state_nav.clone();
                        let imported_lsf = imported_lsf;
                        let selected_lsf = selected_lsf;
                        label(|| "Clear")
                            .style(secondary_button_style)
                            .on_click_stop(move |_| {
                                let count = imported_lsf.get().len();
                                if count == 0 {
                                    return;
                                }

                                let result = rfd::MessageDialog::new()
                                    .set_title("Clear LSF Import")
                                    .set_description(&format!(
                                        "Clear {} imported LSF entr{}?",
                                        count,
                                        if count == 1 { "y" } else { "ies" }
                                    ))
                                    .set_buttons(rfd::MessageButtons::OkCancel)
                                    .show();

                                if result == rfd::MessageDialogResult::Ok {
                                    imported_lsf.set(Vec::new());
                                    selected_lsf.set(None);
                                    imported_dye_name.set(String::new());
                                    imported_display_name.set(String::new());
                                    state_clear.status_message.set("Cleared".to_string());
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
