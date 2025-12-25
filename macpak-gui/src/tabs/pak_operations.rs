//! PAK Operations tab callbacks

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::MacPakApp;

/// Register all PAK Operations tab callbacks
pub fn register_callbacks(app: &MacPakApp) {
    register_extract_callbacks(app);
    register_create_callbacks(app);
    register_list_callbacks(app);
}

fn register_extract_callbacks(app: &MacPakApp) {
    app.on_pak_browse_extract_source({
        let app_weak = app.as_weak();
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Select PAK File to Extract")
                .add_filter("PAK Files", &["pak"]);

            if let Some(path) = dialog.pick_file() {
                if let Some(app) = app_weak.upgrade() {
                    app.set_pak_extract_source(path.to_string_lossy().to_string().into());
                }
            }
        }
    });

    app.on_pak_browse_extract_dest({
        let app_weak = app.as_weak();
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Select Extraction Destination");

            if let Some(path) = dialog.pick_folder() {
                if let Some(app) = app_weak.upgrade() {
                    app.set_pak_extract_dest(path.to_string_lossy().to_string().into());
                }
            }
        }
    });

    app.on_pak_extract({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let source = app.get_pak_extract_source().to_string();
                let dest = app.get_pak_extract_dest().to_string();

                tracing::info!("Extract PAK: {} -> {}", source, dest);
                app.set_pak_is_extracting(true);
                app.set_pak_extract_status("Extracting...".into());
                app.set_pak_extract_progress(0.0);

                let app_weak2 = app_weak.clone();
                std::thread::spawn(move || {
                    // Use MacLarian to extract
                    let result = MacLarian::pak::PakOperations::extract(&source, &dest);

                    slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_weak2.upgrade() {
                            match result {
                                Ok(_) => {
                                    app.set_pak_extract_progress(100.0);
                                    app.set_pak_extract_status("Extraction complete!".into());
                                    tracing::info!("PAK extraction complete");
                                }
                                Err(e) => {
                                    app.set_error_message(format!("Extraction failed: {}", e).into());
                                    app.set_show_error(true);
                                    tracing::error!("PAK extraction failed: {}", e);
                                }
                            }
                            app.set_pak_is_extracting(false);
                        }
                    }).unwrap();
                });
            }
        }
    });
}

fn register_create_callbacks(app: &MacPakApp) {
    app.on_pak_browse_create_source({
        let app_weak = app.as_weak();
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Select Mod Directory");

            if let Some(path) = dialog.pick_folder() {
                if let Some(app) = app_weak.upgrade() {
                    app.set_pak_create_source(path.to_string_lossy().to_string().into());
                }
            }
        }
    });

    app.on_pak_browse_create_dest({
        let app_weak = app.as_weak();
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Save PAK File As")
                .add_filter("PAK Files", &["pak"]);

            if let Some(path) = dialog.save_file() {
                if let Some(app) = app_weak.upgrade() {
                    app.set_pak_create_dest(path.to_string_lossy().to_string().into());
                }
            }
        }
    });

    app.on_pak_create({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let source = app.get_pak_create_source().to_string();
                let dest = app.get_pak_create_dest().to_string();

                tracing::info!("Create PAK: {} -> {}", source, dest);
                app.set_pak_is_creating(true);
                app.set_pak_create_status("Creating PAK...".into());
                app.set_pak_create_progress(0.0);

                let app_weak2 = app_weak.clone();
                std::thread::spawn(move || {
                    let result = MacLarian::pak::PakOperations::create(&source, &dest);

                    slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_weak2.upgrade() {
                            match result {
                                Ok(_) => {
                                    app.set_pak_create_progress(100.0);
                                    app.set_pak_create_status("PAK created successfully!".into());
                                    tracing::info!("PAK creation complete");
                                }
                                Err(e) => {
                                    app.set_error_message(format!("PAK creation failed: {}", e).into());
                                    app.set_show_error(true);
                                    tracing::error!("PAK creation failed: {}", e);
                                }
                            }
                            app.set_pak_is_creating(false);
                        }
                    }).unwrap();
                });
            }
        }
    });
}

fn register_list_callbacks(app: &MacPakApp) {
    app.on_pak_browse_list_source({
        let app_weak = app.as_weak();
        move || {
            let dialog = rfd::FileDialog::new()
                .set_title("Select PAK File")
                .add_filter("PAK Files", &["pak"]);

            if let Some(path) = dialog.pick_file() {
                if let Some(app) = app_weak.upgrade() {
                    app.set_pak_list_source(path.to_string_lossy().to_string().into());
                    // Clear previous contents
                    app.set_pak_list_contents(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
                }
            }
        }
    });

    app.on_pak_list({
        let app_weak = app.as_weak();
        move || {
            if let Some(app) = app_weak.upgrade() {
                let source = app.get_pak_list_source().to_string();

                tracing::info!("List PAK contents: {}", source);
                app.set_pak_is_listing(true);

                let app_weak2 = app_weak.clone();
                std::thread::spawn(move || {
                    let result = MacLarian::pak::PakOperations::list(&source);

                    slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_weak2.upgrade() {
                            match result {
                                Ok(files) => {
                                    let file_list: Vec<SharedString> = files
                                        .iter()
                                        .map(|s| SharedString::from(s.as_str()))
                                        .collect();
                                    app.set_pak_list_contents(ModelRc::new(VecModel::from(file_list)));
                                    tracing::info!("Listed {} files from PAK", files.len());
                                }
                                Err(e) => {
                                    app.set_error_message(format!("Failed to list PAK: {}", e).into());
                                    app.set_show_error(true);
                                    tracing::error!("PAK listing failed: {}", e);
                                }
                            }
                            app.set_pak_is_listing(false);
                        }
                    }).unwrap();
                });
            }
        }
    });
}
