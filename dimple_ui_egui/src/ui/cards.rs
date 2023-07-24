use dimple_core::model::{Release, Artist, Genre};
use eframe::egui::Ui;

use super::{card_grid::{Card, LibraryItem}, theme::Theme};

impl Card for Release {
    fn ui(&self, image_width: f32, _image_height: f32, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            // art
            if theme.carousel(&self.art, image_width as usize, ui).clicked() {
                action = Some(LibraryItem::Release(self.clone()));
            }
            // title
            if ui.link(Theme::bold(&self.title)).clicked() {
                action = Some(LibraryItem::Release(self.clone()));
            }
            // Show each artist as a clickable link separated by commas
            ui.horizontal_wrapped(|ui| {
                // TODO move to common functions
                ui.spacing_mut().item_spacing = [0.0, 0.0].into();
                let len = self.artists.len();
                for (i, artist) in self.artists.iter().enumerate() {
                    if ui.link(&self.artist()).clicked() {
                        action = Some(LibraryItem::Artist(artist.clone()));
                    }
                    if i < len - 1 {
                        ui.label(", ");
                    }
                }
            });
        });
        action
    }       

    fn title(&self) -> String {
        self.title.clone()
    }
}

impl Card for Artist {
    fn ui(&self, image_width: f32, _image_height: f32, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            // art
            if theme.carousel(&self.art, image_width as usize, ui).clicked() {
                action = Some(LibraryItem::Artist(self.clone()));
            }
            // name
            if ui.link(Theme::bold(&self.name)).clicked() {
                action = Some(LibraryItem::Artist(self.clone()));
            }
        });
        action
    }       

    fn title(&self) -> String {
        self.name.clone()
    }
}

impl Card for Genre {
    fn ui(&self, image_width: f32, _image_height: f32, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            // art
            // TODO figure out how to generate cool art for genres next.
            if theme.carousel(&self.art, image_width as usize, ui).clicked() {
                action = Some(LibraryItem::Genre(self.clone()));
            }
            // name
            if ui.link(Theme::bold(&self.name)).clicked() {
                action = Some(LibraryItem::Genre(self.clone()));
            }
        });
        action
    }       


    fn title(&self) -> String {
        self.name.clone()
    }
}

impl From<Release> for Box<dyn Card> {
    fn from(value: Release) -> Self {
        Box::new(value)
    }
}

impl From<Artist> for Box<dyn Card> {
    fn from(value: Artist) -> Self {
        Box::new(value)
    }
}

impl From<Genre> for Box<dyn Card> {
    fn from(value: Genre) -> Self {
        Box::new(value)
    }
}

