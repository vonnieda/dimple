use eframe::egui::{self, Grid, ImageButton, Link, Response, ScrollArea, TextEdit, Ui, Context};
use eframe::epaint::{FontFamily, FontId};
use egui_extras::RetainedImage;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use image::DynamicImage;
use music_library::local::LocalMusicLibrary;
use music_library::{MusicLibrary, Release, EmptyMusicLibrary};
use music_library::navidrome::NavidromeMusicLibrary;
mod music_library;
use config::{Config, File, FileFormat};

// TODO go through https://github.com/catppuccin/egui/blob/main/examples/todo.rs for ideas.
// TODO make grid full width
// TODO make number of columns adapt to window width and tile size
// TODO tile size slider
// TODO check out bliss and bliss-rs https://github.com/Polochon-street/bliss-rs for smart playlist generation
// TODO how to load a custom font and use it globally https://github.com/catppuccin/egui/blob/main/examples/todo.rs#L77
// TODO info on how to do something on first frame: https://github.com/emilk/eframe_template/blob/master/src/app.rs#L24
// TODO escape should clear search
// TODO Continuous updates when downloading and loading libraries



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

// trait Card {
//     id: String,

// }

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

        // load the local music library
        println!("Loading local library");
        let local_library = LocalMusicLibrary::new("data/library");
        let local_releases = local_library.releases();
        println!("Local library contains {} releases", local_releases.len());

        // // load a remote music library
        // let remote_library:Box<dyn MusicLibrary> = match builder.build() {
        //     Ok(config) => {
        //         Box::new(NavidromeMusicLibrary::new(
        //             config.get_string("navidrome.site").unwrap().as_str(),
        //             config.get_string("navidrome.username").unwrap().as_str(),
        //             config.get_string("navidrome.password").unwrap().as_str()))
        //     },
        //     Err(_) => {
        //         Box::new(EmptyMusicLibrary::default())
        //     }
        // };
        // let releases = remote_library.releases();
        // println!("Remote library contains {} releases", releases.len());

        // // merge all the remote releases into the local
        // for (i, release) in releases.iter().enumerate() {
        //     println!("Merging {}/{}: {}", i + 1, releases.len(), release.title);
        //     local_library.merge_release(&release).expect("merge error");
        // }        

        // collect the releases from the music library
        // TODO parallelize the textures, although I think it might all
        // happen on the first frame, in which case we could still do it
        // somehow. Or just do whatever RetainedImage does.
        println!("Loading releases");
        let mut releases = Vec::new();
        for release in local_releases {
            let mut cached_release = CachedRelease::default();
            if let Some(image) = &release.cover_image {
                cached_release.cover_image = Some(dynamic_to_retained("debug_name", &image));
            }
            cached_release.release = release;
            releases.push(cached_release);
        }

        // sort the releases
        println!("Sorting releases");
        releases.sort_by(|a, b| {
            if let Some(aa) = &a.release.artist {
                if let Some(bb) = &b.release.artist {
                    return aa.to_uppercase().cmp(&bb.to_uppercase());
                }    
            }
            a.release.title.to_uppercase().cmp(&b.release.title.to_uppercase())
        });

        // off we go
        println!("Done");
        return Self {
            query_string: String::from(""),
            releases,
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
        
        // ctx.set_debug_on_hover(true);

        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            search_bar(&mut self.query_string, ui);
        });

        egui::TopBottomPanel::bottom("player").show(ctx, |ui| {
            player_bar(&self.releases[259], ctx, ui);
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
            release_grid(&releases, ctx, ui)
        });

        // egui::Window::new("🔧 Settings")
        //     // .open(settings)
        //     .vscroll(true)
        //     .show(ctx, |ui| {
        //         ctx.settings_ui(ui);
        //     });

        // egui::Window::new("🔍 Inspection")
        //     // .open(inspection)
        //     .vscroll(true)
        //     .show(ctx, |ui| {
        //         ctx.inspection_ui(ui);
        //     });

        // egui::Window::new("📝 Memory")
        //     // .open(memory)
        //     .resizable(false)
        //     .show(ctx, |ui| {
        //         ctx.memory_ui(ui);
        //     });
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
        ui.horizontal(|ui| {
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
        ui.spacing_mut().scroll_bar_width = 24.0;
        ui.spacing_mut().scroll_handle_min_length = 24.0;
        ui.spacing_mut().scroll_bar_outer_margin = 16.0;
        ui.spacing_mut().scroll_bar_inner_margin = 16.0;

        ScrollArea::vertical()
            .show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Grid::new("release_grid")
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
        ui.add(Link::new(&release.release.title));
        if let Some(artist) = &release.release.artist {
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
        ui.add(Link::new(&release.release.title));
        if let Some(artist) = &release.release.artist {
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

