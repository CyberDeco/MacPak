//! Native macOS menu using muda
//!
//! Creates the application menu bar with standard macOS items plus Preferences.

use floem::prelude::*;
use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::gui::state::{ConfigState, EditorTabsState};
use crate::gui::tabs::editor::open_file_at_path;

/// Menu item IDs for event handling
static PREFERENCES_ID: std::sync::OnceLock<muda::MenuId> = std::sync::OnceLock::new();
static CLEAR_RECENT_ID: std::sync::OnceLock<muda::MenuId> = std::sync::OnceLock::new();

/// Map of menu item IDs to file paths for recent files
static RECENT_FILE_IDS: std::sync::OnceLock<Mutex<HashMap<muda::MenuId, String>>> =
    std::sync::OnceLock::new();

/// Config state for menu event handler
static CONFIG_STATE: std::sync::OnceLock<ConfigState> = std::sync::OnceLock::new();

/// Editor tabs state for opening files
static EDITOR_TABS_STATE: std::sync::OnceLock<EditorTabsState> = std::sync::OnceLock::new();

/// Active tab signal for switching to editor
static ACTIVE_TAB: std::sync::OnceLock<RwSignal<usize>> = std::sync::OnceLock::new();

/// Set up the native macOS menu bar
///
/// This spawns a thread that waits for the app_ready signal, then dispatches
/// to the main thread to replace floem's default menu with our custom menu.
pub fn setup_native_menu(
    editor_tabs_state: EditorTabsState,
    active_tab: RwSignal<usize>,
    config_state: ConfigState,
) {
    // Store states for later use by menu event handler
    let _ = CONFIG_STATE.set(config_state.clone());
    let _ = EDITOR_TABS_STATE.set(editor_tabs_state);
    let _ = ACTIVE_TAB.set(active_tab);
    let _ = RECENT_FILE_IDS.set(Mutex::new(HashMap::new()));

    // Spawn a thread to set up the menu after app is ready
    std::thread::spawn(move || {
        // Poll for app_ready signal (check every 50ms, timeout after 5 seconds)
        let mut attempts = 0;
        while !config_state.is_ready() && attempts < 100 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            attempts += 1;
        }

        // Small additional delay to ensure floem's menu is fully initialized
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Dispatch to main thread using macOS Grand Central Dispatch
        #[cfg(target_os = "macos")]
        dispatch::Queue::main().exec_async(|| {
            if let Some(config_state) = CONFIG_STATE.get() {
                create_menu_bar(config_state.clone());
            }
        });
    });
}

/// Create and initialize the menu bar (must be called on main thread)
fn create_menu_bar(config_state: ConfigState) {
    // Create the menu bar
    let menu_bar = Menu::new();

    // ============ App submenu (MacPak) ============
    let app_submenu = Submenu::new("MacPak", true);

    // About MacPak
    let _ = app_submenu.append(&PredefinedMenuItem::about(Some("MacPak"), None));
    let _ = app_submenu.append(&PredefinedMenuItem::separator());

    // Preferences (CMD+,)
    let preferences_item = MenuItem::new(
        "Preferences...",
        true,
        Some(Accelerator::new(Some(Modifiers::META), Code::Comma)),
    );
    let _ = PREFERENCES_ID.set(preferences_item.id().clone());
    let _ = app_submenu.append(&preferences_item);

    let _ = app_submenu.append(&PredefinedMenuItem::separator());

    // Services submenu
    let _ = app_submenu.append(&PredefinedMenuItem::services(None));
    let _ = app_submenu.append(&PredefinedMenuItem::separator());

    // Hide MacPak
    let _ = app_submenu.append(&PredefinedMenuItem::hide(None));
    let _ = app_submenu.append(&PredefinedMenuItem::hide_others(None));
    let _ = app_submenu.append(&PredefinedMenuItem::show_all(None));
    let _ = app_submenu.append(&PredefinedMenuItem::separator());

    // Quit MacPak
    let _ = app_submenu.append(&PredefinedMenuItem::quit(None));

    let _ = menu_bar.append(&app_submenu);

    // ============ File submenu ============
    let file_submenu = Submenu::new("File", true);

    // Open Recent submenu
    let recent_submenu = Submenu::new("Open Recent", true);
    let recent_files = config_state.recent_files.get();

    if recent_files.is_empty() {
        // Show disabled "No Recent Files" item
        let no_recent = MenuItem::new("No Recent Files", false, None::<Accelerator>);
        let _ = recent_submenu.append(&no_recent);
    } else {
        // Add recent files
        if let Some(map) = RECENT_FILE_IDS.get() {
            if let Ok(mut map) = map.lock() {
                map.clear();
                for path in &recent_files {
                    // Show just the filename in the menu
                    let display_name = std::path::Path::new(path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.clone());

                    let item = MenuItem::new(&display_name, true, None::<Accelerator>);
                    map.insert(item.id().clone(), path.clone());
                    let _ = recent_submenu.append(&item);
                }
            }
        }

        let _ = recent_submenu.append(&PredefinedMenuItem::separator());

        // Clear Recent Files
        let clear_recent = MenuItem::new("Clear Menu", true, None::<Accelerator>);
        let _ = CLEAR_RECENT_ID.set(clear_recent.id().clone());
        let _ = recent_submenu.append(&clear_recent);
    }

    let _ = file_submenu.append(&recent_submenu);
    let _ = menu_bar.append(&file_submenu);

    // Initialize as macOS app menu
    #[cfg(target_os = "macos")]
    menu_bar.init_for_nsapp();

    // Prevent menu from being dropped (NSApp retains a reference)
    std::mem::forget(menu_bar);

    // Spawn a thread to handle menu events
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                // Check for Preferences
                if let Some(prefs_id) = PREFERENCES_ID.get() {
                    if &event.id == prefs_id {
                        #[cfg(target_os = "macos")]
                        dispatch::Queue::main().exec_async(|| {
                            if let Some(cfg) = CONFIG_STATE.get() {
                                let current = cfg.show_dialog.get();
                                cfg.show_dialog.set(!current);
                            }
                        });
                        continue;
                    }
                }

                // Check for Clear Recent
                if let Some(clear_id) = CLEAR_RECENT_ID.get() {
                    if &event.id == clear_id {
                        #[cfg(target_os = "macos")]
                        dispatch::Queue::main().exec_async(|| {
                            if let Some(cfg) = CONFIG_STATE.get() {
                                cfg.clear_recent_files();
                            }
                        });
                        continue;
                    }
                }

                // Check for recent file click
                if let Some(map) = RECENT_FILE_IDS.get() {
                    if let Ok(map) = map.lock() {
                        if let Some(path) = map.get(&event.id) {
                            let path = path.clone();
                            #[cfg(target_os = "macos")]
                            dispatch::Queue::main().exec_async(move || {
                                if let (Some(editor_state), Some(active_tab)) =
                                    (EDITOR_TABS_STATE.get(), ACTIVE_TAB.get())
                                {
                                    // Switch to Editor tab
                                    active_tab.set(1);
                                    // Open the file
                                    open_file_at_path(editor_state.clone(), &path);
                                }
                            });
                        }
                    }
                }
            }
        }
    });
}
