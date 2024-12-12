use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::CardGridAdapter;
use crate::ui::Page;
use dimple_core::model::Playlist;
use slint::Model;
use slint::ModelRc;
use crate::ui::PlaylistListAdapter;
use slint::ComponentHandle;

pub fn playlist_list_init(_app: &App) {
    let app = _app.clone();
    _app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<PlaylistListAdapter>().on_new_playlist(move || new_playlist(&app));
    }).unwrap();
}

pub fn playlist_list(app: &App) {
    let ui = app.ui.clone();
    let images = app.images.clone();
    let app = app.clone();
    std::thread::spawn(move || {
        let playlists: Vec<Playlist> = app.library
            .query("SELECT * FROM Playlist ORDER BY name ASC", ());
        ui.upgrade_in_event_loop(move |ui| {
            let cards: Vec<CardAdapter> = playlists.iter().cloned().enumerate()
                .map(|(index, playlist)| {
                    let mut card: CardAdapter = playlist.clone().into();
                    card.image.image = images.lazy_get(playlist, 200, 200, move |ui, image| {
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

fn new_playlist(_app: &App) {
    let playlist = _app.library.save(&Playlist {
        name: Some("New Playlist".to_string()),
        ..Default::default()
    });
    let app = _app.clone();
    _app.ui.upgrade_in_event_loop(move |_ui| {
        app.navigate(format!("dimple://playlist/{}", playlist.key.unwrap()).into());
    }).unwrap();
}