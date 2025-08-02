use std::thread;

use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::MediaFile;
use dimple_core::model::Playlist;
use dimple_core::model::Track;
use dimple_core::model::TrackSource;
use dimple_core::plugins;
use dimple_db::sync::SyncEngine;
use slint::{ModelRc, SharedString};

use crate::ui::app_window_controller::App;

use crate::ui::SettingsAdapter;
use crate::ui::Page;


use slint::ComponentHandle;

pub fn settings_init(app: &App) {
    let app_ = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_sidebar_open(move |v| {
            app.config.set_sidebar_open(v);
            // TODO note, we set this here but not in the others below because
            // this changes app state that other pages need to know about. I
            // think it goes away when this page is just reacting to database
            // changes.
            app.ui.upgrade_in_event_loop(move |ui| {
                ui.global::<SettingsAdapter>().set_sidebar_open(v);
            }).unwrap();
        });
        let app = app_.clone();
        ui.global::<SettingsAdapter>().set_sidebar_open(app.config.sidebar_open());

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_offline(move |v| app.config.set_offline(v));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_debug(move |v| app.config.set_debug(v));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_s3_endpoint(move |v| app.config.set_s3_endpoint(plugins::plugins::nempty(&v.to_string())));
        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_s3_region(move |v| app.config.set_s3_region(plugins::plugins::nempty(&v.to_string())));
        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_s3_bucket(move |v| app.config.set_s3_bucket(plugins::plugins::nempty(&v.to_string())));
        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_s3_access_key(move |v| app.config.set_s3_access_key(plugins::plugins::nempty(&v.to_string())));
        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_s3_secret_key(move |v| app.config.set_s3_secret_key(plugins::plugins::nempty(&v.to_string())));
        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_s3_prefix(move |v| app.config.set_s3_prefix(plugins::plugins::nempty(&v.to_string())));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_import_files(move || import_files(&app));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_import_directories(move || import_directories(&app));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_sync_now(move || sync_now(&app));

        ui.global::<SettingsAdapter>().on_quit(move || slint::quit_event_loop().unwrap());
    }).unwrap();
}

pub fn settings(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        let db = app.library.clone();

        // let plugins = app.config.plugin_config();

        let mut database_stats: Vec<String> = vec![];
        database_stats.push(format!("Artists: {}", db.list::<Artist>().len()));
        database_stats.push(format!("Genres: {}", db.list::<Genre>().len()));
        database_stats.push(format!("MediaFiles: {}", db.list::<MediaFile>().len()));
        database_stats.push(format!("Playlists: {}", db.list::<Playlist>().len()));
        database_stats.push(format!("Tracks: {}", db.list::<Track>().len()));
        database_stats.push(format!("TrackSources: {}", db.list::<TrackSource>().len()));

        let cache_stats: Vec<String> = vec![];
        // TODO Before any music has been loaded, there are no images, so the
        // cache is empty, and this blows up. 
        // cache_stats.push(format!("Thumbnail cache: {}", Size::from_bytes(app.images.cache_len())));
        // cache_stats.push(format!("Plugin cache: {}", Size::from_bytes(app.librarian.plugin_cache_len())));
        
        // TODO probably need to load these in from startup so UI is right, especially sidebar
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<SettingsAdapter>().set_offline(app.config.offline());
            ui.global::<SettingsAdapter>().set_debug(app.config.debug());
            ui.global::<SettingsAdapter>().set_s3_endpoint(app.config.s3_endpoint().unwrap_or_default().into());
            ui.global::<SettingsAdapter>().set_s3_region(app.config.s3_region().unwrap_or_default().into());
            ui.global::<SettingsAdapter>().set_s3_bucket(app.config.s3_bucket().unwrap_or_default().into());
            ui.global::<SettingsAdapter>().set_s3_access_key(app.config.s3_access_key().unwrap_or_default().into());
            ui.global::<SettingsAdapter>().set_s3_secret_key(app.config.s3_secret_key().unwrap_or_default().into());
            ui.global::<SettingsAdapter>().set_s3_prefix(app.config.s3_prefix().unwrap_or_default().into());
            let database_stats: Vec<SharedString> = database_stats.into_iter()
                .map(Into::into)
                .collect();
            let cache_stats: Vec<SharedString> = cache_stats.into_iter()
                .map(Into::into)
                .collect();
            // let plugins: Vec<PluginAdapter> = plugins.into_iter()
            //     .map(plugin_adapter)
            //     .collect();
            ui.global::<SettingsAdapter>().set_database_stats(ModelRc::from(database_stats.as_slice()));
            ui.global::<SettingsAdapter>().set_cache_stats(ModelRc::from(cache_stats.as_slice()));
            // ui.global::<SettingsAdapter>().set_plugins(plugins.as_slice().into());
            ui.set_page(Page::Settings);
        }).unwrap();
    });
}

fn import_files(app: &App) {
    use rfd::FileDialog;

    let files = FileDialog::new()
        // .add_filter("text", &["txt", "rs"])
        // .add_filter("rust", &["rs", "toml"])
        // .set_directory("/")
        .pick_files();

    let app = app.clone();
    thread::spawn(move || {
        if let Some(files) = files {
            for file in files.iter() {
                app.library.import(file.to_str().unwrap());
            }
        }
    });
}

fn import_directories(app: &App) {
    use rfd::FileDialog;

    let files = FileDialog::new()
        // .add_filter("text", &["txt", "rs"])
        // .add_filter("rust", &["rs", "toml"])
        // .set_directory("/")
        .pick_folders();

    // TODO just launching this off into the void for now, will eventually
    // change import to use plugin api and show status as we import.
    // Same with the one above.
    let app = app.clone();
    thread::spawn(move || {
        if let Some(files) = files {
            for file in files.iter() {
                app.library.import(file.to_str().unwrap());
            }
        }
    });
}

fn sync_now(app: &App) {
    let app_clone = app.clone();
    thread::spawn(move || {
        let app = app_clone;
        let sync_engine = SyncEngine::builder()
            .prefix(&app.config.s3_prefix().unwrap_or_default())
            .s3(&app.config.s3_endpoint().unwrap_or_default(), 
                &app.config.s3_bucket().unwrap_or_default(), 
                &app.config.s3_region().unwrap_or_default(),
                &app.config.s3_access_key().unwrap_or_default(), 
                &app.config.s3_secret_key().unwrap_or_default()).unwrap()
            .build()
            .unwrap();
        sync_engine.sync(&app.library.db).unwrap();
    });
}

// fn plugin_adapter(plugin_config: PluginConfig) -> PluginAdapter{
//     PluginAdapter {
//         title: plugin_config.type_name.into(),
//         sub_title: plugin_config.config.into(),
//         enabled: plugin_config.enabled,
//         status: Default::default(),
//     }
// }