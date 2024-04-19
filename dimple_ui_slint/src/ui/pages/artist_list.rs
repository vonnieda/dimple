use std::collections::VecDeque;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::ui::images::get_model_image;
use crate::ui::images::lazy_load_images;
use crate::ui::AppWindow;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Artist;
use dimple_librarian::librarian::Librarian;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use slint::ComponentHandle;
use slint::Image;
use slint::Model;
use slint::ModelRc;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;
use slint::Weak;

// https://releases.slint.dev/1.5.1/docs/rust/slint/struct.image#sending-image-to-a-thread
// https://github.com/slint-ui/slint/discussions/4289
// https://github.com/slint-ui/slint/discussions/2527

/// Current thinking is need a placeholder image, and then set later.
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
            let models: Vec<dimple_core::model::Model> = artists.iter().cloned().map(Into::into).collect();
            lazy_load_images(&librarian, models.as_slice(), ui.as_weak(), |ui| ui.get_artist_list().cards);
        }).unwrap();
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

