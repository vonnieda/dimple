use eframe::egui::{self, Grid, ImageButton, Link, Response, ScrollArea, TextEdit, Ui};
use eframe::epaint::{FontFamily, FontId};
use music_library::example::ExampleMusicLibrary;
use music_library::{MusicLibrary, Release};
use music_library::navidrome::NavidromeMusicLibrary;
mod music_library;
use config::{Config, File, FileFormat};

// TODO go through https://github.com/catppuccin/egui/blob/main/examples/todo.rs
// for ideas.
// TODO make grid full width
// TODO make number of columns adapt to window width and tile size
// TODO tile size slider
// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs

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

struct App {
    query_string: String,
    music_library: Box<dyn MusicLibrary>,
}

impl Default for App {
    fn default() -> Self {
        let mut builder = Config::builder()
            .add_source(File::new("config", FileFormat::Toml));
    
        if let Ok(config) = builder.build() {
            // TODO this is all trash
            return Self {
                query_string: String::from(""),
                music_library: Box::new(NavidromeMusicLibrary::new(
                    config.get_string("navidrome.site").unwrap().as_str(),
                    config.get_string("navidrome.username").unwrap().as_str(),
                    config.get_string("navidrome.password").unwrap().as_str())),
            }
        }
        else {
            return Self {
                query_string: String::from(""),
                music_library: Box::new(ExampleMusicLibrary::new()),
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
            egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                search_bar(&mut self.query_string, ui);
                ui.add_space(16.0);
                release_grid(self.music_library.releases(), ctx, ui);
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

fn release_grid(releases: &[Release], ctx: &egui::Context, ui: &mut Ui) {
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

fn release_card(release: &Release, ctx: &egui::Context, ui: &mut Ui) -> Response {
    ui.vertical(|ui| {
        if let Some(cover_image) = &release.cover_image {
            ui.add(ImageButton::new(cover_image.texture_id(ctx), egui::vec2(200.0, 200.0)));
        }
        // else {
            // TODO default image
        //     ui.add()
        // }
        ui.add_space(8.0);
        ui.add(Link::new(&release.title));
        ui.add_space(2.0);
        ui.add(Link::new(&release.artist));
    })
    .response
}

