use std::sync::{RwLock, Arc};

use eframe::egui::{ImageButton, Context, Ui, self};
use egui_extras::RetainedImage;

use crate::{music_library::Release};

use super::{card_grid::{Card, LibraryItem}, theme::Theme};

pub struct ReleaseCard {
    pub release: Release,
    pub image: Arc<RwLock<Arc<RetainedImage>>>,
}

impl ReleaseCard {
    // fn from_release(release: &Release) -> ReleaseCard {
    //     ReleaseCard {
    //         release: release.clone(),
    //         image: self.retained_images.get(release.art.first().unwrap(), 200, 200),
    //         player: self.player.clone(),
    //     }
    // }
}

impl Card for ReleaseCard {
    fn ui(&self, image_width: f32, image_height: f32, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        let mut action = None;
        ui.vertical(|ui| {
            let image_button =
                ImageButton::new(self.image.read().unwrap().texture_id(ctx), 
                    egui::vec2(image_width, image_height));
            // art
            if ui.add(image_button).clicked() {
                action = Some(LibraryItem::Release(self.release.clone()));
            }
            // title
            if ui.link(Theme::big_n_bold(&self.release.title)).clicked() {
                action = Some(LibraryItem::Release(self.release.clone()));
            }
            // Show each artist as a clickable link separated by commas
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = [0.0, 0.0].into();
                let len = self.release.artists.len();
                for (i, artist) in self.release.artists.iter().enumerate() {
                    if ui.link(&self.release.artist()).clicked() {
                        action = Some(LibraryItem::Artist(artist.clone()));
                    }
                    if i < len - 1 {
                        ui.label(",");
                    }
                }
            });
        });
        action
    }   
}

