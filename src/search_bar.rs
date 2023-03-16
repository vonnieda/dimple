use eframe::{
    egui::{Response, TextEdit, Ui, Widget},
    epaint::{FontFamily, FontId},
};

pub struct SearchBar {
    query: String,
    hint: String,
}

impl Default for SearchBar {
    fn default() -> Self {
        Self { 
            query: "".to_owned(),
            hint: String::from("What sounds good?"),
        }
    }
}

impl Widget for SearchBar {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            // ui.add(
            //     TextEdit::singleline(&mut self.query)
            //         .hint_text(&self.hint)
            //         .font(FontId::new(32.0, FontFamily::Proportional))
            //         .desired_width(f32::INFINITY),
            // );
        }).response
    }
}

