//! Section components for the Dyes tab

use floem::prelude::*;
use floem::text::Weight;

use super::color_row::color_row;
use super::shared::ColorCategory;
use super::shared::constants::*;
use crate::gui::state::{DyeColorEntry, DyesState};

/// Creates a section header - matches Results Log style
fn section_header(title: &'static str) -> impl IntoView {
    label(move || title).style(|s| {
        s.font_size(FONT_HEADER)
            .font_weight(Weight::SEMIBOLD)
            .margin_bottom(PADDING_STD)
    })
}

/// Inner card style for the color rows
fn inner_card_style(s: floem::style::Style) -> floem::style::Style {
    s.width_full()
        .background(BG_CARD)
        .border(1.0)
        .border_color(BORDER_CARD)
        .border_radius(RADIUS_STD)
}

/// Outer white card style - matches Results Log container
fn outer_card_style(s: floem::style::Style) -> floem::style::Style {
    s.flex_grow(1.0)
        .flex_basis(0.0)
        .padding(PADDING_LG)
        .background(Color::WHITE)
        .border(1.0)
        .border_color(BORDER_CARD)
        .border_radius(6.0)
}

/// Build a color section from a list of color entries
fn color_section(
    title: &'static str,
    entries: Vec<DyeColorEntry>,
    status: RwSignal<String>,
) -> impl IntoView {
    v_stack((
        section_header(title),
        scroll(
            dyn_stack(
                move || entries.clone(),
                |entry| entry.name,
                move |entry| color_row(entry, status),
            )
            .style(|s| s.width_full().padding(4.0).flex_col()),
        )
        .style(inner_card_style),
    ))
    .style(outer_card_style)
}

/// Required colors section
pub fn required_section(state: DyesState, status: RwSignal<String>) -> impl IntoView {
    let entries: Vec<DyeColorEntry> = state
        .colors_by_category(ColorCategory::Required)
        .into_iter()
        .cloned()
        .collect();

    color_section("Required", entries, status)
}

/// Recommended colors section
pub fn recommended_section(state: DyesState, status: RwSignal<String>) -> impl IntoView {
    let entries: Vec<DyeColorEntry> = state
        .colors_by_category(ColorCategory::Recommended)
        .into_iter()
        .cloned()
        .collect();

    color_section("Recommended", entries, status)
}

/// Commonly used in mods section
pub fn common_section(state: DyesState, status: RwSignal<String>) -> impl IntoView {
    let entries: Vec<DyeColorEntry> = state
        .colors_by_category(ColorCategory::Common)
        .into_iter()
        .cloned()
        .collect();

    color_section("Commonly Used", entries, status)
}

/// Header section with title and status message
pub fn header_section(status: RwSignal<String>) -> impl IntoView {
    h_stack((
        label(|| "Dye Lab").style(|s| s.font_size(FONT_TITLE).font_weight(Weight::BOLD)),
        empty().style(|s| s.flex_grow(1.0)),
        // Status message
        dyn_container(
            move || status.get(),
            move |msg| {
                if msg.is_empty() {
                    empty().into_any()
                } else {
                    label(move || msg.clone())
                        .style(|s| {
                            s.padding_horiz(PADDING_BTN_H)
                                .padding_vert(PADDING_BTN_V)
                                .border_radius(RADIUS_STD)
                                .font_size(FONT_STATUS)
                                .background(BG_SUCCESS)
                                .color(TEXT_SUCCESS)
                        })
                        .into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(PADDING_LG)
            .gap(GAP_STD)
            .items_center()
            .background(Color::WHITE)
            .border_bottom(1.0)
            .border_color(BORDER_CARD)
    })
}
