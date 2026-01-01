//! UI styles shared between import and export sections

use floem::prelude::*;

/// Input field style - fills available width
pub fn input_style(s: floem::style::Style) -> floem::style::Style {
    s.flex_grow(1.0)
        .flex_basis(0.0)
        .width_full()
        .min_width(150.0)
        .padding(6.0)
        .font_size(11.0)
        .font_family("monospace".to_string())
        .background(Color::WHITE)
        .border(1.0)
        .border_color(Color::rgb8(200, 200, 200))
        .border_radius(4.0)
}

/// Primary button style
pub fn button_style(s: floem::style::Style) -> floem::style::Style {
    s.padding_horiz(12.0)
        .padding_vert(6.0)
        .font_size(11.0)
        .background(Color::rgb8(59, 130, 246))
        .color(Color::WHITE)
        .border_radius(4.0)
        .cursor(floem::style::CursorStyle::Pointer)
}

/// Small button style for generate UUID
pub fn small_button_style(s: floem::style::Style) -> floem::style::Style {
    s.padding_horiz(8.0)
        .padding_vert(6.0)
        .font_size(10.0)
        .background(Color::rgb8(107, 114, 128))
        .color(Color::WHITE)
        .border_radius(4.0)
        .cursor(floem::style::CursorStyle::Pointer)
}

/// Secondary button style for import actions
pub fn secondary_button_style(s: floem::style::Style) -> floem::style::Style {
    s.padding_horiz(12.0)
        .padding_vert(6.0)
        .font_size(11.0)
        .background(Color::rgb8(243, 244, 246))
        .color(Color::rgb8(55, 65, 81))
        .border(1.0)
        .border_color(Color::rgb8(209, 213, 219))
        .border_radius(4.0)
        .cursor(floem::style::CursorStyle::Pointer)
}

/// Navigation button style for prev/next
pub fn nav_button_style(s: floem::style::Style) -> floem::style::Style {
    s.padding_horiz(8.0)
        .padding_vert(4.0)
        .font_size(11.0)
        .background(Color::rgb8(229, 231, 235))
        .border_radius(3.0)
        .cursor(floem::style::CursorStyle::Pointer)
}

/// Selector display style
pub fn selector_display_style(s: floem::style::Style) -> floem::style::Style {
    s.flex_grow(1.0)
        .padding(6.0)
        .font_size(11.0)
        .font_family("monospace".to_string())
        .background(Color::WHITE)
        .border(1.0)
        .border_color(Color::rgb8(209, 213, 219))
        .border_radius(3.0)
}
