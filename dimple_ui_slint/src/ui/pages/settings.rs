use std::thread;

use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Playlist;
use dimple_core::model::Release;
use dimple_core::model::Track;
use size::Size;
use slint::{ModelRc, SharedString};

use crate::ui::app_window_controller::App;

use crate::ui::common;
use crate::ui::SettingsAdapter;
use crate::ui::Page;
use crate::ui::AppState;

use slint::{ComponentHandle, Weak};

pub fn settings(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        // let db: &dyn Db = &app.librarian as &dyn Db;
        let db = app.librarian.clone();

        let mut database_stats: Vec<String> = vec![];
        // database_stats.push(format!("Artists: {}", 
        //     db.list(&Artist::default().model(), &None).unwrap().count()));
        // database_stats.push(format!("Release Groups: {}", 
        //     db.list(&ReleaseGroup::default().model(), &None).unwrap().count()));
        // database_stats.push(format!("Releases: {}", 
        //     db.list(&Release::default().model(), &None).unwrap().count()));
        // database_stats.push(format!("Media: {}", 
        //     db.list(&Medium::default().model(), &None).unwrap().count()));
        // database_stats.push(format!("Tracks: {}", 
        //     db.list(&Track::default().model(), &None).unwrap().count()));
        // database_stats.push(format!("Recordings: {}", 
        //     db.list(&Recording::default().model(), &None).unwrap().count()));
        // database_stats.push(format!("Recording Sources: {}", 
        //     db.list(&RecordingSource::default().model(), &None).unwrap().count()));
        // database_stats.push(format!("Genres: {}", 
        //     db.list(&Genre::default().model(), &None).unwrap().count()));
        // database_stats.push(format!("Playlists: {}", 
        //     db.list(&Playlist::default().model(), &None).unwrap().count()));
        // TODO disabled until performance is better
        // database_stats.push(format!("Pictures: {}", 
        //         db.list(&Picture::default().model(), &None).unwrap().count()));

        let mut cache_stats = vec![];
        cache_stats.push(format!("Thumbnail cache: {}", Size::from_bytes(app.images.cache_len())));
        // cache_stats.push(format!("Plugin cache: {}", Size::from_bytes(app.librarian.plugin_cache_len())));
        
        app.ui.upgrade_in_event_loop(move |ui| {
            let database_stats: Vec<SharedString> = database_stats.into_iter()
                .map(Into::into)
                .collect();
            let cache_stats: Vec<SharedString> = cache_stats.into_iter()
                .map(Into::into)
                .collect();
            let adapter = SettingsAdapter {
                database_stats: ModelRc::from(database_stats.as_slice()),
                cache_stats: ModelRc::from(cache_stats.as_slice()),
            };
            ui.set_settings(adapter);
            ui.set_page(Page::Settings);
        }).unwrap();
    });
}

pub fn settings_reset_database(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        log::info!("Resetting database.");
        app.librarian.reset().unwrap();
        log::info!("Done resetting database.");
    });
}

pub fn settings_set_online(app: &App, online: bool) {
    let app = app.clone();
    // app.ui.upgrade_in_event_loop(move |ui| {
    //     app.librarian.set_network_mode(if online { &NetworkMode::Online } else { &NetworkMode::Offline });
    //     ui.global::<AppState>().set_online(app.librarian.network_mode() == NetworkMode::Online);
    // }).unwrap();
}

pub fn settings_set_debug(app: &App, debug: bool) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<AppState>().set_debug(debug);
    }).unwrap();
}

