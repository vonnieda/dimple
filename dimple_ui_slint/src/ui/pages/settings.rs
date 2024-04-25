use std::thread;

use dimple_core::db::Db;
use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::Medium;
use dimple_core::model::Picture;
use dimple_core::model::Recording;
use dimple_core::model::RecordingSource;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use slint::{ModelRc, SharedString};

use crate::ui::app_window_controller::App;

use crate::ui::common;
use crate::ui::SettingsAdapter;
use crate::ui::Page;

pub fn settings(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        let db: &dyn Db = &app.librarian as &dyn Db;

        let mut database_stats: Vec<String> = vec![];
        database_stats.push(format!("Artists: {}", 
            db.list(&Artist::default().model(), None).unwrap().count()));
        database_stats.push(format!("Release Groups: {}", 
            db.list(&ReleaseGroup::default().model(), None).unwrap().count()));
        database_stats.push(format!("Releases: {}", 
            db.list(&Release::default().model(), None).unwrap().count()));
        database_stats.push(format!("Media: {}", 
            db.list(&Medium::default().model(), None).unwrap().count()));
        database_stats.push(format!("Tracks: {}", 
            db.list(&Track::default().model(), None).unwrap().count()));
        database_stats.push(format!("Recordings: {}", 
            db.list(&Recording::default().model(), None).unwrap().count()));
        database_stats.push(format!("Recording Sources: {}", 
            db.list(&RecordingSource::default().model(), None).unwrap().count()));
        database_stats.push(format!("Genres: {}", 
            db.list(&Genre::default().model(), None).unwrap().count()));
        database_stats.push(format!("Pictures: {}", 
                db.list(&Picture::default().model(), None).unwrap().count()));

        let mut cache_stats = vec![];
        cache_stats.push(format!("Thumbnails: {}", app.images.cache_len()));
        
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

pub fn settings_generate_50_artists(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        log::info!("Creating some random data.");
        common::create_random_data(&app.librarian, 50);
        log::info!("Done creating some random data.");
    });
}

pub fn settings_generate_reset_database(app: &App) {
    let app = app.clone();
    thread::spawn(move || {
        log::info!("Resetting database.");
        app.librarian.reset();
        log::info!("Done resetting database.");
    });
}

// let ui = self.ui.as_weak();
// let librarian = self.librarian.clone();
// // TODO moves to settings, or side bar, or wherever it's supposed to go.
// self.ui.global::<AppState>().on_set_online(move |online| {
//     let librarian = librarian.clone();
//     ui.upgrade_in_event_loop(move |ui| {
//         let librarian = librarian.clone();
//         librarian.set_access_mode(if online { &AccessMode::Online } else { &AccessMode::Offline });
//         ui.global::<AppState>().set_online(librarian.access_mode() == AccessMode::Online);
//     }).unwrap();
// });

