use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::thread;

use eframe::egui::{self, Context, Grid, ImageButton, Response, ScrollArea, TextEdit, Ui};
use eframe::epaint::{ColorImage, FontFamily, FontId};
use egui_extras::RetainedImage;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use image::DynamicImage;

use rodio::{Sink};

use crate::music_library::Release;
use crate::music_library::libraries::Libraries;
use crate::music_library::local::LocalLibrary;
use crate::music_library::navidrome::NavidromeLibrary;
use crate::{music_library::Library, player::Player};

use rayon::prelude::*;

// TODO test out the Send Sync idea on Library

/// Okay, so startup:
/// - Window appears empty but immediately starts filling with cards.
/// - Cards may initially not have a visible image, but the image will
///   appear as quickly as possible.
/// - So I think, for now, launch a thread on startup that reads each
///   library and then launch another thread that uses to a channel to
///   pass the Releases back.
/// - Need to decide on a struct for storing the releases, and for the
///   cards.
/// - And get show_rows working so everything doesn't have to be loaded
///   on the first frame.
/// 

pub struct Dimple {
    libraries: Arc<Libraries>,
    cards: Arc<RwLock<Vec<ReleaseCard>>>,
    query_string: String,
    player: Player,
    retained_image_cache: HashMap<String, RetainedImage>,
}

impl eframe::App for Dimple {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
        self.browser(ctx);
    }
}

impl Dimple {
    fn load_cards(libraries: &Libraries, cards: &RwLock<Vec<ReleaseCard>>) {
        log::info!("Reading releases");
        let releases = libraries.releases().unwrap();

        log::info!("Building cards");
        let mut new_cards = Self::cards_from_releases(&libraries, releases);

        log::info!("Merging cards");
        cards.write().unwrap().extend(new_cards);

        log::info!("Sorting cards");
        cards.write().unwrap().sort_by(|a, b| {
            a.subtitle()
                .to_uppercase()
                .cmp(&b.subtitle().to_uppercase())
        });

        log::info!("Done");
    }

    pub fn new(sink: Arc<Sink>) -> Self {
        log::info!("Loading config");
        let config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build().expect("Config error");

        log::info!("Loading libraries");
        let mut libraries = Libraries::default();
        libraries.add_library(Box::new(LocalLibrary::new("data/library")) as Box<dyn Library + Send + Sync>);
        // libraries.add_library(Box::new(NavidromeLibrary::from_config(&config)) as Box<dyn Library + Send + Sync>);
        let libraries = Arc::new(libraries);

        let cards = Arc::new(RwLock::new(vec![])); 

        // Launch a thread that refreshes libraries and updates cards.
        // let libraries_tmp = libraries.clone();
        let libraries_1 = libraries.clone();
        let cards_1 = cards.clone();
        thread::spawn(move || {
            Dimple::load_cards(libraries_1.as_ref(), cards_1.as_ref());
        });

        log::info!("Starting up");
        Self {
            libraries: libraries.clone(),
            cards: cards.clone(),
            query_string: "".to_string(),
            player: Player::new(sink, libraries.clone()),
            retained_image_cache: HashMap::new(),
        }
    }

    fn cards_from_releases(library: &Libraries, releases: Vec<Release>) -> Vec<ReleaseCard> {
        releases
            .par_iter()
            .map(|release| Self::card_from_release(library, release))
            .collect()
    }

    fn card_from_release(library: &Libraries, release: &Release) -> ReleaseCard {
        let image = release.art.first()
            .map_or(None, |image| match library.image(image) {
                Ok(image) => Some(image),
                Err(_) => None,
            })
            .map_or(
                RetainedImage::from_color_image("default", ColorImage::example()), 
                |image| dynamic_to_retained("", &image));

        ReleaseCard { 
            release: release.clone(), 
            image 
        }
    }

