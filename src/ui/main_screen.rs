use std::{sync::Arc, collections::HashMap};

use eframe::{egui::{self, Context, ImageButton, Ui}, epaint::{ColorImage, Color32}};
use egui_extras::RetainedImage;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};


use crate::{player::PlayerHandle, librarian::Librarian, music_library::{Library, Release}};

use super::{search_bar::SearchBar, player_bar::PlayerBar, card_grid::{CardGrid, Card}, utils::{dynamic_to_retained}};

pub struct MainScreen {
    search_bar: SearchBar,
    card_grid: CardGrid,
    player_bar: PlayerBar,
    librarian: Arc<Librarian>,
    cards: Vec<Box<dyn Card>>,
    images: HashMap<String, Arc<RetainedImage>>,
}

impl MainScreen {
    pub fn new(player: PlayerHandle, librarian: Arc<Librarian>) -> Self {
        Self {
            search_bar: SearchBar::default(),
            card_grid: CardGrid::default(),
            player_bar: PlayerBar::new(player),
            librarian,
            cards: Vec::new(),
            images: HashMap::new(),
        }
    }

    pub fn ui(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.add_space(8.0);
            if self.search_bar.ui(ctx, ui).changed() {
                let query = self.search_bar.query.clone();
                let cards = self.cards(&query);
                self.cards = cards;
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
        log::info!("cards: filter({})", query);
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
        log::info!("cards: sort()");
        releases.sort_by(|a, b| {
            a.artist().to_uppercase()
                .cmp(&b.artist().to_uppercase())
                .then(a.title.to_uppercase().cmp(&b.title.to_uppercase()))
        });

        // Convert to Cards
        log::info!("cards: convert()");
        let cards = releases.into_iter()
            .map(|release| {
                Box::new(self.card_from_release(&release)) as Box<dyn Card>
            })
            .collect();
        log::info!("cards: done");
        cards
    }

    fn card_from_release(&mut self, release: &Release) -> ReleaseCard {
        ReleaseCard {
            release: release.clone(),
            image: self.get_release_thumbnail(release),
        }
    }

    fn get_release_thumbnail(&mut self, release: &Release) -> Arc<RetainedImage> {
        // TODO could launch this into a thread to download later
        let key = format!("{}:{}x{}", release.url, 200.0, 200.0);
        
        if let Some(image) = self.images.get(&key) {
            return image.clone();
        }

        if let Some(image) = release.art.first() {
            if let Ok(dynamic) = self.librarian.image(image) {
                let retained = Arc::new(dynamic_to_retained("", &dynamic));
                self.images.insert(key, retained.clone());
                return retained;
            }
        }

        let color = ColorImage::new([200, 200], Color32::BLACK);
        let retained = Arc::new(RetainedImage::from_color_image("", color));
        self.images.insert(key, retained.clone());

        retained
    }
}

pub struct ReleaseCard {
    release: Release,
    image: Arc<RetainedImage>,
}

impl Card for ReleaseCard {
    fn ui(&self, image_width: f32, image_height: f32, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            let image_button =
                ImageButton::new(self.image.texture_id(ctx), 
                    egui::vec2(image_width, image_height)).frame(false);
            ui.add(image_button);
            ui.link(&self.release.title).clicked();
            ui.link(&self.release.artists.first().unwrap().name).clicked();
        });
    }   
}

