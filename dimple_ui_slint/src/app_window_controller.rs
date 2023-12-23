use std::sync::Arc;

use dimple_core::{library::{Library, LibraryHandle}, model::{Artist, HasArtwork}};
use dimple_librarian::librarian::Librarian;
use dimple_navidrome_library::navidrome_library::NavidromeLibrary;
use dimple_player::player::{Player, PlayerHandle};
use image::DynamicImage;
use slint::{ModelRc, Weak, SharedPixelBuffer, Rgba8Pixel};
use dimple_folder_library::folder_library::FolderLibrary;

slint::include_modules!();

pub type LibrarianHandle = Arc<Librarian>;

pub struct AppWindowController {
    ui: AppWindow,
    librarian: LibrarianHandle,
    player: PlayerHandle,
}

struct DynamicImageCard {
    title: String,
    sub_title: String,
    image: DynamicImage,
}

impl AppWindowController {
    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| {
            dbg!(&url);
            let ui = ui.unwrap();
            if url.starts_with("dimple://home") {
                Self::nav_artists(librarian.clone(), ui.as_weak());
                ui.set_page(0);
            } else if url.starts_with("dimple://artists") {
                ui.set_page(1);
            } else if url.starts_with("dimple://releases") {
                ui.set_page(2);
            } else if url.starts_with("dimple://genres") {
                ui.set_page(3);
            } else if url.starts_with("dimple://search") {
                ui.set_page(4);
            }
        });

        // let navidrome = NavidromeLibrary::new(
        //     "01GX2DEYHSG85BDN3ZVV94A7BP",
        //     "Moof",
        //     "http://navidrome.moof.vonnieda.org",
        //     "jason",
        //     "panic stricken raging bull",
        // );
        // self.librarian.add_library(Arc::new(navidrome));
        let folder = FolderLibrary::new("/Users/jason/Music/My Music");
        self.librarian.add_library(Arc::new(folder));
        let librarian = self.librarian.clone();
        std::thread::spawn(move || {
            librarian.refresh_all_libraries();
        });

        self.ui.run()
    }

    fn nav_artists(library: LibraryHandle, ui: Weak<AppWindow>) {
        std::thread::spawn(move || {
            log::info!("Getting artists.");
            let artists = library.artists();

            log::info!("Making artist cards.");
            let artist_cards: Vec<DynamicImageCard> = artists.iter()
                .map(|artist| {
                    artist_to_card(artist, library.clone())
                })
                .collect();

            log::info!("Sending cards to UI.");
            ui.upgrade_in_event_loop(move |ui| { 
                    log::info!("Converting artist cards to card models.");
                    let cards: Vec<CardModel> = artist_cards.into_iter()
                        .map(Into::into)
                        .collect();

                    log::info!("Setting cards.");
                    ui.set_card_grid_model(CardGridModel { cards: ModelRc::from(cards.as_slice()) }); 
                    log::info!("Done setting cards.");
                })
                .unwrap();
        });
    }    
}

impl Default for AppWindowController {
    fn default() -> Self {
        let ui = AppWindow::new().unwrap();
        let librarian = Arc::new(Librarian::default());
        let player = Player::new(librarian.clone());
        Self {
            ui,
            librarian,
            player,
        }
    }
}

fn artist_to_card(artist: &Artist, library: LibraryHandle) -> DynamicImageCard {
    // TODO wow, yikes. How many times do we clone / shuffle the
    // image data here?
    let images = artist.art();
    let image = images.first().unwrap();
    let dynamic_image = library.image(image).unwrap();
    DynamicImageCard {
        title: artist.name.clone(),
        sub_title: "".into(),
        image: dynamic_image,
    }
}

impl From<DynamicImageCard> for CardModel {
    fn from(value: DynamicImageCard) -> Self {
        CardModel {
            title: Link { name: value.title.clone().into(), url: "".into() },
            sub_title: Link { name: value.sub_title.clone().into(), url: "".into() },
            image: ImageLink { image: dynamic_image_to_slint_image(&value.image), name: "".into(), url: "".into() },
        }
    }
}

fn dynamic_image_to_slint_image(dynamic_image: &DynamicImage) -> slint::Image {
    let rgba8_image = dynamic_image.clone().into_rgba8();
    let shared_pixbuf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba8_image.as_raw(),
        rgba8_image.width(),
        rgba8_image.height(),
    );
    slint::Image::from_rgba8(shared_pixbuf)
}


// use std::sync::Arc;

// use dimple_core::{library::LibraryHandle, model::{HasArtwork, Artist}};
// use dimple_ui_slint::{settings::Settings, librarian::Librarian};
// use slint::{SharedString, ModelRc, Image, SharedPixelBuffer, Rgba8Pixel};

// slint::include_modules!();

// struct ArtistCard {
//     title: String,
//     image: SharedPixelBuffer<Rgba8Pixel>,
// }

