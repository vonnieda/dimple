use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::model::Artist;
use slint::ModelRc;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use slint::Model as _;

pub fn artist_list_init(app: &App) {
    let app = app.clone();
    let library = app.library.clone();
    library.on_change(Box::new(move |_event| update_model(&app)));
}

pub fn artist_list(app: &App) {
    update_model(app);
    app.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::ArtistList)).unwrap();
}

fn update_model(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        let library = app.library.clone();
        let artists = library.query("
            SELECT * 
            FROM Artist
            ORDER BY name ASC, disambiguation ASC
        ", ());
        let ui = app.ui.clone();
        let images = app.images.clone();
        ui.upgrade_in_event_loop(move |ui| {
            let cards = artist_cards(&images, &artists);
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_artist_list(adapter);
        }).unwrap();
    });
}

fn artist_cards(images: &ImageMangler, artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned().enumerate()
        .map(|(index, artist)| {
            let mut card: CardAdapter = artist_card(&artist);
            card.image.image = images.lazy_get(artist.clone(), 200, 200, move |ui, image| {
                let mut card = ui.get_artist_list().cards.row_data(index).unwrap();
                card.image.image = image;
                ui.get_artist_list().cards.set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn artist_card(artist: &Artist) -> CardAdapter {
    let artist = artist.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: artist.disambiguation.unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

