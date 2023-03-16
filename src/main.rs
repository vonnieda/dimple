mod search_bar;
// mod card_grid;

use eframe::egui;
use search_bar::SearchBar;

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1440.0, 1024.0)),
        ..Default::default()
    };
    eframe::run_native("Music", options, Box::new(|_cc| Box::new(App::default())))
}

struct App {
}

impl Default for App {
    fn default() -> Self {
        Self {
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.add(&mut SearchBar::default())
        });
    }
}

