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
use dimple_core::model::Model;
use dimple_librarian::librarian::Librarian;
use image::DynamicImage;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use slint::Image;
use slint::ModelRc;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;

// https://releases.slint.dev/1.5.1/docs/rust/slint/struct.image#sending-image-to-a-thread
// https://github.com/slint-ui/slint/discussions/4289
// https://github.com/slint-ui/slint/discussions/2527
pub fn artist_list(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    let librarian = librarian.clone();
    let ui = ui.clone();
    std::thread::spawn(move || {
        // Get the artists and sort them.
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

        // TODO probable race condition with above
        // artists.iter().enumerate().for_each(|(i, artist)| {
        //     // let ui = ui.clone();
        //     // TODO Palette for thumbnail sizes
        //     let image_buf = get_artist_image(artist, 200, 200);
        //     ui.upgrade_in_event_loop(move |ui| {
        //         let image = Image::from_rgba8_premultiplied(image_buf);
        //         let adapter = ui.get_artist_list();
        //         let mut card: LazyCardAdapter = slint::Model::row_data(&adapter.cards, i).unwrap();
        //         card.image.image.image = image;
        //         slint::Model::set_row_data(&adapter.cards, i, card);
        //     }).unwrap();
        //     thread::sleep(Duration::from_millis(100));
        // });
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

/// A magical function that loads or creates an image for the specified model
/// based on currently available data. Calling it again might produce a
/// a different image if related data has been updated.
pub fn get_artist_image(artist: &Artist, width: u32, height: u32) -> SharedPixelBuffer<Rgba8Pixel> {
    // log::info!("generate {}x{} for {:?}", width, height, artist.name);
    // thread::sleep(Duration::from_secs(3));
    // log::info!("okay it's done {}x{} for {:?}", width, height, artist.name);
    let image = image::open("images/light.png").expect("Error loading demo image").into_rgba8();
    let image_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        image.as_raw(),
        image.width(),
        image.height(),
    );
    image_buf
}
