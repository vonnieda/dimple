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
                    // TODO how do I use the theme font? Or the default, more specifically.
                    .font(FontId::new(28.0, FontFamily::Proportional))
                    .desired_width(f32::INFINITY),
            )
        }).inner
    }
}

