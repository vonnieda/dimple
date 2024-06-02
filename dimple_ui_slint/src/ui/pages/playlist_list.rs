use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::db::Db;
use dimple_core::model::Entity;
use dimple_core::model::Playlist;
use slint::Model;
use slint::ModelRc;

pub fn playlist_list(app: &App) {
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {
        let mut playlists: Vec<Playlist> = librarian
            .list(&Playlist::default().into(), &None)
            .unwrap()
            .map(Into::into)
            .collect();
        playlists.sort_by_key(|a| a.name.clone().unwrap_or_default().to_lowercase());

        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = playlists.iter().cloned().enumerate()
                .map(|(index, playlist)| {
                    let mut card: CardAdapter = playlist.clone().into();
                    card.image.image = images.lazy_get(playlist.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_playlist_list().cards.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_playlist_list().cards.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let adapter = CardGridAdapter {
                cards: ModelRc::from(cards.as_slice()),
            };
            ui.set_playlist_list(adapter);
            ui.set_page(Page::PlaylistList);
        }).unwrap();
    });
}
