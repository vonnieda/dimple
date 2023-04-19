use std::sync::{Arc, RwLock};

use eframe::egui::{self, Ui, ScrollArea, Grid};

use crate::{music_library::{Artist, Release, Genre, Playlist, Track}, player::Player};

pub trait Card {
    fn ui(&self, image_width: f32, image_height: f32, ui: &mut Ui) -> Option<LibraryItem>;
}

#[derive(Clone)]
pub enum LibraryItem {
    Artist(Artist),
    Release(Release),
    Genre(Genre),
    Playlist(Playlist),
    Track(Track),
    Player(Arc<RwLock<Player>>),
}

#[derive(Default)]
pub struct CardGrid {
}

// TODO use ScrollArea::show_rows to improve performance. I
// tried previously and I couldn't get the rendering right.
// Oh, a hint, might also need Grid::show_rows
impl CardGrid {
    pub fn ui(&self, cards: &[Box<dyn Card>], image_width: f32, 
        image_height: f32, ui: &mut Ui) -> Option<LibraryItem> {

        // ui.spacing_mut().scroll_bar_width = 18.0;
        // ui.spacing_mut().scroll_bar_outer_margin = 0.0;
        // ui.spacing_mut().scroll_bar_inner_margin = 0.0;
        // ui.spacing_mut().scroll_handle_min_length = 24.0;
        ui.spacing_mut().button_padding[0] = 2.0;

        // This mess calculates the card spacing based on the available width
        // of the UI, ensuring that cards never get cut off and we reduce the
        // number of columns as the available width decreases.
        let scroll_bar_width = ui.spacing().scroll_bar_width 
            + ui.spacing().scroll_bar_inner_margin 
            + ui.spacing().scroll_bar_outer_margin;
        let max_width = ui.available_width() - scroll_bar_width;
        let min_card_spacing = 16.0;
        let card_frame_width = ui.spacing_mut().button_padding[0];
        let card_width = card_frame_width + image_width + card_frame_width + min_card_spacing;
        let num_columns: usize = (max_width / card_width).floor() as usize;
        let cards_width = card_width * num_columns as f32 - min_card_spacing;
        let addl_card_spacing = (max_width - cards_width) / num_columns as f32;
        let card_spacing = min_card_spacing + addl_card_spacing;
        let row_spacing = 16.0;
        let mut action = None;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    Grid::new(ui.next_auto_id())
                        .spacing(egui::vec2(card_spacing, row_spacing))
                        .show(ui, |ui| {
                            for (i, card) in cards.iter().enumerate() {
                                if let Some(a) = card.ui(image_width, image_height, ui) {
                                    action = Some(a);
                                }
                                if i % num_columns == num_columns - 1 {
                                    ui.end_row();
                                }
                            }
                    });
                });
            });
            action
        }

    pub fn ui2(&self, id: &str, cards: &[Box<dyn Card>], image_width: f32, 
        image_height: f32, ui: &mut Ui) -> Option<LibraryItem> {

        // ui.spacing_mut().scroll_bar_width = 18.0;
        // ui.spacing_mut().scroll_bar_outer_margin = 0.0;
        // ui.spacing_mut().scroll_bar_inner_margin = 0.0;
        // ui.spacing_mut().scroll_handle_min_length = 24.0;
        ui.spacing_mut().button_padding[0] = 2.0;

        // This mess calculates the card spacing based on the available width
        // of the UI, ensuring that cards never get cut off and we reduce the
        // number of columns as the available width decreases.
        let scroll_bar_width = ui.spacing().scroll_bar_width 
            + ui.spacing().scroll_bar_inner_margin 
            + ui.spacing().scroll_bar_outer_margin;
        let max_width = ui.available_width() - scroll_bar_width;
        let min_card_spacing = 16.0;
        let card_frame_width = ui.spacing_mut().button_padding[0];
        let card_width = card_frame_width + image_width + card_frame_width + min_card_spacing;
        let num_columns: usize = (max_width / card_width).floor() as usize;
        let cards_width = card_width * num_columns as f32 - min_card_spacing;
        let addl_card_spacing = (max_width - cards_width) / num_columns as f32;
        let card_spacing = min_card_spacing + addl_card_spacing;
        let row_spacing = 16.0;
        let mut action = None;

        ui.horizontal(|ui| {
            Grid::new(id)
                .spacing(egui::vec2(card_spacing, row_spacing))
                .show(ui, |ui| {
                    for (i, card) in cards.iter().enumerate() {
                        if let Some(a) = card.ui(image_width, image_height, ui) {
                            action = Some(a);
                        }
                        if i % num_columns == num_columns - 1 {
                            ui.end_row();
                        }
                    }
            });
        });
        action
    }
}