// fn main() -> Result<(), slint::PlatformError> {
//     let mut builder = env_logger::Builder::new();
//     builder.filter_level(log::LevelFilter::Info);
//     builder.format_timestamp_millis();
//     builder.parse_default_env();
//     builder.init();
//     log::info!("Log initialized.");

//     log::info!("Loading settings.");
//     let settings = Settings::default();

//     log::info!("Loading libraries.");
//     let librarian: Arc<Librarian> = Arc::new(Librarian::from(settings.libraries));
//     let library: LibraryHandle = librarian.clone();

//     log::info!("Initializing UI.");
//     let ui = AppWindow::new()?;

//     let ui_1 = ui.as_weak();
//     let library_1 = library.clone();
//     ui.on_nav_home(move || {
//         let ui_2 = ui_1.clone();
//         let library_2 = library_1.clone();
//         std::thread::spawn(move || {
//             log::info!("Getting artists.");
//             let artists = library_2.artists();
//             log::info!("Making cards.");
//             let artist_cards: Vec<ArtistCard> = artists.iter().map(|artist| {
//                 // TODO wow, yikes. How many times do we clone / shuffle the
//                 // image data here?
//                 let images = artist.art();
//                 let image = images.first().unwrap();
//                 let dynamic_image = library_2.image(image).unwrap();
//                 let rgba8_image = dynamic_image.into_rgba8();
//                 let shared_pixbuf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
//                     rgba8_image.as_raw(),
//                     rgba8_image.width(),
//                     rgba8_image.height(),
//                 );
//                 ArtistCard {
//                     title: artist.name.clone(),
//                     image: shared_pixbuf,
//                 }
//             }).collect();
//             log::info!("Sending cards to UI.");
//             ui_2.upgrade_in_event_loop(move |ui| { 
//                     log::info!("Converting cards to models.");
//                     let cards: Vec<CardModel> = artist_cards.iter().map(|artist_card| {
//                         CardModel {
//                             title: artist_card.title.clone().into(),
//                             sub_title: "".into(),
//                             image: slint::Image::from_rgba8(artist_card.image.clone())
//                         }
//                     })
//                     .collect();
//                     log::info!("Converting model to ModelRc.");
//                     let model = ModelRc::from(cards.as_slice());
//                     log::info!("Setting cards.");
//                     ui.set_cards(model); 
//                     log::info!("Done setting cards.");
//                 })
//                 .unwrap();
//             });
//     });

//     // let ui_handle = ui.as_weak();
//     // ui.on_nav_search(move |query: SharedString| {
//     //     let ui = ui_handle.unwrap();
//     // });

//     // log::info!("Starting Librarian.");
//     // std::thread::spawn(move || {
//     //     log::info!("Refreshing libraries.");
//     //     librarian.refresh_all_libraries();
//     // });

//     log::info!("Running UI.");

// use std::sync::Arc;

// use dimple_core::{library::{LibraryHandle, Library}, model::{HasArtwork, Artist, Release, Genre}};
// use dimple_ui_slint::{settings::Settings, librarian::Librarian};
// use image::{DynamicImage, RgbaImage};
// use slint::{SharedString, ModelRc, Image, SharedPixelBuffer, Rgba8Pixel, Weak};

// slint::include_modules!();

// struct DynamicImageCard {
//     title: String,
//     sub_title: String,
//     image: DynamicImage,
// }

// fn main() -> Result<(), slint::PlatformError> {
//     let mut builder = env_logger::Builder::new();
//     builder.filter_level(log::LevelFilter::Info);
//     builder.format_timestamp_millis();
//     builder.parse_default_env();
//     builder.init();
//     log::info!("Log initialized.");

//     log::info!("Loading settings.");
//     let settings = Settings::default();

//     log::info!("Loading libraries.");
//     let librarian: Arc<Librarian> = Arc::new(Librarian::from(settings.libraries));
//     let library: LibraryHandle = librarian.clone();

//     log::info!("Initializing UI.");
//     let ui = AppWindow::new()?;

//     let library_1 = library.clone();
//     let ui_1 = ui.as_weak();
//     ui.on_nav_home(move || {
//         nav_home(library_1.clone(), ui_1.clone());
//     });

//     let library_1 = library.clone();
//     let ui_1 = ui.as_weak();
//     ui.on_nav_albums(move || {
//         nav_albums(library_1.clone(), ui_1.clone());
//     });

//     let library_1 = library.clone();
//     let ui_1 = ui.as_weak();
//     ui.on_nav_artists(move || {
//         nav_artists(library_1.clone(), ui_1.clone());
//     });

//     let library_1 = library.clone();
//     let ui_1 = ui.as_weak();
//     ui.on_nav_genres(move || {
//         nav_genres(library_1.clone(), ui_1.clone());
//     });

//     ui.invoke_nav_home();

//     // let ui_handle = ui.as_weak();
//     // ui.on_nav_search(move |query: SharedString| {
//     //     let ui = ui_handle.unwrap();
//     // });

