//! Index Search tab callbacks

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::MacPakApp;

/// Register all Search tab callbacks
pub fn register_callbacks(app: &MacPakApp) {
    register_search_execute(app);
    register_index_pak(app);
    register_index_directory(app);
    register_clear_index(app);
}

fn register_search_execute(app: &MacPakApp) {
    app.on_search_execute({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let query = app.get_search_query().to_string();
                tracing::info!("Search: {}", query);

                app.set_search_is_searching(true);
                app.set_search_status(format!("Searching for '{}'...", query).into());

                // TODO: Implement actual search with file index
                let app_weak2 = app_weak.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_weak2.upgrade() {
                            app.set_search_status("Search complete (index not yet implemented)".into());
                            app.set_search_is_searching(false);
                        }
                    }).unwrap();
                });
            }
        }
    });
}

fn register_index_pak(app: &MacPakApp) {
    app.on_search_index_pak({
        let app_weak = app.as_weak();
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Select PAK to Index")
                .add_filter("PAK Files", &["pak"]);

            if let Some(path) = dialog.pick_file() {
                if let Some(app) = app_weak.upgrade() {
                    tracing::info!("Index PAK: {:?}", path);
                    app.set_search_status(format!("Indexing {:?}...", path.file_name().unwrap_or_default()).into());
                    // TODO: Implement indexing
                }
            }
        }
    });
}

fn register_index_directory(app: &MacPakApp) {
    app.on_search_index_directory({
        let app_weak = app.as_weak();
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Select Directory to Index");

            if let Some(path) = dialog.pick_folder() {
                if let Some(app) = app_weak.upgrade() {
                    tracing::info!("Index directory: {:?}", path);
                    app.set_search_status(format!("Indexing {:?}...", path).into());
                    // TODO: Implement indexing
                }
            }
        }
    });
}

fn register_clear_index(app: &MacPakApp) {
    app.on_search_clear_index({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                tracing::info!("Clear search index");
                app.set_search_indexed_files(0);
                app.set_search_indexed_paks(0);
                app.set_search_results(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
                app.set_search_status("Index cleared".into());
            }
        }
    });
}
