

use eframe::egui::{Ui};


use crate::{music_library::Release};

use super::{card_grid::{Card, LibraryItem}, theme::Theme};

impl Card for Release {
    fn ui(&self, image_width: f32, image_height: f32, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            // art
            if theme.carousel(&self.art, image_width as usize, ui).clicked() {
                action = Some(LibraryItem::Release(self.clone()));
            }
            // title
            if ui.link(Theme::big_n_bold(&self.title)).clicked() {
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
}
