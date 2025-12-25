slint::include_modules!();

mod tabs;

use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting MacPak GUI");

    // Create the main window
    let app = MacPakApp::new()?;

    // Shared state for history
    let uuid_history: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let handle_history: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    // Shared state for browser - stores full unfiltered file list
    let all_files: Rc<RefCell<Vec<FileEntry>>> = Rc::new(RefCell::new(Vec::new()));

    // =========================================================================
    // Register Tab Callbacks
    // =========================================================================

    // UUID Generator
    tabs::uuid_generator::register_callbacks(&app, uuid_history.clone(), handle_history.clone());

    // PAK Operations
    tabs::pak_operations::register_callbacks(&app);

    // Universal Editor
    tabs::editor::register_callbacks(&app);

    // Asset Browser
    tabs::asset_browser::register_callbacks(&app, all_files.clone());

    // Index Search
    tabs::search::register_callbacks(&app);

    // =========================================================================
    // Menu/Dialog Callbacks
    // =========================================================================

    app.on_tab_changed({
        move |tab_index| {
            tracing::info!("Tab changed to: {}", tab_index);
        }
    });

    app.on_show_settings({
        move || {
            tracing::info!("Show settings");
            // TODO: Implement settings dialog
        }
    });

    app.on_show_about_dialog({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_show_about(true);
            }
        }
    });

    // Run the application
    tracing::info!("MacPak GUI ready");
    app.run()
}
