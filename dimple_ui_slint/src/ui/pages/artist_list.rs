use std::sync::mpsc::channel;
use std::thread;

use crate::ui::images::get_model_image;
use crate::ui::AppWindow;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Artist;
use dimple_librarian::librarian::Librarian;
use slint::ComponentHandle;
use slint::Image;
use slint::Model;
use slint::ModelRc;

// https://releases.slint.dev/1.5.1/docs/rust/slint/struct.image#sending-image-to-a-thread
// https://github.com/slint-ui/slint/discussions/4289
// https://github.com/slint-ui/slint/discussions/2527
pub fn artist_list(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    let librarian = librarian.clone();
    let ui = ui.clone();
    std::thread::spawn(move || {
        let mut artists: Vec<Artist> = librarian
            .list(&Artist::default().into(), None)
            .unwrap()
            .map(Into::into)
            .collect();
        artists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());

        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = artists.iter().cloned().map(Into::into).collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_artist_list(adapter);
            ui.set_page(Page::ArtistList);
            load_images(&librarian, &artists, ui.as_weak());
        }).unwrap();
    });
}

fn load_images(librarian: &Librarian, artists: &[Artist], ui: slint::Weak<AppWindow>) {
    let artists: Vec<_> = artists.iter().cloned().collect();
    let librarian = librarian.clone();
    thread::spawn(move || {
        // TODO this needs to be improved to not spam the UI thread. 
        artists.iter().enumerate().for_each(|(i, artist)| {
            // TODO Use Palette for thumbnail sizes
            let image = get_model_image(&librarian, &artist.into(), 200, 200);
            ui.upgrade_in_event_loop(move |ui| {
                let image = Image::from_rgba8_premultiplied(image);
                let adapter = ui.get_artist_list();
                let mut card: CardAdapter = adapter.cards.row_data(i).unwrap();
                card.image.image = image;
                adapter.cards.set_row_data(i, card);
            }).unwrap();
        });
    });
}

impl From<Artist> for CardAdapter {
    fn from(value: Artist) -> Self {
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

