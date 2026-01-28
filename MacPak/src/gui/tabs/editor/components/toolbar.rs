//! Editor toolbar component

use floem::prelude::*;
use floem::views::checkbox;

use crate::gui::state::EditorTabsState;

use super::super::operations::{
    convert_file, open_file_dialog, save_file, save_file_as_dialog, validate_content,
};
use super::badges::{format_badge, save_status_badge};

/// Common toolbar button style for consistent height
fn toolbar_button_style(s: floem::style::Style) -> floem::style::Style {
    s.min_height(0.0)
        .height(22.0)
        .max_height(22.0)
        .padding_horiz(6.0)
        .padding_vert(2.0)
        .items_center()
        .justify_center()
}

pub fn editor_toolbar(tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_open = tabs_state.clone();
    let tabs_state_save_check = tabs_state.clone();
    let tabs_state_save_action = tabs_state.clone();
    let tabs_state_save_as = tabs_state.clone();
    let tabs_state_validate = tabs_state.clone();
    let tabs_state_find = tabs_state.clone();
    let tabs_state_lsx = tabs_state.clone();
    let tabs_state_lsj = tabs_state.clone();
    let tabs_state_lsf = tabs_state.clone();
    let _tabs_state_meta = tabs_state.clone();
    let tabs_state_xml = tabs_state.clone();
    let tabs_state_loca = tabs_state.clone();

    h_stack((
        // File operations group
        h_stack((
            button("📂 Open")
                .style(toolbar_button_style)
                .action(move || {
                    open_file_dialog(tabs_state_open.clone());
                }),
            button("💾 Save")
                .style(toolbar_button_style)
                .disabled(move || {
                    tabs_state_save_check.active_tab().map_or(true, |tab| {
                        !tab.modified.get() || tab.converted_from_lsf.get()
                    })
                })
                .action(move || {
                    if let Some(tab) = tabs_state_save_action.active_tab() {
                        save_file(tab);
                    }
                }),
            button("💾 Save As...")
                .style(toolbar_button_style)
                .action(move || {
                    if let Some(tab) = tabs_state_save_as.active_tab() {
                        save_file_as_dialog(tab);
                    }
                }),
        ))
        .style(|s| s.gap(8.0).items_center()),
        separator(),
        // Edit tools group
        h_stack((
            button("🔍 Find").style(toolbar_button_style).action({
                move || {
                    if let Some(tab) = tabs_state_find.active_tab() {
                        let visible = tab.search_visible.get();
                        tab.search_visible.set(!visible);
                    }
                }
            }),
            button("✓ Validate")
                .style(toolbar_button_style)
                .action(move || {
                    if let Some(tab) = tabs_state_validate.active_tab() {
                        validate_content(tab, tabs_state.status_message);
                    }
                }),
            line_number_toggle(tabs_state.show_line_numbers),
        ))
        .style(|s| s.gap(8.0).items_center()),
        // Spacer
        empty().style(|s| s.flex_grow(1.0)),
        separator(),
        // LSF/LSX/LSJ Convert section (disabled for LOCA files)
        h_stack((
            convert_button_lsf_group("LSX", tabs_state_lsx),
            convert_button_lsf_group("LSJ", tabs_state_lsj),
            convert_button_lsf_group("LSF", tabs_state_lsf),
        ))
        .style(|s| s.gap(8.0).items_center()),
        separator(),
        // LOCA/XML Convert section (disabled for LSF/LSX/LSJ files)
        h_stack((
            convert_button_loca_group("XML", tabs_state_xml),
            convert_button_loca_group("LOCA", tabs_state_loca),
        ))
        .style(|s| s.gap(8.0).items_center()),
        separator(),
        // Format badge
        format_badge(tabs_state.clone()),
        // Save status indicator (top right corner)
        save_status_badge(tabs_state.clone()),
    ))
    .style(|s| {
        s.width_full()
            .height(50.0)
            .padding(10.0)
            .gap(8.0)
            .items_center()
            .background(Color::rgb8(245, 245, 245))
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

/// Convert button for LSF/LSX/LSJ group - disabled for LOCA files
fn convert_button_lsf_group(format: &'static str, tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_check = tabs_state.clone();
    let tabs_state_action = tabs_state.clone();

    button(format)
        .style(toolbar_button_style)
        .disabled(move || {
            tabs_state_check.active_tab().map_or(true, |tab| {
                let f = tab.file_format.get().to_uppercase();
                let empty = tab.content.get().is_empty();
                // Disable if: current format matches, empty, OR it's a LOCA-related file
                f == format || empty || matches!(f.as_str(), "LOCA" | "XML")
            })
        })
        .action(move || {
            if let Some(tab) = tabs_state_action.active_tab() {
                convert_file(tab, format);
            }
        })
}

/// Convert button for LOCA/XML group - disabled for LSF/LSX/LSJ files
fn convert_button_loca_group(format: &'static str, tabs_state: EditorTabsState) -> impl IntoView {
    let tabs_state_check = tabs_state.clone();
    let tabs_state_action = tabs_state.clone();

    button(format)
        .style(toolbar_button_style)
        .disabled(move || {
            tabs_state_check.active_tab().map_or(true, |tab| {
                let f = tab.file_format.get().to_uppercase();
                let empty = tab.content.get().is_empty();
                // Disable if: current format matches, empty, OR it's a LSF-related file
                f == format || empty || matches!(f.as_str(), "LSF" | "LSX" | "LSJ")
            })
        })
        .action(move || {
            if let Some(tab) = tabs_state_action.active_tab() {
                convert_file(tab, format);
            }
        })
}

fn separator() -> impl IntoView {
    empty().style(|s| {
        s.width(1.0)
            .height(30.0)
            .background(Color::rgb8(200, 200, 200))
            .margin_horiz(4.0)
    })
}

fn line_number_toggle(show_line_numbers: RwSignal<bool>) -> impl IntoView {
    h_stack((
        checkbox(move || show_line_numbers.get())
            .on_update(move |checked| {
                show_line_numbers.set(checked);
            })
            .style(move |s| s.margin_right(8.0)),
        label(|| "Show Line Numbers")
            .style(|s| s.font_size(12.0).cursor(floem::style::CursorStyle::Pointer)),
    ))
    .on_click_stop(move |_| {
        show_line_numbers.set(!show_line_numbers.get());
    })
    .style(|s| s.padding_horiz(8.0).padding_vert(4.0).items_center())
}
