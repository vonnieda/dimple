use std::collections::HashMap;

use dimple_core::{library::LibraryHandle, model::Image};
use dimple_player::player::PlayerHandle;
use image::DynamicImage;
use slint::{Weak, ModelRc, SharedString};

slint::include_modules!();

pub struct AppWindowController {
    ui: AppWindow,
    library: LibraryHandle,
    player: PlayerHandle,
    images_by_url: HashMap<String, slint::Image>,
}

impl AppWindowController {
    pub fn new(library: LibraryHandle, player: PlayerHandle) -> Self {
        let ui = AppWindow::new().unwrap();

        Self {
            ui,
            library,
            player,
            images_by_url: HashMap::new(),
        }
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui_1 = self.ui.as_weak();
        self.ui.global::<Navigator>().on_navigate(move |url| self.navigate(ui_1.clone(), url));

        self.ui.run()
    }

    fn image(&mut self, image: &Image) -> slint::Image {
        if let Some(image) = self.images_by_url.get(&image.url) {
            return image.clone();
        }

        if let Ok(dyn_image) = self.library.image(image) {
            let slint_image = crate::images::dynamic_image_to_slint_image(&dyn_image);
            self.images_by_url.insert(image.url.clone(), slint_image.clone());
            return slint_image;
        }

        slint::Image::default()
    }

    fn navigate(&self, ui: Weak<AppWindow>, url: slint::SharedString) {
        println!("{}", url);
        if url == "dimple://home" {
            home(ui, url);
        }
        else if url == "dimple://artists" {
            artists(ui, url);
        }
        else if url == "dimple://releases" {
            releases(ui, url);
        }
        else if url == "dimple://genres" {
            genres(ui, url);
        }
        else {
            home(ui, url);
        }
    }    
}

// struct DynamicImageCard {
//     title: String,
//     sub_title: String,
//     image: DynamicImage,
//     url: String,
// }

fn home(ui: Weak<AppWindow>, url: slint::SharedString) {
    ui.upgrade_in_event_loop(move |ui| {
        // Clear the model
        let mut model: CardGridModel = ui.get_card_grid_model();
        // Switch the page
        ui.set_page(0);
        // Start a worker to execute the query
        // thread::spawn({
        //     // Execute the query
        //     // Convert the results to CardModels
        //     // TODO see if I can stream them in as I load images? Might mean
        //     // setting the model.
        // });
        // let cards: Vec<CardModel> = vec![
        //     CardModel {
        //         image: ImageLink {
        //             name: "Test".into(),
        //             image: slint::Image::default(),
        //             url: "dimple://artists".into(),
        //         },
        //         title: Link {
        //             name: "Test".into(),
        //             url: "dimple://artists".into(),
        //         },
        //         sub_title: Link {
        //             name: "Test".into(),
        //             url: "dimple://artists".into(),
        //         },
        //     },
        // ];
        // ui.set_page(0);
        // let mut model: CardGridModel = ui.get_card_grid_model();
        // model.cards = ModelRc::from(cards.as_slice());
        // ui.set_card_grid_model(model);
    }).unwrap();
}

fn artists(ui: Weak<AppWindow>, url: slint::SharedString) {
    ui.upgrade_in_event_loop(move |ui| {
    });
}

fn releases(ui: Weak<AppWindow>, url: slint::SharedString) {
    ui.upgrade_in_event_loop(move |ui| {
    });
}

fn genres(ui: Weak<AppWindow>, url: slint::SharedString) {
    ui.upgrade_in_event_loop(move |ui| {
    });
}








//                 log::info!("Converting artist cards to card models.");
//                 let cards: Vec<CardModel> = artist_cards.into_iter()
//                     .map(Into::into)
//                     .collect();

//                 log::info!("Setting cards.");
//                 ui.set_cards(ModelRc::from(cards.as_slice())); 
//                 log::info!("Done setting cards.");

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
//         url: artist.url.clone(),
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
//         url: release.url.clone(),
//     }
// }

// fn genre_to_card(genre: &Genre, library: LibraryHandle) -> DynamicImageCard {
//     // let images = genre.art();
//     // let image = images.first().unwrap();
//     // let dynamic_image = library.image(image).unwrap();
//     DynamicImageCard {
//         title: genre.name.clone(),
//         sub_title: "".into(),
//         image: crate::image::generate_abstract_image(500, 500, &[], &[]),
//         url: genre.url.clone(),
//     }
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

// fn dynamic_image_to_slint_image(dynamic_image: &DynamicImage) -> slint::Image {
//     // TODO may be possible to limit the number of copies here
//     let rgb8_image = dynamic_image.clone().into_rgb8();
//     let shared_pixbuf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
//         rgb8_image.as_raw(),
//         rgb8_image.width(),
//         rgb8_image.height(),
//     );
//     slint::Image::from_rgb8(shared_pixbuf)
// }

// impl From<DynamicImageCard> for CardModel {
//     fn from(value: DynamicImageCard) -> Self {
//         CardModel {
//             title: value.title.clone().into(),
//             sub_title: value.sub_title.clone().into(),
//             image: dynamic_image_to_slint_image(&value.image),
//             url: value.url.into(),
//         }
//     }
// }

// /// Load the given image from cache, or from the library and then cache it,
// /// and return it.
// fn dimple_image_to_dynamic_image(dimple_image: &Image, library: &dyn Library) -> DynamicImage {
// }

