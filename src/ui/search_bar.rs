use eframe::{egui::{Context, Ui, TextEdit, Response}, epaint::{FontId, FontFamily}};

#[derive(Default)]
pub struct SearchBar {
    pub query: String,
}

impl SearchBar {
    pub fn ui(&mut self, _ctx: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(move |ui| {
            ui.add(
                TextEdit::singleline(&mut self.query)
                    .hint_text("What sounds good?")
                    .font(FontId::new(32.0, FontFamily::Proportional))
                    .desired_width(f32::INFINITY),
            )
        }).inner
    }
}

