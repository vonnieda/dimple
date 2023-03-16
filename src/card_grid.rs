use eframe::{
    egui::{self, Grid, ImageButton, Link, Response, TextEdit, Ui, Widget, Layout, ScrollArea},
    epaint::{FontFamily, FontId}, emath::Align,
};
use egui_extras::RetainedImage;

pub struct CardGrid {

}

impl CardGrid {
    fn new() -> Self {
        Self {

        }
    }
}

impl Widget for CardGrid {
    fn ui(self, ui: &mut Ui) -> Response {
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
    }
}

struct Card {
    image: RetainedImage,
    title: String,
    subtitle: String,
}

impl Card {
    fn new(image: RetainedImage, title: String, subtitle: String) -> Self {
        Self {
            image,
            title,
            subtitle,
        }
    }
}

impl Widget for Card {
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

// search_query: "".to_string(),
// // image8: RetainedImage::from_image_bytes(
// //     "Bild",
// //     include_bytes!("./samples/art/getCoverArt-7.jpg"),
// // )
// // .unwrap(),
