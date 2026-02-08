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
pub mod shared;
pub mod state;
pub mod tabs;
pub mod utils;

use floem::Application;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::prelude::*;
use floem::window::WindowConfig;

use shared::{ThemeColors, init_theme, theme_signal};
use state::*;
use tabs::browser::{cleanup_temp_files, open_folder_dialog};
use tabs::dyes::import_from_mod_folder;
use tabs::editor::{init_config_state, open_file_dialog, save_file};
use tabs::gr2::open_gr2_file;
use tabs::pak_ops::extract_pak_file;
use tabs::virtual_textures::open_gts_file;
use tabs::*;
use utils::config_dialog;

/// Run the MacPak GUI application
pub fn run_app() {
    // Load persisted config for window size
    let persisted = state::PersistedConfig::load();
    let window_width = persisted.window.width;
    let window_height = persisted.window.height;

    Application::new()
        .window(
            move |_| app_view(persisted),
            Some(
                WindowConfig::default()
                    .size((window_width, window_height))
                    .title("MacPak"),
            ),
        )
        .run();
}

fn app_view(persisted: state::PersistedConfig) -> impl IntoView {
    // Initialize all state
    let app_state = AppState::new();
    app_state.active_tab.set(persisted.active_tab);

    let config_state = ConfigState::new();
    init_config_state(config_state.clone()); // For recent files tracking
    init_theme(config_state.theme.get()); // Initialize global theme signal

    let editor_tabs_state = EditorTabsState::new();
    editor_tabs_state.apply_persisted(&persisted.editor);

    let browser_state = BrowserState::new();
    browser_state.apply_persisted(&persisted.browser);

    let pak_ops_state = PakOpsState::new();

    let search_state = SearchState::new();
    search_state.apply_persisted(&persisted.search);

    let gr2_state = Gr2State::new();
    let vt_state = VirtualTexturesState::new();
    let dyes_state = DyesState::new();

    let dialogue_state = DialogueState::new();
    dialogue_state.apply_persisted(&persisted.dialogue);

    let workbench_state = WorkbenchState::new();
    workbench_state.apply_persisted(&persisted.workbench);

    let active_tab = app_state.active_tab;
    let config_state_for_keyboard = config_state.clone();

    // Set up native macOS menu with Preferences
    native_menu::setup_native_menu(editor_tabs_state.clone(), active_tab, config_state.clone());

    let editor_tabs_for_keyboard = editor_tabs_state.clone();
    let editor_tabs_for_close = editor_tabs_state.clone();
    let browser_state_for_keyboard = browser_state.clone();
    let pak_ops_state_for_keyboard = pak_ops_state.clone();
    let gr2_state_for_keyboard = gr2_state.clone();
    let config_state_for_gr2_keyboard = config_state.clone();
    let vt_state_for_keyboard = vt_state.clone();
    let config_state_for_vt_keyboard = config_state.clone();
    let dyes_state_for_keyboard = dyes_state.clone();
    let config_state_for_dialogue = config_state.clone();
    let workbench_state_for_keyboard = workbench_state.clone();

    // Clones for save_session on close
    let app_state_for_close = app_state.clone();
    let browser_state_for_close = browser_state.clone();
    let search_state_for_close = search_state.clone();
    let dialogue_state_for_close = dialogue_state.clone();
    let workbench_state_for_close = workbench_state.clone();
    let config_state_for_close = config_state.clone();

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
            dialogue_state,
            workbench_state,
            config_state_for_dialogue,
        ),
        // Config dialog (overlays when visible)
        config_dialog(config_state.clone()),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .position(floem::style::Position::Relative)
    })
    .window_title(|| "MacPak".to_string())
    // Signal that app is ready once the view is rendered
    .on_event_cont(EventListener::WindowGotFocus, move |_| {
        if !config_state.is_ready() {
            config_state.set_ready();
        }
    })
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
            // Save session state before exiting
            config_state_for_close.save_session(
                &app_state_for_close,
                &editor_tabs_for_close,
                &browser_state_for_close,
                &search_state_for_close,
                &dialogue_state_for_close,
                &workbench_state_for_close,
            );

            // Kill any running preview process before exiting
            kill_preview_process();
            // Clean up temporary files
            cleanup_temp_files();
            // Force quit the app when the window is closed (macOS behavior fix)
            // Using process::exit because quit_app() alone doesn't terminate
            // background threads or cleanup all resources on macOS
            std::process::exit(0);
        }

        EventPropagation::Stop
    })
    .on_event_cont(EventListener::KeyDown, move |e| {
        // Global keyboard shortcuts - context-aware based on active tab
        if let Event::KeyDown(key_event) = e {
            let is_cmd_or_ctrl = key_event.modifiers.contains(Modifiers::META)
                || key_event.modifiers.contains(Modifiers::CONTROL);
            let current_tab = active_tab.get();

            // CMD+F / Ctrl+F - Find (Editor tab only)
            let is_named_find = key_event.key.logical_key == Key::Named(NamedKey::Find);
            let is_f_key = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str().eq_ignore_ascii_case("f")
            );
            if is_named_find || (is_cmd_or_ctrl && is_f_key) {
                if current_tab == 1 {
                    // Editor tab - toggle search panel
                    if let Some(tab) = editor_tabs_for_keyboard.active_tab() {
                        tab.search_visible.set(!tab.search_visible.get());
                    }
                }
                return;
            }

            // CMD+O / Ctrl+O - Open (context-aware)
            let is_o_key = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str().eq_ignore_ascii_case("o")
            );
            if is_cmd_or_ctrl && is_o_key {
                match current_tab {
                    0 => open_folder_dialog(browser_state_for_keyboard.clone()), // Browser - open folder
                    1 => open_file_dialog(editor_tabs_for_keyboard.clone()), // Editor - open file
                    2 => extract_pak_file(pak_ops_state_for_keyboard.clone()), // PAK Ops - extract PAK
                    3 => open_gr2_file(
                        gr2_state_for_keyboard.clone(),
                        config_state_for_gr2_keyboard.clone(),
                    ), // GR2 - open GR2 file
                    4 => open_gts_file(
                        vt_state_for_keyboard.clone(),
                        config_state_for_vt_keyboard.clone(),
                    ), // Textures - open GTS file
                    5 => {
                        // Dyes - import from mod folder
                        // Create temporary signals for display (actual data stored in state)
                        let temp_name = RwSignal::new(String::new());
                        let temp_display = RwSignal::new(String::new());
                        let temp_mod_name = RwSignal::new(String::new());
                        let temp_author = RwSignal::new(String::new());
                        import_from_mod_folder(
                            dyes_state_for_keyboard.clone(),
                            temp_name,
                            temp_display,
                            temp_mod_name,
                            temp_author,
                        );
                    }
                    // Dialogue tab (7) - no CMD+O action, use toolbar buttons instead
                    8 => {
                        // Workbench - open existing project
                        tabs::workbench::open_project_dialog(workbench_state_for_keyboard.clone());
                    }
                    _ => {} // Other tabs - no action
                }
                return;
            }

            // CMD+S / Ctrl+S - Save (Editor tab only)
            let is_s_key = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str().eq_ignore_ascii_case("s")
            );
            if is_cmd_or_ctrl && is_s_key {
                if current_tab == 1 {
                    if let Some(tab) = editor_tabs_for_keyboard.active_tab() {
                        // Only save if modified and not converted from LSF
                        if tab.modified.get() && !tab.converted_from_lsf.get() {
                            save_file(tab);
                        }
                    }
                }
                return;
            }

            // CMD+W / Ctrl+W - Close tab (Editor tab only)
            let is_w_key = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str().eq_ignore_ascii_case("w")
            );
            if is_cmd_or_ctrl && is_w_key {
                if current_tab == 1 {
                    let tabs = editor_tabs_for_keyboard.tabs.get();
                    let active_index = editor_tabs_for_keyboard.active_tab_index.get();
                    if active_index < tabs.len() {
                        editor_tabs_for_keyboard.try_close_tab(active_index);
                    }
                }
                return;
            }

            // CMD+, / Ctrl+, - Preferences (toggle config dialog)
            let is_comma_key = matches!(
                &key_event.key.logical_key,
                Key::Character(c) if c.as_str() == ","
            );
            if is_cmd_or_ctrl && is_comma_key {
                let current = config_state_for_keyboard.show_dialog.get();
                config_state_for_keyboard.show_dialog.set(!current);
                return;
            }

            // Escape - close config dialog if open
            if key_event.key.logical_key == Key::Named(NamedKey::Escape) {
                if config_state_for_keyboard.show_dialog.get() {
                    config_state_for_keyboard.show_dialog.set(false);
                    return;
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
        tab_button("💬 Dialogue", 7, active_tab),
        tab_button("🛠 Workbench", 8, active_tab),
        empty().style(|s| s.flex_grow(1.0)),
        // App info
        label(|| format!("MacPak v{}", env!("CARGO_PKG_VERSION"))).style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.color(colors.text_muted).font_size(12.0)
        }),
    ))
    .style(move |s| {
        let colors = theme_signal()
            .map(|t| ThemeColors::for_theme(t.get().effective()))
            .unwrap_or_else(ThemeColors::dark);
        s.width_full()
            .height(44.0)
            .padding_horiz(8.0)
            .gap(4.0)
            .items_center()
            .background(colors.bg_surface)
    })
}

