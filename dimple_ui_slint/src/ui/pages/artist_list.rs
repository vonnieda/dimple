use std::thread;
use std::time::Duration;

use crate::ui::AppWindow;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::ImageLinkAdapter;
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
    {
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
                let cards: Vec<CardAdapter> = artists.iter().map(from_ref_artist_for_card_adapter).collect();
                let adapter = CardGridAdapter {
                    cards: ModelRc::from(cards.as_slice()),
                };
                ui.set_artist_list(adapter);
                ui.set_page(Page::ArtistList);
            })
            .unwrap();
        });
    }

    {
        let librarian = librarian.clone();
        let ui = ui.clone();
        std::thread::spawn(move || {
            thread::sleep(Duration::from_secs(5));
            log::info!("here we go!");
            ui.upgrade_in_event_loop(move |ui| {
                let adapter = ui.get_artist_list();
                // let pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(640, 480);
                // let image = Image::from_rgba8_premultiplied(pixel_buffer);

                let mut demo_image = image::open("images/light.png").expect("Error loading demo image").into_rgba8();

                image::imageops::colorops::brighten_in_place(&mut demo_image, 20);
                
                let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    demo_image.as_raw(),
                    demo_image.width(),
                    demo_image.height(),
                );
                let image = Image::from_rgba8(buffer);

                for (i, card) in adapter.cards.iter().enumerate() {
                    let mut card = card.clone();
                    card.image.image = image.clone();
                    card.title.name = "Wow".to_string().into();
                    adapter.cards.set_row_data(i, card);
                }
            })
            .unwrap();
        });
    }
}

fn from_ref_artist_for_card_adapter(value: &Artist) -> CardAdapter {
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(), // HOW NOW?!
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

impl From<&Artist> for CardAdapter {
    fn from(value: &Artist) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(), // HOW NOW?!
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