    // it's not really the browser, it's more like the main screen.
    fn browser(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.add_space(8.0);
            self.search_bar(ui);
            ui.add_space(8.0);
        });

        egui::TopBottomPanel::bottom("player").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.player_bar(ctx, ui);
                ui.add_space(8.0);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.card_grid(ctx, ui);
        });
    }

    fn search_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut self.query_string)
                    .hint_text("What sounds good?")
                    // TODO how do I use the theme font? Or the default, more specifically.
                    .font(FontId::new(28.0, FontFamily::Proportional))
                    .desired_width(f32::INFINITY),
            );
        });
    }

    fn card_grid(&mut self, ctx: &Context, ui: &mut Ui) {
        let matcher = SkimMatcherV2::default();
        // TODO just do this when search changes, not every frame
        let binding = self.cards.read().unwrap();
        let _cards: Vec<&ReleaseCard> = binding.iter().filter(|card| {
            let haystack = format!("{} {}", card.title(), card.subtitle());
            return matcher
                .fuzzy_match(haystack.as_str(), &self.query_string)
                .is_some();
        })
        .collect();

        let num_columns = 6;

        // TODO use ScrollArea::show_rows to improve performance. I
        // tried previously and I couldn't get the rendering right.
        // Oh, a hint, might also need Grid::show_rows
        ui.vertical_centered_justified(|ui| {
            ui.spacing_mut().scroll_bar_width = 16.0;
            ui.spacing_mut().scroll_handle_min_length = 22.0;
            // ui.spacing_mut().scroll_bar_outer_margin = 10.0;
            // ui.spacing_mut().scroll_bar_inner_margin = 10.0;
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.horizontal(|ui| {
                            // TODO magic yeesh.
                            ui.add_space(40.0);
                            Grid::new("card_grid")
                                .spacing(egui::vec2(16.0, 16.0))
                                .show(ui, |ui| {
                                    for (i, card) in _cards.iter().enumerate() {
                                        if Self::card(card, 200.0, 200.0, ctx, ui).clicked() {
                                            self.player.add_release(&card.release);
                                        }
                                        if i % num_columns == num_columns - 1 {
                                            ui.end_row();
                                        }
                                    }
                                });
                        })
                    });
                });
        });
    }

    fn card(
        card: &ReleaseCard,
        width: f32,
        height: f32,
        ctx: &Context,
        ui: &mut Ui,
    ) -> Response {
        ui.vertical(|ui| {
            let image_button =
                ImageButton::new(card.image().texture_id(ctx), egui::vec2(width, height));
            let response = ui.add(image_button);
            ui.link(card.title());
            ui.link(card.subtitle());
            return response;
        })
        .inner
    }

    fn player_bar(self: &mut Self, ctx: &Context, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.horizontal(|ui| {
                let track = self.player.current_track();
                let image = RetainedImage::from_color_image("default", ColorImage::example());
                let title = track.map_or("".to_string(), |track| track.title.to_string());
                // TODO till we know what album it's from
                let subtitle = title.clone();

                ui.add(ImageButton::new(
                    image.texture_id(ctx),
                    egui::vec2(120.0, 120.0),
                ));
                ui.vertical(|ui| {
                    ui.link(&title);
                    ui.link(&subtitle);
                    self.plot_scrubber(ctx, ui);
                    self.slider_scrubber(ctx, ui);
                    if ui.button("Play").clicked() {
                        self.player.play();
                    }
                    if ui.button("Pause").clicked() {
                        self.player.pause();
                    }
                    if ui.button("Next").clicked() {
                        self.player.next();
                    }
                    
                });
                // self.card(&self.up_next, 60.0, 60.0, ctx, ui);
            });
        });
    }

    fn plot_scrubber(self: &Self, ctx: &Context, ui: &mut Ui) {
        // let sin: PlotPoints = (0..1000).map(|i| {
        //     let x = i as f64 * 0.01;
        //     [x, x.sin()]
        // }).collect();
        // let line = Line::new(sin);
        // Plot::new("my_plot")
        //     .view_aspect(1.0)
        //     .show(ui, |plot_ui| plot_ui.line(line));
    }

    fn slider_scrubber(self: &Self, ctx: &Context, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            let mut my_f32: f32 = 0.33;
            ui.add(
                egui::Slider::new(&mut my_f32, 0.0..=1.0)
                    .show_value(false)
                    .trailing_fill(true),
            );
        });
    }
}

// TODO take a look at how RetainedImage does it's loading and see if I can
// optimize or remove this.
fn dynamic_to_retained(debug_name: &str, image: &DynamicImage) -> RetainedImage {
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    let retained = RetainedImage::from_color_image(debug_name, color);
    retained
}

impl ReleaseCard {
    fn image(&self) -> &RetainedImage {
        return &self.image;
    }

    fn title(&self) -> &str {
        return &self.release.title;
    }

    fn subtitle(&self) -> &str {
        self.release.artists.first().map_or("Unknown", |artist| artist.name.as_str())
    }
}

// TODO okay, I still think this becomes a Trait and then we have like ReleaseCard,
// and ArtistCard, and etc.
struct ReleaseCard {
    release: Release,
    image: RetainedImage,
}

