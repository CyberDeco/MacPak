use eframe::egui;

pub mod tabs;
pub mod widgets;
pub mod dialogs;
pub mod state;

pub struct MacPakApp {
    current_tab: usize,
}

impl MacPakApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_tab: 0,
        }
    }
}

impl eframe::App for MacPakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MacPak - BG3 Modding Toolkit");
            ui.label("GUI coming soon!");
        });
    }
}