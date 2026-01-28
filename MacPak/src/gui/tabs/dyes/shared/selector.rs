//! Reusable selector components and utilities for dye lists

use super::constants::*;
use super::styles::{nav_button_style, selector_display_style};
use floem::prelude::*;
use floem::text::Weight;

/// Calculate previous index with wrap-around
pub fn prev_index(current: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let curr = current.unwrap_or(0);
    Some(if curr == 0 { len - 1 } else { curr - 1 })
}

/// Calculate next index with wrap-around
pub fn next_index(current: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some((current.unwrap_or(0) + 1) % len)
}

/// Style for green selector container (generated dyes, LSF imports)
pub fn selector_container_green(s: floem::style::Style) -> floem::style::Style {
    s.width_full()
        .padding(PADDING_STD)
        .margin_bottom(PADDING_STD)
        .gap(GAP_STD)
        .items_center()
        .background(BG_SUCCESS)
        .border(1.0)
        .border_color(BORDER_SUCCESS)
        .border_radius(RADIUS_STD)
}

/// Style for gray selector container (TXT imports)
pub fn selector_container_gray(s: floem::style::Style) -> floem::style::Style {
    s.width_full()
        .padding(PADDING_STD)
        .margin_bottom(PADDING_STD)
        .gap(GAP_STD)
        .items_center()
        .background(BG_SECONDARY)
        .border(1.0)
        .border_color(BORDER_SECONDARY)
        .border_radius(RADIUS_STD)
}

/// Empty state label style
pub fn empty_state_style(s: floem::style::Style) -> floem::style::Style {
    s.font_size(FONT_BODY)
        .color(TEXT_MUTED)
        .padding(PADDING_STD)
}

/// Selector label prefix style ("Dyes:", "TXT:", "LSF:")
pub fn selector_label_style(s: floem::style::Style) -> floem::style::Style {
    s.font_size(FONT_BODY).font_weight(Weight::SEMIBOLD)
}

/// Navigation row wrapper style
pub fn nav_row_style(s: floem::style::Style) -> floem::style::Style {
    s.flex_grow(1.0).gap(4.0).items_center()
}

/// Creates a navigation row with prev/next buttons and display label
///
/// Arguments:
/// - `selected_index`: Signal for current selection
/// - `item_count`: Signal for total item count
/// - `display_text`: Closure that returns the display text for current selection
/// - `on_prev`: Callback when prev button clicked
/// - `on_next`: Callback when next button clicked
pub fn nav_row<F, P, N>(
    selected_index: RwSignal<Option<usize>>,
    item_count: impl Fn() -> usize + 'static + Clone,
    display_text: F,
    on_prev: P,
    on_next: N,
) -> impl IntoView
where
    F: Fn() -> String + 'static + Clone,
    P: Fn() + 'static + Clone,
    N: Fn() + 'static + Clone,
{
    let item_count_prev = item_count.clone();
    let item_count_next = item_count.clone();
    let on_prev = on_prev.clone();
    let on_next = on_next.clone();

    h_stack((
        // Previous button
        label(|| "<")
            .style(nav_button_style)
            .on_click_stop(move |_| {
                let len = item_count_prev();
                if len > 0 {
                    if let Some(new_idx) = prev_index(selected_index.get(), len) {
                        selected_index.set(Some(new_idx));
                        on_prev();
                    }
                }
            }),
        // Current selection display
        label(display_text).style(selector_display_style),
        // Next button
        {
            let on_next = on_next.clone();
            label(|| ">")
                .style(nav_button_style)
                .on_click_stop(move |_| {
                    let len = item_count_next();
                    if len > 0 {
                        if let Some(new_idx) = next_index(selected_index.get(), len) {
                            selected_index.set(Some(new_idx));
                            on_next();
                        }
                    }
                })
        },
    ))
    .style(nav_row_style)
}
