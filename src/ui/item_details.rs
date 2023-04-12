use std::{sync::Arc};

use eframe::{egui::{Context, Ui}, epaint::{Color32, FontFamily, FontId}};

use crate::{music_library::{HasArtwork}};

use super::{card_grid::LibraryItem, utils, retained_images::RetainedImages};

pub struct ItemDetails {
    retained_images: Arc<RetainedImages>,
}

impl ItemDetails {
    pub fn new(retained_images: Arc<RetainedImages>) -> Self {
        Self {
            retained_images,
        }
    }

    pub fn ui(&mut self, item: LibraryItem, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal_top(|ui| {
                let (title, art) = match item {
                    LibraryItem::Release(release) => {
                        (release.title.clone(), release.art())
                    },
                    LibraryItem::Artist(artist) => {
                        (artist.name.clone(), artist.art())
                    },
                    LibraryItem::Track(track) => {
                        (track.title.clone(), track.art())
                    },
                    LibraryItem::Genre(genre) => {
                        (genre.name.clone(), genre.art())
                    },
                    LibraryItem::Playlist(playlist) => {
                        (playlist.name.clone(), playlist.art())
                    },
                };
                let texture_id = match art.first() {
                    Some(image) => self.retained_images.get(image, 300, 300).read().unwrap().texture_id(ctx),
                    None => utils::sample_image(Color32::LIGHT_GRAY, 300, 300).texture_id(ctx),
                };
                ui.image(texture_id, [300.0, 300.0]);
                // TODO icon (artist, release, genre, etc)
                // TODO maybe next to the art we show artist, album, title?
                ui.scope(|ui| {
                    ui.style_mut().override_font_id = Some(FontId::new(36.0, FontFamily::Proportional));
                    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                    ui.label(title);
                });
            });
        });
    }
}
