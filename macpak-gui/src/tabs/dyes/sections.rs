//! Section components for the Dyes tab

use floem::prelude::*;
use floem::text::Weight;

use crate::state::DyesState;
use super::color_row::color_row;

/// Creates a section header - matches Results Log style
fn section_header(title: &'static str) -> impl IntoView {
    label(move || title)
        .style(|s| {
            s.font_size(14.0)
                .font_weight(Weight::SEMIBOLD)
                .margin_bottom(8.0)
        })
}

/// Inner card style for the color rows
fn inner_card_style(s: floem::style::Style) -> floem::style::Style {
    s.width_full()
        .background(Color::rgb8(250, 250, 250))
        .border(1.0)
        .border_color(Color::rgb8(220, 220, 220))
        .border_radius(4.0)
}

/// Outer white card style - matches Results Log container
fn outer_card_style(s: floem::style::Style) -> floem::style::Style {
    s.flex_grow(1.0)
        .flex_basis(0.0)
        .padding(16.0)
        .background(Color::WHITE)
        .border(1.0)
        .border_color(Color::rgb8(220, 220, 220))
        .border_radius(6.0)
}

/// Required colors section
pub fn required_section(state: DyesState, status: RwSignal<String>) -> impl IntoView {
    v_stack((
        section_header("Required"),
        scroll(
            v_stack((
                color_row(state.cloth_primary.clone(), status),
                color_row(state.cloth_secondary.clone(), status),
                color_row(state.cloth_tertiary.clone(), status),
                color_row(state.color_01.clone(), status),
                color_row(state.color_02.clone(), status),
                color_row(state.color_03.clone(), status),
                color_row(state.custom_1.clone(), status),
                color_row(state.custom_2.clone(), status),
                color_row(state.leather_primary.clone(), status),
                color_row(state.leather_secondary.clone(), status),
                color_row(state.leather_tertiary.clone(), status),
                color_row(state.metal_primary.clone(), status),
                color_row(state.metal_secondary.clone(), status),
                color_row(state.metal_tertiary.clone(), status),
            ))
            .style(|s| s.width_full().padding(4.0)),
        )
        .style(inner_card_style),
    ))
    .style(outer_card_style)
}

/// Recommended colors section
pub fn recommended_section(state: DyesState, status: RwSignal<String>) -> impl IntoView {
    v_stack((
        section_header("Recommended"),
        scroll(
            v_stack((
                color_row(state.accent_color.clone(), status),
                color_row(state.glow_color.clone(), status),
                color_row(state.glow_colour.clone(), status),
            ))
            .style(|s| s.width_full().padding(4.0)),
        )
        .style(inner_card_style),
    ))
    .style(outer_card_style)
}

/// Commonly used in mods section
pub fn common_section(state: DyesState, status: RwSignal<String>) -> impl IntoView {
    v_stack((
        section_header("Commonly Used"),
        scroll(
            v_stack((
                color_row(state.added_color.clone(), status),
                color_row(state.highlight_color.clone(), status),
                color_row(state.base_color.clone(), status),
                color_row(state.inner_color.clone(), status),
                color_row(state.outer_color.clone(), status),
                color_row(state.primary_color.clone(), status),
                color_row(state.secondary_color.clone(), status),
                color_row(state.tetriary_color.clone(), status),
                color_row(state.primary.clone(), status),
                color_row(state.secondary.clone(), status),
                color_row(state.tertiary.clone(), status),
                color_row(state.primary_color_underscore.clone(), status),
                color_row(state.secondary_color_underscore.clone(), status),
                color_row(state.tertiary_color_underscore.clone(), status),
            ))
            .style(|s| s.width_full().padding(4.0)),
        )
        .style(inner_card_style),
    ))
    .style(outer_card_style)
}
