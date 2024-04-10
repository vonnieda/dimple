use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use crate::ui::AppWindow;
use crate::ui::CardAdapter;
use crate::ui::LazyImage;
use crate::ui::LazyCardAdapter;
use crate::ui::LazyImageStatus;
use crate::ui::CardGridAdapter;
use crate::ui::LazyCardGridAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LazyImageLinkAdapter;
use crate::ui::LinkAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Artist;
use dimple_librarian::librarian::Librarian;
use image::DynamicImage;
use slint::Image;
use slint::Model;
use slint::ModelRc;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;

// https://releases.slint.dev/1.5.1/docs/rust/slint/struct.image#sending-image-to-a-thread
// https://github.com/slint-ui/slint/discussions/4289
// https://github.com/slint-ui/slint/discussions/2527
pub fn artist_list(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    let librarian = librarian.clone();
    let ui = ui.clone();
    // First get the artist list and metadata, and set it on the UI, so that
    // the page shows quickly. Then we'll go back and fill in images.
    std::thread::spawn(move || {
        let mut artists: Vec<Artist> = librarian
            .list(&Artist::default().into(), None)
            .unwrap()
            .map(Into::into)
            .collect();
        artists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());
        {
            let artists = artists.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let cards: Vec<LazyCardAdapter> = artists.iter().map(Into::into).collect();
                let adapter = LazyCardGridAdapter {
                    cards: ModelRc::from(cards.as_slice()),
                };
                ui.set_artist_list(adapter);
                ui.set_page(Page::ArtistList);
            }).unwrap();
        }

        // Fill in images.
        // TODO thinking of a deal where we loop through the models
        // continuously (every 100ms) filling in any images that are marked
        // not loaded with any images that have since been loaded until none
        // are marked not loaded?
        // TODO this should just become a utility function I can pass a RcModel
        // of CardAdapters to, I think.
        loop {
            ui.upgrade_in_event_loop(move |ui| {
                let adapter = ui.get_artist_list();
                for (i, card) in adapter.cards.iter().enumerate() {
                    match card.image.image.status {
                        LazyImageStatus::NotLoaded => {

                        },
                        _ => todo!(),
                    }
                }
            }).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
        // for (i, artist) in artists.iter().enumerate() {
        //     // TODO trash, but testing. this is a race condition with
        //     // the above populating the adapters, and it's also bad for
        //     // doing these upgrades in the loop. Should batch them or 
        //     // something?
        //     let artist = artist.clone();
        //     ui.upgrade_in_event_loop(move |ui| {
        //         log::info!("look up image for artist {:?} {:?}", artist.name, artist.key);
        //         let image = get_artist_image(artist.key.unwrap());
        //         let adapter = ui.get_artist_list();
        //         let mut card = adapter.cards.row_data(i).unwrap();
        //         card.image.image = image;
        //         adapter.cards.set_row_data(i, card);
        //     }).unwrap();
        // }
    });
}

impl From<&Artist> for CardAdapter {
    fn from(value: &Artist) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            sub_title: LinkAdapter {
                name: value.disambiguation.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
        }
    }
}

impl From<&Artist> for LazyCardAdapter {
    fn from(value: &Artist) -> Self {
        LazyCardAdapter {
            image: LazyImageLinkAdapter {
                image: LazyImage {
                    key: value.key.clone().unwrap().into(),
                    status: LazyImageStatus::NotLoaded,
                    image: Default::default()
                },
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            sub_title: LinkAdapter {
                name: value.disambiguation.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
        }
    }
}


//             let mut demo_image = image::open("images/light.png").expect("Error loading demo image").into_rgba8();

//             image::imageops::colorops::brighten_in_place(&mut demo_image, 20);
            
//             let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
//                 demo_image.as_raw(),
//                 demo_image.width(),
//                 demo_image.height(),
//             );
//             let image = Image::from_rgba8(buffer);

//             for (i, card) in adapter.cards.iter().enumerate() {
//                 let mut card = card.clone();
//                 card.image.image = image.clone();
//                 card.title.name = "Wow".to_string().into();
//                 adapter.cards.set_row_data(i, card);
//             }
