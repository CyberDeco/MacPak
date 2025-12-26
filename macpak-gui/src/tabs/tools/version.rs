//! Version Calculator

use floem::prelude::*;
use floem::text::Weight;

use crate::state::ToolsState;
use super::helpers::copy_to_clipboard;
use super::widgets::tool_card_style;

pub fn version_calculator_section(state: ToolsState) -> impl IntoView {
    let version_int = state.version_int;
    let major = state.version_major;
    let minor = state.version_minor;
    let patch = state.version_patch;
    let build = state.version_build;
    let status = state.status_message;

    v_stack((
        label(|| "Version Calculator").style(|s| s.font_size(16.0).font_weight(Weight::BOLD).margin_bottom(12.0)),

        label(|| "BG3 uses Int64 version numbers (Major.Minor.Patch.Build)")
            .style(|s| s.font_size(11.0).color(Color::rgb8(100, 100, 100)).margin_bottom(12.0)),

        // Int64 input
        h_stack((
            label(|| "Int64:").style(|s| s.font_size(12.0).width(50.0)),
            text_input(version_int)
                .placeholder("Enter version number...")
                .style(|s| {
                    s.flex_grow(1.0)
                        .padding(8.0)
                        .font_size(13.0)
                        .font_family("monospace".to_string())
                        .background(Color::rgb8(245, 245, 245))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                }),
            button("Parse")
                .style(|s| s.padding_horiz(12.0).padding_vert(8.0).font_size(12.0))
                .action(move || {
                    let input = version_int.get();
                    if let Ok(v) = input.trim().parse::<u64>() {
                        let (ma, mi, pa, bu) = int_to_version(v);
                        major.set(ma);
                        minor.set(mi);
                        patch.set(pa);
                        build.set(bu);
                        status.set(format!("Parsed: {}.{}.{}.{}", ma, mi, pa, bu));
                    } else {
                        status.set("Invalid number".to_string());
                    }
                }),
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),

        // Version components
        h_stack((
            version_field("Major", major),
            version_field("Minor", minor),
            version_field("Patch", patch),
            version_field("Build", build),
        ))
        .style(|s| s.width_full().gap(8.0).margin_top(12.0)),

        // Readable version display
        h_stack((
            label(move || format!("{}.{}.{}.{}", major.get(), minor.get(), patch.get(), build.get()))
                .style(|s| {
                    s.flex_grow(1.0)
                        .padding(10.0)
                        .font_size(14.0)
                        .font_weight(Weight::BOLD)
                        .font_family("monospace".to_string())
                        .background(Color::rgb8(245, 245, 245))
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                        .justify_center()
                }),
        ))
        .style(|s| s.width_full().margin_top(12.0)),

        // Action buttons
        h_stack((
            {
                let status_copy = status;
                button("Copy Int64")
                    .style(|s| {
                        s.flex_grow(1.0)
                            .padding_vert(10.0)
                            .font_size(12.0)
                            .background(Color::rgb8(255, 152, 0))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .hover(|s| s.background(Color::rgb8(245, 124, 0)))
                    })
                    .action(move || {
                        let v = version_to_int(major.get(), minor.get(), patch.get(), build.get());
                        copy_to_clipboard(&v.to_string());
                        version_int.set(v.to_string());
                        status_copy.set("Copied Int64!".to_string());
                    })
            },
            {
                let status_copy = status;
                button("Copy Readable")
                    .style(|s| {
                        s.flex_grow(1.0)
                            .padding_vert(10.0)
                            .font_size(12.0)
                            .background(Color::rgb8(0, 150, 136))
                            .color(Color::WHITE)
                            .border_radius(4.0)
                            .hover(|s| s.background(Color::rgb8(0, 121, 107)))
                    })
                    .action(move || {
                        let readable = format!("{}.{}.{}.{}", major.get(), minor.get(), patch.get(), build.get());
                        copy_to_clipboard(&readable);
                        status_copy.set("Copied version string!".to_string());
                    })
            },
        ))
        .style(|s| s.width_full().gap(8.0).margin_top(12.0)),
    ))
    .style(|s| tool_card_style(s))
}

fn version_field(label_text: &'static str, value: RwSignal<u32>) -> impl IntoView {
    v_stack((
        label(move || label_text).style(|s| s.font_size(10.0).color(Color::rgb8(100, 100, 100))),
        h_stack((
            button("-")
                .style(|s| s.padding_horiz(8.0).padding_vert(4.0).font_size(12.0))
                .action(move || {
                    let v = value.get();
                    if v > 0 {
                        value.set(v - 1);
                    }
                }),
            label(move || format!("{}", value.get()))
                .style(|s| {
                    s.min_width(30.0)
                        .padding_horiz(6.0)
                        .font_size(13.0)
                        .font_family("monospace".to_string())
                        .justify_center()
                }),
            button("+")
                .style(|s| s.padding_horiz(8.0).padding_vert(4.0).font_size(12.0))
                .action(move || {
                    value.set(value.get() + 1);
                }),
        ))
        .style(|s| s.items_center()),
    ))
    .style(|s| s.items_center())
}

fn version_to_int(major: u32, minor: u32, patch: u32, build: u32) -> u64 {
    ((major as u64) << 55)
        | ((minor as u64) << 47)
        | ((patch as u64) << 31)
        | (build as u64)
}

fn int_to_version(v: u64) -> (u32, u32, u32, u32) {
    let major = ((v >> 55) & 0x1FF) as u32;      // 9 bits
    let minor = ((v >> 47) & 0xFF) as u32;        // 8 bits
    let patch = ((v >> 31) & 0xFFFF) as u32;      // 16 bits
    let build = (v & 0x7FFFFFFF) as u32;          // 31 bits
    (major, minor, patch, build)
}