fn tab_button(
    label_text: &'static str,
    index: usize,
    active_tab: RwSignal<usize>,
) -> impl IntoView {
    button(label_text)
        .style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            let is_active = active_tab.get() == index;
            let s = s
                .padding_horiz(16.0)
                .padding_vert(8.0)
                .border_radius(6.0)
                .font_size(13.0);

            if is_active {
                s.background(colors.bg_elevated).color(colors.text_primary)
            } else {
                s.background(Color::TRANSPARENT)
                    .color(colors.text_secondary)
                    .hover(|s| s.background(colors.bg_hover).color(colors.text_primary))
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
    dialogue_state: DialogueState,
    workbench_state: WorkbenchState,
    config_state: ConfigState,
) -> impl IntoView {
    dyn_container(
        move || active_tab.get(),
        move |tab_index| match tab_index {
            0 => browser_tab(
                app_state.clone(),
                browser_state.clone(),
                editor_tabs_state.clone(),
                active_tab,
                config_state.clone(),
            )
            .into_any(),
            1 => editor_tab(app_state.clone(), editor_tabs_state.clone()).into_any(),
            2 => pak_ops_tab(
                app_state.clone(),
                pak_ops_state.clone(),
                config_state.clone(),
            )
            .into_any(),
            3 => gr2_tab(app_state.clone(), gr2_state.clone(), config_state.clone()).into_any(),
            4 => virtual_textures_tab(app_state.clone(), vt_state.clone(), config_state.clone())
                .into_any(),
            5 => dyes_tab(app_state.clone(), dyes_state.clone()).into_any(),
            6 => search_tab(
                app_state.clone(),
                search_state.clone(),
                config_state.clone(),
                editor_tabs_state.clone(),
                dialogue_state.clone(),
                active_tab,
            )
            .into_any(),
            7 => dialogue_tab(
                app_state.clone(),
                dialogue_state.clone(),
                config_state.clone(),
            )
            .into_any(),
            8 => workbench_tab(
                workbench_state.clone(),
                editor_tabs_state.clone(),
                active_tab,
            )
            .into_any(),
            _ => browser_tab(
                app_state.clone(),
                browser_state.clone(),
                editor_tabs_state.clone(),
                active_tab,
                config_state.clone(),
            )
            .into_any(),
        },
    )
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0) // Allow content to shrink for scroll
            .background(Color::WHITE)
    })
}
