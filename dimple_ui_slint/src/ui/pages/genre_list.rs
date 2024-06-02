use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use slint::Model;
use slint::ModelRc;

pub fn genre_list(app: &App) {
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {
        let mut genres: Vec<Genre> = librarian
            .list(&Genre::default().into(), &None)
            .unwrap()
            .map(Into::into)
            .collect();
        genres.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());

        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = genres.iter().cloned().enumerate()
                .map(|(index, genre)| {
                    let mut card: CardAdapter = genre.clone().into();
                    card.image.image = images.lazy_get(genre.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_genre_list().cards.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_genre_list().cards.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_genre_list(adapter);
            ui.set_page(Page::GenreList);
        }).unwrap();
    });
}
