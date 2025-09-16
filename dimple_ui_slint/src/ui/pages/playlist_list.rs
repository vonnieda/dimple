use anyhow::Result;
use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use dimple_core::model::Playlist;
use dimple_db::db::query::QuerySubscription;
use crate::ui::PlaylistListAdapter;
use slint::ComponentHandle;

pub struct PlaylistListController {
    _playlists_subscription: QuerySubscription,
}

impl PlaylistListController {
    pub fn new(app: &App) -> Result<Self> {
        // Subscribe to playlist changes
        let ui = app.ui.clone();
        let playlists_subscription = app.library.db.query_subscribe(
            "SELECT * FROM Playlist
            WHERE name NOT LIKE '__dimple_system_play_queue_%'
            AND save = TRUE
            ORDER BY name ASC",
            (),
            move |playlists: Vec<Playlist>| {
                ui.upgrade_in_event_loop(move |ui| {
                    let cards: Vec<CardAdapter> = playlists.iter().cloned().map(Into::into).collect();
                    ui.global::<PlaylistListAdapter>().set_cards(cards.as_slice().into());
                }).unwrap();
            },
        )?;

        // Set up UI callbacks
        let app_ = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_;
            ui.global::<PlaylistListAdapter>().on_new_playlist(move || new_playlist(&app));
        }).unwrap();

        Ok(Self {
            _playlists_subscription: playlists_subscription,
        })
    }
}

fn new_playlist(_app: &App) {
    let playlist = _app.library.save(&Playlist {
        name: Some("New Playlist".to_string()),
        ..Default::default()
    }).unwrap();
    let app = _app.clone();
    _app.ui.upgrade_in_event_loop(move |_ui| {
        app.navigate(format!("dimple://playlist/{}", playlist.id.unwrap()).into());
    }).unwrap();
}