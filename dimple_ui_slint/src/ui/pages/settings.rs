use std::thread;

use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::MediaFile;
use dimple_core::model::Playlist;
use dimple_core::model::Track;
use dimple_core::model::TrackSource;
use size::Size;
use slint::{ModelRc, SharedString};

use crate::config::PluginConfig;
use crate::ui::app_window_controller::App;

use crate::ui::SettingsAdapter;
use crate::ui::Page;
use crate::ui::AppState;
use crate::ui::Styles;
use crate::ui::PluginAdapter;

use slint::ComponentHandle;

pub fn settings_init(app: &App) {
    let app_ = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_online(
            move |online| set_online(&app, online));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_debug(
            move |debug| set_debug(&app, debug));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_set_font_size(
            move |font_size| set_font_size(&app, font_size));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_import_files(
            move || import_files(&app));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_import_directories(
            move || import_directories(&app));

        let app = app_.clone();
        ui.global::<SettingsAdapter>().on_quit(move || {
            slint::quit_event_loop().unwrap();
        });
    }).unwrap();
}

pub fn settings(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        let db = app.library.clone();

        let plugins = app.config.plugin_config();

        let mut database_stats: Vec<String> = vec![];
        database_stats.push(format!("Artists: {}", db.list::<Artist>().len()));
        database_stats.push(format!("Genres: {}", db.list::<Genre>().len()));
        database_stats.push(format!("MediaFiles: {}", db.list::<MediaFile>().len()));
        database_stats.push(format!("Playlists: {}", db.list::<Playlist>().len()));
        database_stats.push(format!("Tracks: {}", db.list::<Track>().len()));
        database_stats.push(format!("TrackSources: {}", db.list::<TrackSource>().len()));

        let mut cache_stats: Vec<String> = vec![];
        // TODO Before any music has been loaded, there are no images, so the
        // cache is empty, and this blows up. 
        // cache_stats.push(format!("Thumbnail cache: {}", Size::from_bytes(app.images.cache_len())));
        // cache_stats.push(format!("Plugin cache: {}", Size::from_bytes(app.librarian.plugin_cache_len())));
        
        app.ui.upgrade_in_event_loop(move |ui| {
            let database_stats: Vec<SharedString> = database_stats.into_iter()
                .map(Into::into)
                .collect();
            let cache_stats: Vec<SharedString> = cache_stats.into_iter()
                .map(Into::into)
                .collect();
            let plugins: Vec<PluginAdapter> = plugins.into_iter()
                .map(plugin_adapter)
                .collect();
            ui.global::<SettingsAdapter>().set_database_stats(ModelRc::from(database_stats.as_slice()));
            ui.global::<SettingsAdapter>().set_cache_stats(ModelRc::from(cache_stats.as_slice()));
            ui.global::<SettingsAdapter>().set_plugins(plugins.as_slice().into());
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

fn set_online(app: &App, online: bool) {
    let app = app.clone();
    // app.ui.upgrade_in_event_loop(move |ui| {
    //     app.librarian.set_network_mode(if online { &NetworkMode::Online } else { &NetworkMode::Offline });
    //     ui.global::<AppState>().set_online(app.librarian.network_mode() == NetworkMode::Online);
    // }).unwrap();
}

fn set_debug(app: &App, debug: bool) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<AppState>().set_debug(debug);
    }).unwrap();
}

fn set_font_size(app: &App, font_size: f32) {
    let app = app.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.global::<Styles>().set_default_font_size(font_size);
    }).unwrap();
}

fn plugin_adapter(plugin_config: PluginConfig) -> PluginAdapter{
    PluginAdapter {
        title: plugin_config.type_name.into(),
        sub_title: plugin_config.config.into(),
        enabled: plugin_config.enabled,
        status: Default::default(),
    }
}