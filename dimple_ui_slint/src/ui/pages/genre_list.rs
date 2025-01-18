use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use slint::ModelRc;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;
use slint::Model as _;

pub fn genre_list_init(app: &App) {
    let app = app.clone();
    let library = app.library.clone();
    // library.on_change(Box::new(move |_event| update_model(&app)));
}

pub fn genre_list(app: &App) {
    update_model(app);
    app.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::GenreList)).unwrap();
}

fn update_model(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        let library = app.library.clone();
        let genres = library.query("
            SELECT * 
            FROM Genre 
            ORDER BY name ASC, disambiguation ASC", ());
        let ui = app.ui.clone();
        let images = app.images.clone();
        ui.upgrade_in_event_loop(move |ui| {
            let cards = genre_cards(&images, &genres);
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_genre_list(adapter);
        }).unwrap();
    });
}

fn genre_cards(images: &ImageMangler, genres: &[Genre]) -> Vec<CardAdapter> {
    genres.iter().cloned().enumerate()
        .map(|(index, genre)| {
            let mut card: CardAdapter = genre_card(&genre);
            card.image.image = images.lazy_get(genre.clone(), 200, 200, move |ui, image| {
                let mut card = ui.get_genre_list().cards.row_data(index).unwrap();
                card.image.image = image;
                ui.get_genre_list().cards.set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn genre_card(genre: &Genre) -> CardAdapter {
    let genre = genre.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: genre.disambiguation.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.key.clone().unwrap_or_default()).into(),
        },
    }
}

