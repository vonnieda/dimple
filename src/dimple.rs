use std::collections::HashMap;
use std::sync::{Arc, RwLock};
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

pub struct Dimple {
    libraries: Arc<Libraries>,
    cards: Arc<RwLock<Vec<ReleaseCard>>>,
    query_string: String,
    player: Player,
    retained_image_cache: HashMap<String, RetainedImage>,
    first_frame: bool,
}

impl eframe::App for Dimple {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO gross hack, see:
        // TODO info on how to do something on first frame: https://github.com/emilk/eframe_template/blob/master/src/app.rs#L24
        if !self.first_frame {
            self.first_frame = true;

            catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);

            // Launch a thread that refreshes libraries and updates cards.
            // TODO temporary, just needs a place to live for a moment            
            let libraries_1 = self.libraries.clone();
            let cards_1 = self.cards.clone();
            let context_1 = ctx.clone();
            thread::spawn(move || {
                // TODO currently just runs once, eventually will handle merging
                // cards and will refresh.
                for release in libraries_1.releases().iter() {
                    let card = Self::card_from_release(&libraries_1, &release);
                    cards_1.write().unwrap().push(card);
                    context_1.request_repaint();
                }
            });

        }
        self.browser(ctx);
    }
}

impl Dimple {
    pub fn new(sink: Arc<Sink>) -> Self {
        // Load config
        let config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build().expect("Config error");

        // Load libraries
        // TODO once I understand the config system a bit better these will be passed
        // on to the implementations. Ultimately, probably just use serde.
        let mut libraries = Libraries::new();

        for library in config.get_array("library").unwrap().into_iter() {
            let values = library.into_table().unwrap();
            match values.get("type").unwrap().clone().into_string().unwrap().as_str() {
                "local" => {
                    let ulid = values.get("type").unwrap().clone().into_string().unwrap();
                    let name = values.get("name").unwrap().clone().into_string().unwrap();
                    libraries.add_library(Box::new(LocalLibrary::new(&ulid, &name)) as Box<dyn Library>);
                },
                "navidrome" => {
                    let ulid = values.get("ulid").unwrap().clone().into_string().unwrap();
                    let name = values.get("name").unwrap().clone().into_string().unwrap();
                    let site = values.get("site").unwrap().clone().into_string().unwrap();
                    let username = values.get("username").unwrap().clone().into_string().unwrap();
                    let password = values.get("password").unwrap().clone().into_string().unwrap();
                    libraries.add_library(Box::new(NavidromeLibrary::new(&ulid, &name, &site, &username, &password)) as Box<dyn Library>);
                },
                &_ => todo!()
            }
        }

        let libraries = Arc::new(libraries);

        Self {
            libraries: libraries.clone(),
            cards: Arc::new(RwLock::new(Vec::new())),
            query_string: String::new(),
            player: Player::new(sink, libraries.clone()),
            retained_image_cache: HashMap::new(),
            first_frame: false,
        }
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
            image,
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
        let mut _cards: Vec<&ReleaseCard> = binding.iter().filter(|card| {
            let haystack = format!("{} {}", card.title(), card.subtitle());
            return matcher
                .fuzzy_match(haystack.as_str(), &self.query_string)
                .is_some();
        })
        .collect();

        _cards.sort_by(|a, b| {
            a.subtitle()
                .to_uppercase()
                .cmp(&b.subtitle().to_uppercase())
        });

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