//     // log::info!("Starting Librarian.");
//     // std::thread::spawn(move || {
//     //     log::info!("Refreshing libraries.");
//     //     librarian.refresh_all_libraries();
//     // });

//     log::info!("Running UI.");
//     ui.run()
// }

// fn nav_home(library: LibraryHandle, ui: Weak<AppWindow>) {
//     nav_artists(library, ui)
// }

// fn nav_artists(library: LibraryHandle, ui: Weak<AppWindow>) {
//     std::thread::spawn(move || {
//         log::info!("Getting artists.");
//         let artists = library.artists();

//         log::info!("Making artist cards.");
//         let artist_cards: Vec<DynamicImageCard> = artists.iter()
//             .map(|artist| {
//                 artist_to_card(artist, library.clone())
//             })
//             .collect();

//         log::info!("Sending cards to UI.");
//         ui.upgrade_in_event_loop(move |ui| { 
//                 log::info!("Converting artist cards to card models.");
//                 let cards: Vec<CardModel> = artist_cards.into_iter()
//                     .map(Into::into)
//                     .collect();

//                 log::info!("Setting cards.");
//                 ui.set_cards(ModelRc::from(cards.as_slice())); 
//                 log::info!("Done setting cards.");
//             })
//             .unwrap();
//     });
// }

// fn nav_albums(library: LibraryHandle, ui: Weak<AppWindow>) {
//     std::thread::spawn(move || {
//         log::info!("Getting albums.");
//         let releases = library.releases();

//         log::info!("Making album cards.");
//         let album_cards: Vec<DynamicImageCard> = releases.iter()
//             .map(|release| {
//                 release_to_card(&release, library.clone())
//             })
//             .collect();

//         log::info!("Sending cards to UI.");
//         ui.upgrade_in_event_loop(move |ui| { 
//                 log::info!("Converting album cards to card models.");
//                 let cards: Vec<CardModel> = album_cards.into_iter()
//                     .map(Into::into)
//                     .collect();

//                 log::info!("Setting cards.");
//                 ui.set_cards(ModelRc::from(cards.as_slice())); 
//                 log::info!("Done setting cards.");
//             })
//             .unwrap();
//     });
// }

// fn nav_genres(library: LibraryHandle, ui: Weak<AppWindow>) {
//     std::thread::spawn(move || {
//         log::info!("Getting genres.");
//         let objects = library.genres();

//         log::info!("Making cards.");
//         let cards: Vec<DynamicImageCard> = objects.iter()
//             .map(|o| {
//                 genre_to_card(o, library.clone())
//             })
//             .collect();

//         log::info!("Sending cards to UI.");
//         ui.upgrade_in_event_loop(move |ui| { 
//                 log::info!("Converting cards to card models.");
//                 let cards: Vec<CardModel> = cards.into_iter()
//                     .map(Into::into)
//                     .collect();

//                 log::info!("Setting cards.");
//                 ui.set_cards(ModelRc::from(cards.as_slice())); 
//                 log::info!("Done setting cards.");
//             })
//             .unwrap();
//     });
// }

// fn dynamic_image_to_slint_image(dynamic_image: &DynamicImage) -> slint::Image {
//     let rgba8_image = dynamic_image.clone().into_rgba8();
//     let shared_pixbuf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
//         rgba8_image.as_raw(),
//         rgba8_image.width(),
//         rgba8_image.height(),
//     );
//     slint::Image::from_rgba8(shared_pixbuf)
// }

// fn artist_to_card(artist: &Artist, library: LibraryHandle) -> DynamicImageCard {
//     // TODO wow, yikes. How many times do we clone / shuffle the
//     // image data here?
//     let images = artist.art();
//     let image = images.first().unwrap();
//     let dynamic_image = library.image(image).unwrap();
//     DynamicImageCard {
//         title: artist.name.clone(),
//         sub_title: "".into(),
//         image: dynamic_image,
//     }
// }

// fn release_to_card(release: &Release, library: LibraryHandle) -> DynamicImageCard {
//     let images = release.art();
//     let image = images.first().unwrap();
//     let dynamic_image = library.image(image).unwrap();
//     DynamicImageCard {
//         title: release.title.clone(),
//         sub_title: release.artist(),
//         image: dynamic_image,
//     }
// }

// fn genre_to_card(genre: &Genre, library: LibraryHandle) -> DynamicImageCard {
//     // let images = genre.art();
//     // let image = images.first().unwrap();
//     // let dynamic_image = library.image(image).unwrap();
//     DynamicImageCard {
//         title: genre.name.clone(),
//         sub_title: "".into(),
//         image: RgbaImage::new(500, 500).into(),
//     }
// }

// impl From<DynamicImageCard> for CardModel {
//     fn from(value: DynamicImageCard) -> Self {
//         CardModel {
//             title: value.title.clone().into(),
//             sub_title: value.sub_title.clone().into(),
//             image: dynamic_image_to_slint_image(&value.image),
//         }
//     }
// }
