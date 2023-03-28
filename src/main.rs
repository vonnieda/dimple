use dimple::music_library::local::LocalMusicLibrary;
use dimple::music_library::navidrome::NavidromeLibrary;
use dimple::music_library::{Library, Release, Track};
use eframe::egui::{self, Context, Grid, ImageButton, Response, ScrollArea, TextEdit, Ui};
use eframe::epaint::{ColorImage, FontFamily, FontId};
use egui_extras::RetainedImage;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use image::DynamicImage;

use image::imageops::FilterType;
use rayon::prelude::*;

#[macro_use]
extern crate log;
extern crate env_logger;

// TODO BLOCKED make grid full width https://github.com/emilk/egui/discussions/1144#discussioncomment-2035457
// TODO make number of columns adapt to window width and tile size
// TODO tile size slider
// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs for smart playlist generation
// TODO how to load a custom font and use it globally https://github.com/catppuccin/egui/blob/main/examples/todo.rs#L77
// TODO info on how to do something on first frame: https://github.com/emilk/eframe_template/blob/master/src/app.rs#L24
// TODO escape should clear search
// TODO Continuous updates when downloading and loading libraries
// TODO search is sensitive to word order, i.e. infected shawarma vs. shawarma infected
// TODO parallelize the textures, although I think it might all happen on the
// first frame, in which case we could still do it somehow. Or just do whatever
// RetainedImage does.
// TODO Dashboard makes sense actually, and in fact, I can have as wild a tree
// as I want as long as:
// 1. Clicking anything updates the search bar with the terms needed to get to
//    where we are.
// 2. Hitting escape clears search.
// Dashbaord can contain "Artists", "Albums", "For You", "Today In", "Genres",
// etc. A bunch of derived stuff. And then scrolling down can include favorites
// and recents and such.

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.init();

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1440.0, 1024.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Dimple",
        native_options,
        Box::new(|_cc| Box::new(App::default())),
    )
    .expect("eframe: pardon me, but no thank you");
}
struct App {
    _library: Box<dyn Library>,
    cards: Vec<ReleaseCard>,
    query_string: String,
    playlist: Vec<Track>,
}

// TODO okay, I still think this becomes a Trait and then we have like ReleaseCard,
// and ArtistCard, and etc.
struct ReleaseCard {
    release: Release,
    image: RetainedImage,
}

impl Default for App {
    fn default() -> Self {
        info!("Loading config");
        let _config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build().expect("Config error");

        info!("Opening local library");
        let library = LocalMusicLibrary::new("data/library");
        // let library = NavidromeLibrary::from_config(&_config);

        info!("Reading releases");
        let releases = library.releases().unwrap();

        info!("Building cards");
        let mut cards = App::cards_from_releases(releases);

        info!("Sorting cards");
        cards.sort_by(|a, b| {
            a.subtitle()
                .to_uppercase()
                .cmp(&b.subtitle().to_uppercase())
        });

        info!("Done!");
        Self {
            _library: Box::new(library),
            cards: cards,
            query_string: "".to_string(),
            playlist: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
        self.browser(ctx);
    }
}

impl App {
    fn cards_from_releases(releases: Vec<Release>) -> Vec<ReleaseCard> {
        releases
            .par_iter()
            .map(|release| App::card_from_release(release))
            .collect()
    }

    fn card_from_release(release: &Release) -> ReleaseCard {
        let image = match release.art.first() {
            Some(image) => dynamic_to_retained(&image.url, &image.original.resize(200, 200, FilterType::CatmullRom)),
            None => RetainedImage::from_color_image("default", ColorImage::example()),
        };

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
        let _cards: Vec<&ReleaseCard> = self.cards.iter().filter(|card| {
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
                                            let tracks = card.release.tracks.clone();
                                            self.playlist.extend(tracks);
                                            println!("{:?}", self.playlist);
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
        // ui.vertical_centered_justified(|ui| {
        //     ui.horizontal(|ui| {
        //         let np = &self.now_playing;
        //         ui.add(ImageButton::new(
        //             np.image.texture_id(ctx),
        //             egui::vec2(120.0, 120.0),
        //         ));
        //         ui.vertical(|ui| {
        //             ui.link(&np.title);
        //             ui.link(&np.subtitle);
        //             self.plot_scrubber(ctx, ui);
        //             self.slider_scrubber(ctx, ui);
        //         });
        //         self.card(&self.up_next, 60.0, 60.0, ctx, ui);
        //     });
        // });
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
