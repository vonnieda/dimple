use std::{sync::{Arc, RwLock}};

use eframe::{egui::{self, Context, ImageButton, Ui, Link}, epaint::{Color32, FontId}};

use egui_extras::RetainedImage;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{player::PlayerHandle, librarian::Librarian, music_library::{Library, Release, Artist, Genre, Playlist}};

use super::{search_bar::SearchBar, player_bar::PlayerBar, card_grid::{CardGrid, Card, LibraryItem}, retained_images::RetainedImages, item_details::ItemDetails};

pub struct MainScreen {
    librarian: Arc<Librarian>,
    player: PlayerHandle,
    retained_images: Arc<RetainedImages>,

    search_bar: SearchBar,
    card_grid: CardGrid,
    player_bar: PlayerBar,
    cards: Vec<Box<dyn Card>>,

    selected_item: Option<LibraryItem>,
    item_details: ItemDetails,
}

// Artist Cards
// Genre Cards
// Playlist Cards
// TODO Release Details
// TODO Artist Details
// TODO Genre Details

impl MainScreen {
    pub fn new(player: PlayerHandle, librarian: Arc<Librarian>) -> Self {
        let retained_images = Arc::new(RetainedImages::new(librarian.clone()));
        let mut main_screen = Self {
            librarian: librarian.clone(),
            player: player.clone(),
            retained_images: retained_images.clone(),
            search_bar: SearchBar::default(),
            card_grid: CardGrid::default(),
            player_bar: PlayerBar::new(player.clone(), retained_images.clone()),
            cards: Vec::new(),
            selected_item: None,
            item_details: ItemDetails::new(retained_images.clone()),
        };
        main_screen.cards = main_screen.cards("");
        main_screen
    }

    pub fn ui(&mut self, ctx: &Context) {
        // egui::Window::new("Style").show(ctx, |ui| {
        //     ctx.style_ui(ui);
        // });
        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.add_space(8.0);
            if self.search_bar.ui(ctx, ui).changed() {
                let query = self.search_bar.query.clone();
                self.cards = self.cards(&query);
                self.selected_item = None;
            }
            ui.add_space(8.0);
        });
        
        egui::TopBottomPanel::bottom("player").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.player_bar.ui(ctx, ui);
                ui.add_space(8.0);
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(item) = &self.selected_item {
                self.item_details.ui(item.clone(), ctx, ui);
            }
            else {
                let action = self.card_grid.ui(&self.cards, 200.0, 200.0, ctx, ui);
                if action.is_some() {
                    self.selected_item = action;
                }
            }
        });
    }

    /// Get the list of Cards to show in the grid. Performs filtering, sorting,
    /// and caching.
    fn cards(&mut self, query: &str) -> Vec<Box<dyn Card>> {
        // Filter Releases by the query
        let matcher = SkimMatcherV2::default();
        let mut releases: Vec<Release> = self.librarian.releases().into_iter()
            .filter(|release| {
                let haystack = format!("{} {}", release.title, release.artist())
                    .to_uppercase();
                return matcher
                    .fuzzy_match(haystack.as_str(), query)
                    .is_some();
            })
            .collect();

        // Sort Releases by Artist Name then Release Title
        releases.sort_by(|a, b| {
            a.artist().to_uppercase()
                .cmp(&b.artist().to_uppercase())
                .then(a.title.to_uppercase().cmp(&b.title.to_uppercase()))
        });

        // Convert to Cards
        releases.into_iter()
            .map(|release| {
                Box::new(self.card_from_release(&release)) as Box<dyn Card>
            })
            .collect()
    }

    fn card_from_release(&mut self, release: &Release) -> ReleaseCard {
        ReleaseCard {
            release: release.clone(),
            image: self.retained_images.get(release.art.first().unwrap(), 200, 200),
            player: self.player.clone(),
        }
    }
}

pub struct ReleaseCard {
    release: Release,
    image: Arc<RwLock<Arc<RetainedImage>>>,
    player: PlayerHandle,
}

impl Card for ReleaseCard {
    fn ui(&self, image_width: f32, image_height: f32, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        let mut action = None;
        ui.vertical(|ui| {
            let image_button =
                ImageButton::new(self.image.read().unwrap().texture_id(ctx), 
                    egui::vec2(image_width, image_height));
            if ui.add(image_button).clicked() {
                action = Some(LibraryItem::Release(self.release.clone()));
            }
            ui.scope(|ui| {
                ui.visuals_mut().override_text_color = Some(Color32::from_gray(0xE7));
                ui.visuals_mut().hyperlink_color = Color32::from_gray(0xE7);
                ui.style_mut().override_font_id = Some(FontId::proportional(15.0));
                if ui.link(&self.release.title).clicked() {
                    action = Some(LibraryItem::Release(self.release.clone()));
                }
            });
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

