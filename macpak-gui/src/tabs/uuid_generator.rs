//! UUID and Handle Generator tab callbacks

use std::cell::RefCell;
use std::rc::Rc;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::MacPakApp;

/// Register all UUID Generator tab callbacks
pub fn register_callbacks(
    app: &MacPakApp,
    uuid_history: Rc<RefCell<Vec<String>>>,
    handle_history: Rc<RefCell<Vec<String>>>,
) {
    register_uuid_generate(app, uuid_history.clone());
    register_handle_generate(app, handle_history.clone());
    register_copy_to_clipboard(app);
    register_export_history(app, uuid_history.clone(), handle_history.clone());
    register_clear_history(app, uuid_history, handle_history);
}

fn register_uuid_generate(app: &MacPakApp, history: Rc<RefCell<Vec<String>>>) {
    app.on_uuid_generate({
        let app_weak = app.as_weak();
        move || {
            let uuid = uuid::Uuid::new_v4().to_string().to_uppercase();
            tracing::info!("Generated UUID: {}", uuid);

            if let Some(app) = app_weak.upgrade() {
                app.set_last_uuid(uuid.clone().into());

                // Add to history
                history.borrow_mut().push(uuid.clone());

                // Update the UI history model
                let history_vec: Vec<SharedString> = history
                    .borrow()
                    .iter()
                    .map(|s| SharedString::from(s.as_str()))
                    .collect();
                app.set_uuid_history(ModelRc::new(VecModel::from(history_vec)));
            }
        }
    });
}

fn register_handle_generate(app: &MacPakApp, history: Rc<RefCell<Vec<String>>>) {
    app.on_handle_generate({
        let app_weak = app.as_weak();
        move || {
            let handle = rand::random::<u64>();
            let handle_str = handle.to_string();
            tracing::info!("Generated handle: h{}", handle_str);

            if let Some(app) = app_weak.upgrade() {
                app.set_last_handle(handle_str.clone().into());

                // Add to history
                history.borrow_mut().push(handle_str.clone());

                // Update the UI history model
                let history_vec: Vec<SharedString> = history
                    .borrow()
                    .iter()
                    .map(|s| SharedString::from(s.as_str()))
                    .collect();
                app.set_handle_history(ModelRc::new(VecModel::from(history_vec)));
            }
        }
    });
}

fn register_copy_to_clipboard(app: &MacPakApp) {
    app.on_copy_to_clipboard({
        let app_weak = app.as_weak();
        move |value| {
            let value_str = value.to_string();
            tracing::info!("Copy to clipboard: {}", value_str);

            // Use pbcopy on macOS
            #[cfg(target_os = "macos")]
            {
                use std::process::Command;
                let _ = Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        use std::io::Write;
                        if let Some(stdin) = child.stdin.as_mut() {
                            stdin.write_all(value_str.as_bytes())?;
                        }
                        child.wait()
                    });
            }

            if let Some(app) = app_weak.upgrade() {
                app.set_copy_status("Copied!".into());

                // Clear the status after a delay
                let app_weak2 = app.as_weak();
                slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
                    if let Some(app) = app_weak2.upgrade() {
                        app.set_copy_status("".into());
                    }
                });
            }
        }
    });
}

fn register_export_history(
    app: &MacPakApp,
    uuids: Rc<RefCell<Vec<String>>>,
    handles: Rc<RefCell<Vec<String>>>,
) {
    app.on_uuid_export_history({
        let app_weak = app.as_weak();
        move || {
            tracing::info!("Export history to JSON");

            let uuids = uuids.borrow();
            let handles = handles.borrow();

            let json = serde_json::json!({
                "uuids": *uuids,
                "handles": handles.iter().map(|h| format!("h{}", h)).collect::<Vec<_>>()
            });

            // Open save dialog
            let dialog = rfd::FileDialog::new()
                .set_title("Export History")
                .add_filter("JSON", &["json"])
                .set_file_name("macpak_ids.json");

            if let Some(path) = dialog.save_file() {
                match std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()) {
                    Ok(_) => {
                        tracing::info!("Exported to {:?}", path);
                        if let Some(app) = app_weak.upgrade() {
                            app.set_copy_status("Exported!".into());
                            let app_weak2 = app.as_weak();
                            slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
                                if let Some(app) = app_weak2.upgrade() {
                                    app.set_copy_status("".into());
                                }
                            });
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to export: {}", e);
                        if let Some(app) = app_weak.upgrade() {
                            app.set_error_message(format!("Failed to export: {}", e).into());
                            app.set_show_error(true);
                        }
                    }
                }
            }
        }
    });
}

fn register_clear_history(
    app: &MacPakApp,
    uuids: Rc<RefCell<Vec<String>>>,
    handles: Rc<RefCell<Vec<String>>>,
) {
    app.on_uuid_clear_history({
        let app_weak = app.as_weak();
        move || {
            tracing::info!("Clear history");
            uuids.borrow_mut().clear();
            handles.borrow_mut().clear();

            if let Some(app) = app_weak.upgrade() {
                app.set_uuid_history(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
                app.set_handle_history(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
            }
        }
    });
}
