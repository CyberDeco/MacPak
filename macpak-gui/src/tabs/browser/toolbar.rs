//! Browser toolbar with navigation, path input, and filters

use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem::views::PlaceholderTextClass;

use crate::state::BrowserState;
use super::operations::{apply_filters, go_up, load_directory, open_folder_dialog, refresh};

pub fn browser_toolbar(state: BrowserState) -> impl IntoView {
    let state_open = state.clone();
    let state_up = state.clone();
    let state_refresh = state.clone();
    let state_nav = state.clone();
    let state_search = state.clone();
    let state_search_enter = state.clone();
    let path_input = state.browser_path;
    let state_filter = state.clone();
    let state_all = state.clone();
    let state_pak = state.clone();

    // Binary files (LSBC and LSBS not included)
    let state_lsx = state.clone();
    let state_lsj = state.clone();
    let state_lsf = state.clone();
    let state_lsfx = state.clone();
    
    // Image files
    let state_dds = state.clone();

    // Model files
    let state_gr2 = state.clone();
    let state_dae = state.clone();
    let state_gltf = state.clone();

    // Cheat sheet
    // "PAK" => "📦",
    // "LSF" | "LSX" | "LSJ" | "LSFX" | "LSBC" | "LSBS" => "📖",
    // "DDS" | "PNG" | "JPG" | "JPEG" => "🖼️",
    // "GR2" | "DAE" | "glTF" => "🎨",
    // "WEM" | "WAV" => "🔊",
    // "LUA" | "OSI" | "gameScript" | "itemScript" => "📜",
    // "XML" | "TXT" | "KHN" | "TMPL" => "📝",
    // "LOCA" => "🌐",
    // "SHD" | "BSHD" | "METAL" => "✏️",
    // "DAT" | "DATA" | "PATCH" | "CLC" | "CLM" | "CLN" => "🖥️",
    // "ANC" | "ANM" | "ANN" => "🪄"
    
    v_stack((
        // Row 1: Navigation + file path
        h_stack((
            button("⬆️ Up").action(move || {
                go_up(state_up.clone());
            }),
            // Editable file path input - flex_grow to fill remaining space
            text_input(path_input)
                .placeholder("Enter path or open folder...")
                .style(|s| {
                    s.flex_grow(1.0)
                        .flex_basis(0.0)
                        .width_full()
                        .min_width(100.0)
                        .padding(6.0)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                        .class(PlaceholderTextClass, |s| s.color(Color::rgb8(120, 120, 120)))
                })
                .on_key_down(
                    Key::Named(NamedKey::Enter),
                    |_| true,
                    move |_| {
                        let path = state_nav.browser_path.get();
                        if !path.is_empty() {
                            load_directory(&path, state_nav.clone());
                        }
                    },
                ),
            separator(),
            button("📂 Browse...").action(move || {
                open_folder_dialog(state_open.clone());
            }),
            button("🔄 Refresh").action(move || {
                refresh(state_refresh.clone());
            }),
            
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),

        // Row 2: Search + quick filters
        h_stack((
            text_input(state_search.search_query)
                .placeholder("Search files...")
                .style(|s| {
                    s.width(200.0)
                        .padding(6.0)
                        .border(1.0)
                        .border_color(Color::rgb8(200, 200, 200))
                        .border_radius(4.0)
                        .class(PlaceholderTextClass, |s| s.color(Color::rgb8(120, 120, 120)))
                })
                .on_key_down(
                    Key::Named(NamedKey::Enter),
                    |_| true,
                    move |_| {
                        apply_filters(state_search_enter.clone());
                    },
                ),
            button("🔎").action(move || {
                apply_filters(state_filter.clone());
            }),
            separator(),
            label(|| "Quick Filter:").style(|s| s.color(Color::rgb8(100, 100, 100))),
            filter_button("All", state_all),
            separator(),
            filter_button("PAK", state_pak),
            separator(),
            // LSF format filters
            h_stack((
                filter_button("LSX", state_lsx),
                filter_button("LSJ", state_lsj),
                filter_button("LSF", state_lsf),
                filter_button("LSFX", state_lsfx),
            )).style(|s| s.gap(8.0)),
            separator(),
            filter_button("DDS", state_dds),
            separator(),
            // Model format filters
            h_stack((
                filter_button("GR2", state_gr2),
                filter_button("DAE", state_dae),
                filter_button("glTF", state_gltf),
            )).style(|s| s.gap(8.0)),
            empty().style(|s| s.flex_grow(1.0)),
        ))
        .style(|s| s.width_full().gap(8.0).items_center()),
    ))
    .style(|s| {
        s.width_full()
            .padding(10.0)
            .gap(8.0)
            .background(Color::rgb8(245, 245, 245))
            .border_bottom(1.0)
            .border_color(Color::rgb8(220, 220, 220))
    })
}

fn filter_button(filter_type: &'static str, state: BrowserState) -> impl IntoView {
    let current_filter = state.type_filter;
    let state_click = state.clone();

    button(filter_type)
        .style(move |s| {
            let is_active = current_filter.get() == filter_type;
            let s = s.padding_horiz(8.0).padding_vert(4.0).border_radius(4.0);

            if is_active {
                s.background(Color::rgb8(33, 150, 243)).color(Color::WHITE)
            } else {
                s.background(Color::rgb8(230, 230, 230))
                    .color(Color::rgb8(60, 60, 60))
                    .hover(|s| s.background(Color::rgb8(200, 200, 200)))
            }
        })
        .action(move || {
            state_click.type_filter.set(filter_type.to_string());
            apply_filters(state_click.clone());
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
