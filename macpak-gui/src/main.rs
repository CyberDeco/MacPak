use eframe::egui;

mod app;
mod theme;
mod config;

use app::MacPakApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "MacPak - BG3 Modding Toolkit",
        options,
        Box::new(|cc| Box::new(MacPakApp::new(cc))),
    )
}