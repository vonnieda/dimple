#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{
    egui::{self, Grid, ImageButton, Link, Response, TextEdit, Ui, Widget, Layout, ScrollArea},
    epaint::{FontFamily, FontId}, emath::Align,
};
use egui_extras::RetainedImage;

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native("Music", options, Box::new(|_cc| Box::new(App::default())))
}

// TODO: all this 'a stuff seems wrong, and I don't know why I needed it.
struct Card<'a> {
    image: &'a RetainedImage,
    title: &'a str,
    subtitle: &'a str,
}

impl<'a> Card<'a> {
    fn new(image: &'a RetainedImage, title: &'a str, subtitle: &'a str) -> Self {
        Self {
            image,
            title,
            subtitle,
        }
    }
}

impl<'a> Widget for Card<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.add(
                ImageButton::new(
                    self.image.texture_id(ui.ctx()), 
                    egui::vec2(150.0, 150.0)),
            );
            ui.add_space(4.0);
            ui.add(Link::new(self.title));
            ui.add_space(4.0);
            ui.add(Link::new(self.subtitle));
        })
        .response
    }
}

struct App {
    search_query: String,
    image1: RetainedImage,
    image2: RetainedImage,
    image3: RetainedImage,
    image4: RetainedImage,
    image5: RetainedImage,
    image6: RetainedImage,
    image7: RetainedImage,
    image8: RetainedImage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            search_query: "".to_owned(),
            image1: RetainedImage::from_image_bytes(
                "Bild",
                include_bytes!("./art/getCoverArt.jpg"),
            )
            .unwrap(),
            image2: RetainedImage::from_image_bytes(
                "Bild",
                include_bytes!("./art/getCoverArt-1.jpg"),
            )
            .unwrap(),
            image3: RetainedImage::from_image_bytes(
                "Bild",
                include_bytes!("./art/getCoverArt-2.jpg"),
            )
            .unwrap(),
            image4: RetainedImage::from_image_bytes(
                "Bild",
                include_bytes!("./art/getCoverArt-3.jpg"),
            )
            .unwrap(),
            image5: RetainedImage::from_image_bytes(
                "Bild",
                include_bytes!("./art/getCoverArt-4.jpg"),
            )
            .unwrap(),
            image6: RetainedImage::from_image_bytes(
                "Bild",
                include_bytes!("./art/getCoverArt-5.jpg"),
            )
            .unwrap(),
            image7: RetainedImage::from_image_bytes(
                "Bild",
                include_bytes!("./art/getCoverArt-6.jpg"),
            )
            .unwrap(),
            image8: RetainedImage::from_image_bytes(
                "Bild",
                include_bytes!("./art/getCoverArt-7.jpg"),
            )
            .unwrap(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    TextEdit::singleline(&mut self.search_query)
                        .hint_text("What sounds good?")
                        .font(FontId::new(32.0, FontFamily::Proportional))
                        .desired_width(f32::INFINITY),
                );
            });

            ScrollArea::vertical().show(ui, |ui| {
                let grid = Grid::new("some_unique_id");
                grid.show(ui, |ui| {
                    ui.add(Card::new(&self.image1, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image2, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image3, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image4, "Carrion Skies", "Fen"));
                    ui.end_row();
                    ui.add(Card::new(&self.image5, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image6, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image7, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image8, "Carrion Skies", "Fen"));
                    ui.end_row();
                    ui.add(Card::new(&self.image5, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image6, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image7, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image8, "Carrion Skies", "Fen"));
                    ui.end_row();
                    ui.add(Card::new(&self.image5, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image6, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image7, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image8, "Carrion Skies", "Fen"));
                    ui.end_row();
                    ui.add(Card::new(&self.image5, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image6, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image7, "Carrion Skies", "Fen"));
                    ui.add(Card::new(&self.image8, "Carrion Skies", "Fen"));
                    ui.end_row();
                });
            })
        });
    }
}

// ui.heading("My egui Application");
// ui.horizontal(|ui| {
//     let name_label = ui.label("Your name: ");
//     ui.text_edit_singleline(&mut self.name)
//         .labelled_by(name_label.id);
// });
// ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
// if ui.button("Click each year").clicked() {
//     self.age += 1;
// }
// ui.label(format!("Hello '{}', age {}", self.name, self.age));

// struct SearchBar {
//     query: String,
// }

// impl Default for SearchBar {
//     fn default() -> Self {
//         Self { query: "".to_string() }
//     }
// }

// impl SearchBar {
//     fn ui(&mut self, ui: &mut Ui) {
//         ui.horizontal(|ui| {
//             ui.add(egui::widgets::Label::new("Search")); // .text_color(egui::Color32::GRAY)
//             ui.spacing_mut().item_spacing.x = 4.0;
//             ui.add(
//                 egui::widgets::TextEdit::singleline(&mut self.query)
//                     .hint_text("Search")
//                     .desired_width(200.0)
//                     .text_style(egui::TextStyle::Button),
//             )
//             .on_hover_text("Enter your search query");
//         });
//     }
// }

// use eframe::egui; //::{Align2, CtxRef, Key, Sense, Ui, Vec2};
// use eframe::{egui::epi::Frame, epi::App, epi::IntegrationInfo, NativeOptions};

// struct SearchBar {
//     query: String,
// }

// impl Default for SearchBar {
//     fn default() -> Self {
//         Self { query: "".to_string() }
//     }
// }

// impl SearchBar {
//     fn ui(&mut self, ui: &mut Ui) {
//         ui.horizontal(|ui| {
//             ui.add(egui::widgets::Label::new("Search").text_color(egui::Color32::GRAY));
//             ui.spacing_mut().item_spacing.x = 4.0;
//             ui.add(
//                 egui::widgets::TextEdit::singleline(&mut self.query)
//                     .hint_text("Search")
//                     .desired_width(200.0)
//                     .text_style(egui::TextStyle::Button),
//             )
//             .on_hover_text("Enter your search query");
//         });
//     }
// }
