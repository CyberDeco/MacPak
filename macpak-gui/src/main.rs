//! MacPak - BG3 Modding Toolkit
//!
//! A native macOS application for Baldur's Gate 3 modding, built with Floem.
//! Features include:
//! - Universal Editor for LSX/LSJ/LSF files
//! - Asset Browser for PAK file contents
//! - PAK extraction and creation
//! - Index search across game files
//! - UUID generator for modding

mod state;
mod tabs;
mod utils;

use floem::prelude::*;
use floem::Application;
use floem::window::WindowConfig;

use state::*;
use tabs::*;

fn main() {
    Application::new()
        .window(
            move |_| app_view(),
            Some(
                WindowConfig::default()
                    .size((1200.0, 850.0))
                    .title("MacPak"),
            ),
        )
        .run();
}

fn app_view() -> impl IntoView {
    // Initialize all state
    let app_state = AppState::new();
    let editor_tabs_state = EditorTabsState::new();
    let browser_state = BrowserState::new();
    let pak_ops_state = PakOpsState::new();
    let search_state = SearchState::new();
    let tools_state = ToolsState::new();
    let gr2_state = Gr2State::new();
    let vt_state = VirtualTexturesState::new();
    let dyes_state = DyesState::new();

    let active_tab = app_state.active_tab;

    v_stack((
        // Tab bar
        tab_bar(active_tab),

        // Tab content
        tab_content(
            active_tab,
            app_state,
            editor_tabs_state,
            browser_state,
            pak_ops_state,
            gr2_state,
            vt_state,
            dyes_state,
            search_state,
            tools_state,
        ),
    ))
    .style(|s| s.width_full().height_full())
    .window_title(|| "MacPak".to_string())
    .on_event(floem::event::EventListener::WindowClosed, |_| {
        // Kill any running preview process before exiting
        kill_preview_process();
        // Force quit the app when the window is closed (macOS behavior fix)
        // Using process::exit because quit_app() alone doesn't terminate
        // background threads or cleanup all resources on macOS
        std::process::exit(0);
    })
}

fn tab_bar(active_tab: RwSignal<usize>) -> impl IntoView {
    h_stack((
        tab_button("📂 Browser", 0, active_tab),
        tab_button("📝 Editor", 1, active_tab),
        tab_button("📦 PAK Ops", 2, active_tab),
        tab_button("🦴 GR2", 3, active_tab),
        tab_button("🖼️ Textures", 4, active_tab),
        tab_button("🧪 Dyes", 5, active_tab),
        tab_button("🔍 Search", 6, active_tab),
        tab_button("🛠️ Tools", 7, active_tab),
        empty().style(|s| s.flex_grow(1.0)),
        // App info
        label(|| "MacPak v0.1.0")
            .style(|s| s.color(Color::rgb8(128, 128, 128)).font_size(12.0)),
    ))
    .style(|s| {
        s.width_full()
            .height(44.0)
            .padding_horiz(8.0)
            .gap(4.0)
            .items_center()
            .background(Color::rgb8(38, 38, 38))
    })
}

fn tab_button(label_text: &'static str, index: usize, active_tab: RwSignal<usize>) -> impl IntoView {
    button(label_text)
        .style(move |s| {
            let is_active = active_tab.get() == index;
            let s = s
                .padding_horiz(16.0)
                .padding_vert(8.0)
                .border_radius(6.0)
                .font_size(13.0);

            if is_active {
                s.background(Color::rgb8(60, 60, 60))
                    .color(Color::WHITE)
            } else {
                s.background(Color::TRANSPARENT)
                    .color(Color::rgb8(180, 180, 180))
                    .hover(|s| {
                        s.background(Color::rgb8(50, 50, 50))
                            .color(Color::WHITE)
                    })
            }
        })
        .action(move || {
            active_tab.set(index);
        })
}

fn tab_content(
    active_tab: RwSignal<usize>,
    app_state: AppState,
    editor_tabs_state: EditorTabsState,
    browser_state: BrowserState,
    pak_ops_state: PakOpsState,
    gr2_state: Gr2State,
    vt_state: VirtualTexturesState,
    dyes_state: DyesState,
    search_state: SearchState,
    tools_state: ToolsState,
) -> impl IntoView {
    dyn_container(
        move || active_tab.get(),
        move |tab_index| {
            match tab_index {
                0 => browser_tab(app_state.clone(), browser_state.clone(), editor_tabs_state.clone(), active_tab).into_any(),
                1 => editor_tab(app_state.clone(), editor_tabs_state.clone()).into_any(),
                2 => pak_ops_tab(app_state.clone(), pak_ops_state.clone()).into_any(),
                3 => gr2_tab(app_state.clone(), gr2_state.clone()).into_any(),
                4 => virtual_textures_tab(app_state.clone(), vt_state.clone()).into_any(),
                5 => dyes_tab(app_state.clone(), dyes_state.clone()).into_any(),
                6 => search_tab(app_state.clone(), search_state.clone()).into_any(),
                7 => tools_tab(app_state.clone(), tools_state.clone()).into_any(),
                _ => browser_tab(app_state.clone(), browser_state.clone(), editor_tabs_state.clone(), active_tab).into_any(),
            }
        },
    )
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)  // Allow content to shrink for scroll
            .background(Color::WHITE)
    })
}
