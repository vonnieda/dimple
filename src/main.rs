use std::sync::{Arc, Mutex};

use eframe::egui::{self, Grid, ImageButton, Link, Response, ScrollArea, TextEdit, Ui, Context};
use eframe::epaint::{FontFamily, FontId, ColorImage};
use egui_extras::RetainedImage;
// use music_library::example::ExampleMusicLibrary;
use music_library::{MusicLibrary, Release, EmptyMusicLibrary};
use music_library::navidrome::NavidromeMusicLibrary;
mod music_library;
use config::{Config, File, FileFormat};

// TODO go through https://github.com/catppuccin/egui/blob/main/examples/todo.rs
// for ideas.
// TODO make grid full width
// TODO make number of columns adapt to window width and tile size
// TODO tile size slider
// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs
// TODO I think it makes sense for the libraries to be as simple and generic
// as possible, and this app just synchronizes their objects to the local store.
// The local store is, I think, the definitive list because we want the user to
// be able to update album art, lyrics, etc. even if the library doesn't support
// writing those properties.
// Shit, no. I'm stupid. I've been forgetting that I'll need to interest with
// the source again later to stream music, write back changes, refresh, etc.
// So, for tonight I think just get it working and then tomorrow I figure out how
// to have a merged view of library releases and to keep the ownership of those
// objects in the library. Or at least a reference.

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
        let builder = Config::builder()
            .add_source(File::new("config", FileFormat::Toml));
    
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

        let mut releases = Vec::new();
        for release in music_library.releases() {
            let mut cached_release = CachedRelease::default();
            if let Some(image) = &release.cover_image {
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color = egui::ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice());
                let retained = RetainedImage::from_color_image("asad", color);
                cached_release.cover_image = Some(retained);
            }
            cached_release.release = release;
            releases.push(cached_release);
        }
        return Self {
            query_string: String::from(""),
            releases: releases,
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
            egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                search_bar(&mut self.query_string, ui);
                ui.add_space(16.0);
                release_grid(&self.releases, ctx, ui);
            });
        });
    }
}

fn search_bar(query_string: &mut String, ui: &mut Ui) -> Response {
    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(query_string)
                .hint_text("What sounds good?")
                // TODO how do I use the theme font? Or the default, more specifically.
                .font(FontId::new(32.0, FontFamily::Proportional))
                .desired_width(f32::INFINITY),
        );
    })
    .response
}

fn release_grid(releases: &Vec<CachedRelease>, ctx: &Context, ui: &mut Ui) {
    let num_columns = 6;
    // TODO use show_rows to improve performance. I 
    // tried previously and I couldn't get the rendering right.
    ScrollArea::vertical().show(ui, |ui| {
        Grid::new("release_grid")
            .num_columns(num_columns)
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
}

fn release_card(release: &CachedRelease, ctx: &Context, ui: &mut Ui) -> Response {
    ui.vertical(|ui| {
        if let Some(cover_image) = &release.cover_image {
            ui.add(ImageButton::new(
                cover_image.texture_id(ctx), 
                egui::vec2(200.0, 200.0)));
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

