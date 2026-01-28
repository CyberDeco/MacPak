//! UI styles shared between import and export sections

use super::constants::*;
use floem::prelude::*;

/// Input field style - fills available width
pub fn input_style(s: floem::style::Style) -> floem::style::Style {
    s.flex_grow(1.0)
        .flex_basis(0.0)
        .width_full()
        .min_width(INPUT_MIN_WIDTH_LG)
        .padding(PADDING_BTN_V)
        .font_size(FONT_BODY)
        .font_family("monospace".to_string())
        .background(Color::WHITE)
        .border(1.0)
        .border_color(BORDER_INPUT)
        .border_radius(RADIUS_STD)
}

/// Primary button style
pub fn button_style(s: floem::style::Style) -> floem::style::Style {
    s.padding_horiz(PADDING_BTN_H)
        .padding_vert(PADDING_BTN_V)
        .font_size(FONT_BODY)
        .background(ACCENT_PRIMARY)
        .color(Color::WHITE)
        .border_radius(RADIUS_STD)
        .cursor(floem::style::CursorStyle::Pointer)
}

/// Secondary button style for import actions
pub fn secondary_button_style(s: floem::style::Style) -> floem::style::Style {
    s.padding_horiz(PADDING_BTN_H)
        .padding_vert(PADDING_BTN_V)
        .font_size(FONT_BODY)
        .background(BG_SECONDARY)
        .color(TEXT_BUTTON)
        .border(1.0)
        .border_color(BORDER_SECONDARY)
        .border_radius(RADIUS_STD)
        .cursor(floem::style::CursorStyle::Pointer)
}

/// Navigation button style for prev/next
pub fn nav_button_style(s: floem::style::Style) -> floem::style::Style {
    s.padding_horiz(PADDING_STD)
        .padding_vert(4.0)
        .font_size(FONT_BODY)
        .background(BG_NAV_BUTTON)
        .border_radius(RADIUS_SM)
        .cursor(floem::style::CursorStyle::Pointer)
}

/// Selector display style
pub fn selector_display_style(s: floem::style::Style) -> floem::style::Style {
    s.flex_grow(1.0)
        .padding(PADDING_BTN_V)
        .font_size(FONT_BODY)
        .font_family("monospace".to_string())
        .background(Color::WHITE)
        .border(1.0)
        .border_color(BORDER_SECONDARY)
        .border_radius(RADIUS_SM)
}
