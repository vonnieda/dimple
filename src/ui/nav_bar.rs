use eframe::{egui::{Context, Ui, TextEdit, TextStyle}};


use super::theme::Theme;

#[derive(Default)]
pub struct NavBar {
    pub query: String,
}

pub enum NavEvent {
    Home,
    Back,
    Search(String),
}

impl NavBar {
    pub fn ui(&mut self, ctx: &Context, ui: &mut Ui) -> Option<NavEvent> {
        let theme = Theme::get(ctx);
        ui.vertical(|ui| {
            ui.horizontal(move |ui| {
                if Theme::icon_button(&theme.home_icon, 42, 42, ctx, ui).clicked() {
                    return Some(NavEvent::Home);
                }
                if Theme::icon_button(&theme.back_icon, 42, 42, ctx, ui).clicked() {
                    return Some(NavEvent::Back);
                }
                if TextEdit::singleline(&mut self.query)
                    .hint_text("What sounds good?")
                    .desired_width(f32::INFINITY)
                    .font(TextStyle::Heading)
                    .show(ui)
                    .response.changed() {
                    
                    Some(NavEvent::Search(self.query.clone()))
                }
                else {
                    None
                }
            }).inner
        }).inner
    }
}

