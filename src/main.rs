use eframe::egui::{self, Grid, ImageButton, Link, Response, ScrollArea, TextEdit, Ui, Context};
use eframe::epaint::{FontFamily, FontId, ColorImage};
use egui_extras::RetainedImage;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use image::DynamicImage;
use music_library::local::LocalMusicLibrary;
use music_library::{MusicLibrary, Release};
mod music_library;

// TODO make grid full width
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


// TODO someday this is a Trait that lays it's own UI out so that we can have
// more than just title and subtitle.
struct Card {
    // TODO id?
    // TODO reference to object
    image: RetainedImage,
    title: String,
    subtitle: String,
}

impl Default for Card {
    fn default() -> Self {
        Self { 
            image: RetainedImage::from_color_image("default", ColorImage::example()),
            title: Default::default(), 
            subtitle: Default::default() 
        }
    }
}

struct App {
    query_string: String,
    cards: Vec<Card>,
    now_playing: Card,
    up_next: Card,
}

impl Default for App {
    fn default() -> Self {
        // load the local music library
        println!("Loading local library");
        let library = LocalMusicLibrary::new("data/library");
        let releases = library.releases();
        println!("Local library contains {} releases", releases.len());

        // collect the releases from the music library
        println!("Releases -> Cards");
        let mut cards = Vec::new();
        for release in releases {
            let image = match release.cover_image {
                Some(image) => image,
                None => create_release_image(&release),
            };
            let card = Card {
                title: release.title.clone(),
                subtitle: release.artist.unwrap_or(String::from("")),
                image: dynamic_to_retained(&release.title, &image),
            };
            cards.push(card);
        }

        // sort the cards
        println!("Sorting Cards");
        cards.sort_by(|a, b| {
            a.subtitle.to_uppercase().cmp(&b.subtitle.to_uppercase())
        });

        // off we go
        println!("Done");
        return Self {
            query_string: String::from(""),
            cards,
            now_playing: Default::default(),
            up_next: Default::default(),
        };
    }
}

impl App {
    fn browser(self: &mut Self, ctx: &Context) {
        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            self.search_bar(ui);
        });

        egui::TopBottomPanel::bottom("player").show(ctx, |ui| {
            self.player_bar(ctx, ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let matcher = SkimMatcherV2::default();
            let cards: Vec<&Card> = self.cards
                .iter()
                .filter(|card| {
                    let haystack = card.title.clone() + " " + &card.subtitle;
                    return matcher.fuzzy_match(
                        haystack.as_str(), 
                        &self.query_string
                    ).is_some();
                })
                .collect();
            self.card_grid(&cards, ctx, ui)
        });
    }

    fn search_bar(self: &mut Self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut self.query_string)
                    .hint_text("What sounds good?")
                    // TODO how do I use the theme font? Or the default, more specifically.
                    .font(FontId::new(28.0, FontFamily::Proportional))
                    .desired_width(f32::INFINITY),
            );
        })
        .response
    }

    fn card_grid(self: &Self, cards: &Vec<&Card>, ctx: &Context, ui: &mut Ui) -> Response {
        let num_columns = 6;

        // TODO use ScrollArea::show_rows to improve performance. I 
        // tried previously and I couldn't get the rendering right.
        // Oh, a hint, might also need Grid::show_rows
        ui.vertical_centered_justified(|ui| {
            ui.spacing_mut().scroll_bar_width = 20.0;
            ui.spacing_mut().scroll_handle_min_length = 28.0;
            ui.spacing_mut().scroll_bar_outer_margin = 10.0;
            ui.spacing_mut().scroll_bar_inner_margin = 10.0;
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Grid::new("card_grid")
                    .spacing(egui::vec2(16.0, 16.0))
                    .show(ui, |ui| {
                        for (i, card) in cards.iter().enumerate() {
                            self.card(&card, 200.0, 200.0, ctx, ui);
                            if i % num_columns == num_columns - 1 {
                                ui.end_row();
                            }
                        }
                    });
                });
            });
        }).response
    }
    
    fn card(self: &Self, card: &Card, width: f32, height: f32, ctx: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.add(ImageButton::new(card.image.texture_id(ctx), 
                egui::vec2(width, height)));
            ui.add(Link::new(&card.title));
            ui.add(Link::new(&card.subtitle));
        })
        .response
    }
    
    fn player_bar(self: &Self, ctx: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let np = &self.now_playing;
                ui.add(ImageButton::new(np.image.texture_id(ctx), 
                    egui::vec2(120.0, 120.0)));
                ui.vertical(|ui| {
                    ui.link(&np.title);
                    ui.link(&np.subtitle);
                    // visual_scrubber(release, ctx, ui);
                    self.slider_scrubber(ctx, ui);
                });
                self.card(&self.up_next, 60.0, 60.0, ctx, ui);
            })    
        })
        .response
    }
    
    // fn visual_scrubber(release: &CachedRelease, ctx: &Context, ui: &mut Ui) -> Response {
    // }
    
    fn slider_scrubber(self: &Self, ctx: &Context, ui: &mut Ui) -> Response {
        // TODO magic
        ui.spacing_mut().slider_width = 1000.0;
        let mut my_f32:f32 = 0.33;
        ui.add(egui::Slider::new(&mut my_f32, 0.0..=1.0)
            .show_value(false)
            .trailing_fill(true)
        )
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
        
        // ctx.set_debug_on_hover(true);

        self.browser(ctx);
    }
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

// TODO it would be fun to generate a cool artwork for the release
// based on maybe a similar function that generates artwork for a genre
// like use the genre as a background for a stylized title or something
fn create_release_image(_release: &Release) -> DynamicImage {
    DynamicImage::new_rgba8(200, 200)
}


        // load config
        // let builder = Config::builder()
        //     .add_source(File::new("config", FileFormat::Toml));


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
        
        