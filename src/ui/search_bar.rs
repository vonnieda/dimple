use eframe::{egui::{Context, Ui, TextEdit, Response, FontDefinitions, TextStyle}, epaint::{FontId, FontFamily, Color32}};

#[derive(Default)]
pub struct SearchBar {
    pub query: String,
}

impl SearchBar {
    pub fn ui(&mut self, _ctx: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let resp = ui.horizontal(move |ui| {
                let resp = ui.add(
                    TextEdit::singleline(&mut self.query)
                        .hint_text("What sounds good?")
                        .desired_width(f32::INFINITY)
                        .font(TextStyle::Heading),
                );
                resp
            }).inner;
            resp
        }).inner
    }
}

