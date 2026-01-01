//! MacPak GUI - BG3 Modding Toolkit
//!
//! A native macOS application for Baldur's Gate 3 modding, built with Floem.
//! Features include:
//! - Universal Editor for LSX/LSJ/LSF files
//! - Asset Browser for PAK file contents
//! - PAK extraction and creation
//! - Index search across game files
//! - UUID generator for modding

mod native_menu;
pub mod state;
pub mod tabs;
pub mod utils;

use floem::event::{Event, EventListener};
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::prelude::*;
use floem::Application;
use floem::window::WindowConfig;

use state::*;
use tabs::*;
use tabs::editor::{open_file_dialog, save_file};

/// Run the MacPak GUI application
pub fn run_app() {
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
    let gr2_state = Gr2State::new();
    let vt_state = VirtualTexturesState::new();
    let dyes_state = DyesState::new();

    let active_tab = app_state.active_tab;

    // Set up native macOS menu with CMD+F shortcut
    native_menu::setup_native_menu(editor_tabs_state.clone(), active_tab);

    let editor_tabs_for_keyboard = editor_tabs_state.clone();
    let editor_tabs_for_close = editor_tabs_state.clone();

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
        ),
    ))
    .style(|s| s.width_full().height_full())
    .window_title(|| "MacPak".to_string())
    .on_event(EventListener::WindowClosed, move |_| {
        use floem::event::EventPropagation;

        // Check for unsaved changes before closing
        let should_quit = if editor_tabs_for_close.has_unsaved_changes() {
            let response = rfd::MessageDialog::new()
                .set_title("Unsaved Changes")
                .set_description("You have unsaved changes. Are you sure you want to quit?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show();
            response == rfd::MessageDialogResult::Yes
        } else {
            true
        };

        if should_quit {
            // Kill any running preview process before exiting
            kill_preview_process();
            // Force quit the app when the window is closed (macOS behavior fix)
            // Using process::exit because quit_app() alone doesn't terminate
            // background threads or cleanup all resources on macOS
            std::process::exit(0);
        }

        EventPropagation::Stop
    })
    .on_event_cont(EventListener::KeyDown, move |e| {
        // Global keyboard shortcuts for Editor tab
        if let Event::KeyDown(key_event) = e {
            let is_cmd_or_ctrl = key_event.modifiers.contains(Modifiers::META)
                || key_event.modifiers.contains(Modifiers::CONTROL);

            // Only handle when Editor tab is active (tab index 1)
            if active_tab.get() != 1 {
                return;
            }

            // CMD+F / Ctrl+F - Find
            let is_named_find = key_event.key.logical_key == Key::Named(NamedKey::Find);
            let is_f_key = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str().eq_ignore_ascii_case("f")
            );
            if is_named_find || (is_cmd_or_ctrl && is_f_key) {
                if let Some(tab) = editor_tabs_for_keyboard.active_tab() {
                    tab.search_visible.set(!tab.search_visible.get());
                }
                return;
            }

            // CMD+O / Ctrl+O - Open
            let is_o_key = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str().eq_ignore_ascii_case("o")
            );
            if is_cmd_or_ctrl && is_o_key {
                open_file_dialog(editor_tabs_for_keyboard.clone());
                return;
            }

            // CMD+S / Ctrl+S - Save
            let is_s_key = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str().eq_ignore_ascii_case("s")
            );
            if is_cmd_or_ctrl && is_s_key {
                if let Some(tab) = editor_tabs_for_keyboard.active_tab() {
                    // Only save if modified and not converted from LSF
                    if tab.modified.get() && !tab.converted_from_lsf.get() {
                        save_file(tab);
                    }
                }
            }
        }
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
