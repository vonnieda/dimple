use std::{sync::{Arc, RwLock}, collections::HashMap};

use eframe::{egui::{self, Context, ImageButton, Ui}, epaint::{ColorImage, Color32}};
use egui_extras::RetainedImage;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use threadpool::ThreadPool;

use crate::{player::PlayerHandle, librarian::Librarian, music_library::{Library, Release, Image}};

use super::{search_bar::SearchBar, player_bar::PlayerBar, card_grid::{CardGrid, Card}, utils};

pub struct MainScreen {
    search_bar: SearchBar,
    card_grid: CardGrid,
    player_bar: PlayerBar,
    librarian: Arc<Librarian>,
    cards: Vec<Box<dyn Card>>,
    retained_images: Arc<RwLock<HashMap<String, Arc<RetainedImage>>>>,
    image_loader_pool: threadpool::ThreadPool,
}

impl MainScreen {
    pub fn new(player: PlayerHandle, librarian: Arc<Librarian>) -> Self {
        let mut main_screen = Self {
            search_bar: SearchBar::default(),
            card_grid: CardGrid::default(),
            player_bar: PlayerBar::new(player),
            librarian,
            cards: Vec::new(),
            retained_images: Arc::new(RwLock::new(HashMap::new())),
            image_loader_pool: ThreadPool::default(),
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
            image: self.get_retained_image(release.art.first().unwrap(), 
                200, 200),
        }
    }

    /// Get a thumbnail for the given Image, returning a RetainedImage.
    /// Caches for performance. Unbounded for now.
    /// Requests the image from the Librarian if it's not in the cache.
    fn get_retained_image(&mut self, image: &Image, 
        width: usize, height: usize) -> Arc<RwLock<Arc<RetainedImage>>> {
        
        let key = format!("{}:{}x{}", image.url, width, height);
        
        if let Some(image) = self.retained_images.read().unwrap().get(&key) {
            return Arc::new(RwLock::new(image.clone()));
        }

        // TODO needs variable name cleanup and maybe turn some of this into
        // a function, or even a class
        let placeholder = ColorImage::new([width, height], Color32::BLACK);
        let retained_arc = Arc::new(RetainedImage::from_color_image("", placeholder));
        self.retained_images.write().unwrap().insert(key.clone(), retained_arc.clone());
        let retained = Arc::new(RwLock::new(retained_arc));

        let librarian_1 = self.librarian.clone();
        let image_1 = image.clone();
        let retained_images_1 = self.retained_images.clone();
        let retained_1 = retained.clone();
        self.image_loader_pool.execute(move || {
            if let Ok(dynamic) = librarian_1.image(&image_1) {
                let new_retained = Arc::new(utils::dynamic_to_retained("", &dynamic));
                retained_images_1.write().unwrap().insert(key, new_retained.clone());
                *retained_1.write().unwrap() = new_retained;
            }
        });

        retained
    }
}

pub struct ReleaseCard {
    release: Release,
    image: Arc<RwLock<Arc<RetainedImage>>>,
}

impl Card for ReleaseCard {
    fn ui(&self, image_width: f32, image_height: f32, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            let image_button =
                ImageButton::new(self.image.read().unwrap().texture_id(ctx), 
                    egui::vec2(image_width, image_height));
            ui.add(image_button);
            ui.link(&self.release.title).clicked();
            ui.link(&self.release.artists.first().unwrap().name).clicked();
        });
    }   
}

