use eframe::{egui::{Context, Ui, TextEdit, Response, FontDefinitions, TextStyle, ImageButton}, epaint::{FontId, FontFamily, Color32}};
use egui_extras::RetainedImage;

use crate::dimple::Theme;

pub struct NavBar {
    pub query: String,

    home_icon: RetainedImage,
    back_icon: RetainedImage,    
}

pub enum NavEvent {
    Home,
    Back,
    Search(String),
}

impl Default for NavBar {
    fn default() -> Self {
        Self {
            home_icon: Theme::svg_icon(include_bytes!("../icons/material/home_FILL0_wght400_GRAD0_opsz48.svg")),
            back_icon: Theme::svg_icon(include_bytes!("../icons/material/arrow_back_FILL0_wght400_GRAD0_opsz48.svg")),
            query: String::default(),
        }
    }
}

impl NavBar {
    pub fn ui(&mut self, ctx: &Context, ui: &mut Ui) -> Option<NavEvent> {
        ui.vertical(|ui| {
            ui.horizontal(move |ui| {
                if Theme::icon_button(&self.home_icon, 42, 42, ctx, ui).clicked() {
                    return Some(NavEvent::Home);
                }
                if Theme::icon_button(&self.back_icon, 42, 42, ctx, ui).clicked() {
                    return Some(NavEvent::Back);
                }
                if TextEdit::singleline(&mut self.query)
                    .hint_text("What sounds good?")
                    .desired_width(f32::INFINITY)
                    .font(TextStyle::Heading)
                    .show(ui)
                    .response.changed() {
                    
                    return Some(NavEvent::Search(self.query.clone()));
                }
                else {
                    return None;
                }
            }).inner
        }).inner
    }
}

