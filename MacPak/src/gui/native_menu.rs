//! Native macOS menu using muda
//!
//! Creates the application menu bar with standard macOS items plus Preferences.

use floem::prelude::*;
use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};

use crate::gui::state::{ConfigState, EditorTabsState};

/// Menu item IDs for event handling
static PREFERENCES_ID: std::sync::OnceLock<muda::MenuId> = std::sync::OnceLock::new();

/// Config state for menu event handler
static CONFIG_STATE: std::sync::OnceLock<ConfigState> = std::sync::OnceLock::new();

/// Set up the native macOS menu bar
///
/// This spawns a thread that waits for the app_ready signal, then dispatches
/// to the main thread to replace floem's default menu with our custom menu.
pub fn setup_native_menu(
    _editor_tabs_state: EditorTabsState,
    _active_tab: RwSignal<usize>,
    config_state: ConfigState,
) {
    // Store config state for later use by menu event handler
    let _ = CONFIG_STATE.set(config_state.clone());

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
fn create_menu_bar(_config_state: ConfigState) {

    // Create the menu bar
    let menu_bar = Menu::new();

    // App submenu (MacPak)
    let app_submenu = Submenu::new("MacPak", true);

    // About MacPak
    let _ = app_submenu.append(&PredefinedMenuItem::about(
        Some("MacPak"),
        None,
    ));

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

    // Hide Others
    let _ = app_submenu.append(&PredefinedMenuItem::hide_others(None));

    // Show All
    let _ = app_submenu.append(&PredefinedMenuItem::show_all(None));

    let _ = app_submenu.append(&PredefinedMenuItem::separator());

    // Quit MacPak
    let _ = app_submenu.append(&PredefinedMenuItem::quit(None));

    // Add app submenu to menu bar
    let _ = menu_bar.append(&app_submenu);

    // Initialize as macOS app menu
    #[cfg(target_os = "macos")]
    menu_bar.init_for_nsapp();

    // Prevent menu from being dropped (NSApp retains a reference)
    std::mem::forget(menu_bar);

    // Spawn a thread to handle menu events
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                if let Some(prefs_id) = PREFERENCES_ID.get() {
                    if &event.id == prefs_id {
                        // Dispatch to main thread to toggle preferences dialog
                        #[cfg(target_os = "macos")]
                        dispatch::Queue::main().exec_async(|| {
                            if let Some(cfg) = CONFIG_STATE.get() {
                                // Toggle the dialog
                                let current = cfg.show_dialog.get();
                                cfg.show_dialog.set(!current);
                            }
                        });
                    }
                }
            }
        }
    });
}
