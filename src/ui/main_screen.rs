use std::{sync::{Arc, RwLock}};

use eframe::{egui::{self, Context, ImageButton, Ui}};

use egui_extras::RetainedImage;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{player::PlayerHandle, librarian::Librarian, music_library::{Library, Release}};

use super::{search_bar::SearchBar, player_bar::PlayerBar, card_grid::{CardGrid, Card}, retained_images::RetainedImages};

pub struct MainScreen {
    librarian: Arc<Librarian>,
    player: PlayerHandle,
    retained_images: Arc<RetainedImages>,

    search_bar: SearchBar,
    card_grid: CardGrid,
    player_bar: PlayerBar,
    cards: Vec<Box<dyn Card>>,
}

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
        };
        main_screen.cards = main_screen.cards("");
        main_screen
    }

    pub fn ui(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.add_space(8.0);
            if self.search_bar.ui(ctx, ui).changed() {
                let query = self.search_bar.query.clone();
                self.cards = self.cards(&query);
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
            self.card_grid.ui(&self.cards, 200.0, 200.0, ctx, ui);
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
            image: self.retained_images.retained_image(release.art.first().unwrap(), 200, 200),
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
    fn ui(&self, image_width: f32, image_height: f32, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            let image_button =
                ImageButton::new(self.image.read().unwrap().texture_id(ctx), 
                    egui::vec2(image_width, image_height));
            // If the release image is clicked, go to the release. But for now queue the release.                    
            if ui.add(image_button).clicked() {
                self.player.write().unwrap().queue_release(&self.release);
            }
            // If the release title is clicked, go to the release.
            ui.link(&self.release.title).clicked();
            // if the artist name is clicked, go to the artist.
            ui.link(&self.release.artist()).clicked();
        });
    }   
}

