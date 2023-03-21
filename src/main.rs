use eframe::egui::{self, Grid, ImageButton, Link, Response, ScrollArea, TextEdit, Ui, Context, ProgressBar, Slider};
use eframe::epaint::{FontFamily, FontId, Rect};
use egui_extras::RetainedImage;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use image::DynamicImage;
use music_library::{MusicLibrary, Release, EmptyMusicLibrary};
use music_library::navidrome::NavidromeMusicLibrary;
mod music_library;
use config::{Config, File, FileFormat};

// TODO go through https://github.com/catppuccin/egui/blob/main/examples/todo.rs
// for ideas.
// TODO make grid full width
// TODO make number of columns adapt to window width and tile size
// TODO tile size slider
// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs for smart playlist generation
// TODO how to load a custom font and use it globally https://github.com/catppuccin/egui/blob/main/examples/todo.rs#L77
// TODO info on how to do something on first frame: https://github.com/emilk/eframe_template/blob/master/src/app.rs#L24
// TODO escape should clear search

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1440.0, 1024.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Music",
        native_options,
        Box::new(|_cc| Box::new(App::default())),
    )
    .expect("eframe: pardon me, but no thank you");
}

#[derive(Default)]
struct CachedRelease {
    release: Release,
    cover_image: Option<RetainedImage>,
}

struct App {
    query_string: String,
    releases: Vec<CachedRelease>,
}

impl Default for App {
    fn default() -> Self {
        // load config
        let builder = Config::builder()
            .add_source(File::new("config", FileFormat::Toml));
    
        // load a music library
        let music_library:Box<dyn MusicLibrary> = match builder.build() {
            Ok(config) => {
                Box::new(NavidromeMusicLibrary::new(
                    config.get_string("navidrome.site").unwrap().as_str(),
                    config.get_string("navidrome.username").unwrap().as_str(),
                    config.get_string("navidrome.password").unwrap().as_str()))
            },
            Err(_) => {
                Box::new(EmptyMusicLibrary::default())
            }
        };

        // collect the releases from the music library
        // TODO parallelize the textures, although I think it might all
        // happen on the first frame, in which case we could still do it
        // somehow. Or just do whatever RetainedImage does.
        let mut releases = Vec::new();
        for release in music_library.releases() {
            let mut cached_release = CachedRelease::default();
            if let Some(image) = &release.cover_image {
                cached_release.cover_image = Some(dynamic_to_retained("debug_name", &image));
            }
            cached_release.release = release;
            releases.push(cached_release);
        }

        // sort the releases
        releases.sort_by(|a, b| {
            a.release.title.cmp(&b.release.title)
        });

        // off we go
        return Self {
            query_string: String::from(""),
            releases: releases,
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
        
        ctx.set_debug_on_hover(true);

        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.add_space(12.0);
            search_bar(&mut self.query_string, ui);
            ui.add_space(4.0);
        });

        egui::TopBottomPanel::bottom("player").show(ctx, |ui| {
            ui.add_space(4.0);
            player_bar(&self.releases[1], ctx, ui);
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let matcher = SkimMatcherV2::default();
            let releases: Vec<&CachedRelease> = self.releases
                .iter()
                .filter(|release| {
                    // TODO still sensitive to word order, i.e.:
                    // infected shawarma vs. shawarma infected
                    let mut haystack = release.release.title.clone();
                    if let Some(artist) = &release.release.artist {
                        haystack = haystack + " " + artist;
                    }
                    return matcher.fuzzy_match(
                        haystack.as_str(), 
                        &self.query_string
                    ).is_some();
                })
                .collect();
            release_grid(&releases, ctx, ui);
        });
    }
}

fn search_bar(query_string: &mut String, ui: &mut Ui) -> Response {
    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(query_string)
                .hint_text("What sounds good?")
                // TODO how do I use the theme font? Or the default, more specifically.
                .font(FontId::new(28.0, FontFamily::Proportional))
                .desired_width(f32::INFINITY),
        );
    })
    .response
}

fn player_bar(release: &CachedRelease, ctx: &Context, ui: &mut Ui) -> Response {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            if let Some(cover_image) = &release.cover_image {
                let button = ImageButton::new(
                    cover_image.texture_id(ctx), 
                    egui::vec2(120.0, 120.0));
                ui.add(button);
            }
            ui.vertical(|ui| {
                ui.link(&release.release.title);
                if let Some(artist) = &release.release.artist {
                    ui.link(artist);
                }
                // visual_scrubber(release, ctx, ui);
                slider_scrubber(release, ctx, ui);
            });
            up_next(release, ctx, ui);
            ui.add_space(4.0);
        })    
    })
    .response
}

// fn visual_scrubber(release: &CachedRelease, ctx: &Context, ui: &mut Ui) -> Response {
// }

fn slider_scrubber(release: &CachedRelease, ctx: &Context, ui: &mut Ui) -> Response {
    // TODO magic
    ui.spacing_mut().slider_width = 1000.0;
    let mut my_f32:f32 = 0.33;
    ui.add(egui::Slider::new(&mut my_f32, 0.0..=1.0)
        .show_value(false)
        .trailing_fill(true)
    )
}

fn release_grid(releases: &Vec<&CachedRelease>, ctx: &Context, ui: &mut Ui) {
    let num_columns = 6;
    // TODO use ScrollArea::show_rows to improve performance. I 
    // tried previously and I couldn't get the rendering right.
    // Oh, a hint, might also need Grid::show_rows

    ui.vertical_centered_justified(|ui| {
        ScrollArea::vertical()
        // .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                Grid::new("release_grid")
                // .num_columns(num_columns)
                .spacing(egui::vec2(16.0, 16.0))
                .show(ui, |ui| {
                    for (i, release) in releases.iter().enumerate() {
                        release_card(&release, ctx, ui);
                        if i % num_columns == num_columns - 1 {
                            ui.end_row();
                        }
                    }
                });
            });
        });
    });
}

// TODO this is a copy of release_card, DRY it out
fn up_next(release: &CachedRelease, ctx: &Context, ui: &mut Ui) -> Response {
    ui.vertical(|ui| {
        ui.label("Up Next");
        if let Some(cover_image) = &release.cover_image {
            let button = ImageButton::new(
                cover_image.texture_id(ctx), 
                egui::vec2(60.0, 60.0));
            ui.add(button);
        }
        // TODO default image
        ui.add_space(4.0);
        ui.add(Link::new(&release.release.title));
        if let Some(artist) = &release.release.artist {
            // ui.add_space(2.0);
            ui.add(Link::new(artist));
            }
    })
    .response
}

fn release_card(release: &CachedRelease, ctx: &Context, ui: &mut Ui) -> Response {
    ui.vertical(|ui| {
        if let Some(cover_image) = &release.cover_image {
            let button = ImageButton::new(
                cover_image.texture_id(ctx), 
                egui::vec2(200.0, 200.0));
            ui.add(button);
        }
        // TODO default image
        ui.add_space(8.0);
        ui.add(Link::new(&release.release.title));
        if let Some(artist) = &release.release.artist {
            ui.add_space(2.0);
            ui.add(Link::new(artist));
            }
    })
    .response
}

fn dynamic_to_retained(debug_name: &str, image: &DynamicImage) -> RetainedImage {
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color = egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice());
    let retained = RetainedImage::from_color_image(debug_name, color);
    retained
}

