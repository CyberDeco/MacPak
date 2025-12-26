//! History Sections

use floem::prelude::*;
use floem::text::Weight;

use crate::state::ToolsState;
use super::color::parse_hex_color;
use super::helpers::copy_to_clipboard;

pub fn uuid_history_section(state: ToolsState) -> impl IntoView {
    let history = state.uuid_history;
    let status = state.status_message;

    v_stack((
        label(|| "UUID History").style(|s| s.font_size(13.0).font_weight(Weight::BOLD).margin_bottom(6.0)),

        scroll(
            dyn_stack(
                move || history.get(),
                |uuid| uuid.clone(),
                move |uuid| {
                    history_row(uuid, status)
                },
            )
            .style(|s| s.width_full().flex_direction(floem::style::FlexDirection::Column)),
        )
        .style(|s| {
            s.width_full()
                .height(100.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_pct(50.0)
            .padding(12.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
    })
}

pub fn handle_history_section(state: ToolsState) -> impl IntoView {
    let history = state.handle_history;
    let status = state.status_message;

    v_stack((
        label(|| "Handle History").style(|s| s.font_size(13.0).font_weight(Weight::BOLD).margin_bottom(6.0)),

        scroll(
            dyn_stack(
                move || history.get(),
                |handle| handle.clone(),
                move |handle| {
                    let display = format!("h{}", handle);
                    history_row(display, status)
                },
            )
            .style(|s| s.width_full().flex_direction(floem::style::FlexDirection::Column)),
        )
        .style(|s| {
            s.width_full()
                .height(100.0)
                .background(Color::rgb8(250, 250, 250))
                .border(1.0)
                .border_color(Color::rgb8(220, 220, 220))
                .border_radius(4.0)
        }),
    ))
    .style(|s| {
        s.width_pct(50.0)
            .padding(12.0)
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
    })
}

pub fn color_history_section(state: ToolsState) -> impl IntoView {
    let history = state.color_history;
    let status = state.status_message;
    let picker_r = state.color_r;
    let picker_g = state.color_g;
    let picker_b = state.color_b;
    let picker_hex = state.color_hex;

    h_stack((
        label(|| "Color History").style(|s| s.font_size(13.0).font_weight(Weight::BOLD)),

        dyn_container(
            move || history.get().is_empty(),
            move |is_empty| {
                if is_empty {
                    label(|| "No colors saved yet")
                        .style(|s| s.color(Color::rgb8(150, 150, 150)).font_size(12.0))
                        .into_any()
                } else {
                    dyn_stack(
                        move || history.get(),
                        |color| color.clone(),
                        move |color| {
                            let hex_str = color.trim_start_matches('#').to_string();
                            let (r, g, b) = parse_hex_color(&hex_str).unwrap_or((128, 128, 128));
                            let hex_for_picker = hex_str.clone();
                            let status_copy = status;

                            empty()
                                .style(move |s| {
                                    s.width(20.0)
                                        .height(20.0)
                                        .border_radius(3.0)
                                        .border(1.0)
                                        .border_color(Color::rgb8(180, 180, 180))
                                        .background(Color::rgb8(r, g, b))
                                        .cursor(floem::style::CursorStyle::Pointer)
                                })
                                .on_click_stop(move |_| {
                                    // Update the color picker with this color
                                    picker_r.set(r);
                                    picker_g.set(g);
                                    picker_b.set(b);
                                    picker_hex.set(hex_for_picker.clone());
                                    status_copy.set("Color loaded".to_string());
                                })
                        },
                    )
                    .style(|s| s.gap(4.0).flex_wrap(floem::style::FlexWrap::Wrap).flex_direction(floem::style::FlexDirection::Row))
                    .into_any()
                }
            },
        ),
    ))
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .gap(12.0)
            .items_center()
            .background(Color::WHITE)
            .border(1.0)
            .border_color(Color::rgb8(220, 220, 220))
            .border_radius(6.0)
            .margin_top(10.0)
    })
}

fn history_row(value: String, status: RwSignal<String>) -> impl IntoView {
    let value_copy = value.clone();

    h_stack((
        label(move || value.clone())
            .style(|s| {
                s.flex_grow(1.0)
                    .font_family("monospace".to_string())
                    .font_size(11.0)
            }),
        button("Copy")
            .style(|s| s.font_size(10.0).padding_horiz(8.0).padding_vert(2.0))
            .action(move || {
                copy_to_clipboard(&value_copy);
                status.set("Copied!".to_string());
            }),
    ))
    .style(|s| {
        s.width_full()
            .padding(6.0)
            .padding_right(18.0)
            .border_bottom(1.0)
            .border_color(Color::rgb8(240, 240, 240))
            .hover(|s| s.background(Color::rgb8(245, 245, 245)))
    })
}
