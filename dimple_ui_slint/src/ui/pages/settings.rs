use std::thread;

use anyhow::Result;
use dimple_core::plugins;
use dimple_db::db::query::QuerySubscription;
use dimple_db::{sync::SyncEngine};
use serde::{Deserialize, Serialize};
use slint::{ModelRc, SharedString, ComponentHandle};

use crate::config::ConfigValue;
use crate::ui::app_window_controller::App;
use crate::ui::SettingsAdapter;
use crate::ui::Page;

pub struct SettingsController {
    _stats_subscription: QuerySubscription,
    _config_subscription: QuerySubscription,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EntityCounts {
    artist_count: i64,
    release_count: i64,
    track_count: i64,
    track_source_count: i64,
    playlist_count: i64,
    genre_count: i64,
    dimage_count: i64,
    link_count: i64,
    change_count: i64,
}

impl SettingsController {
    pub fn new(app: &App) -> Result<Self> {
    // Subscribe to entity count updates - using a single combined query for efficiency
        let ui = app.ui.clone();
        let stats_subscription = app.library.db.query_subscribe(
        "SELECT
            (SELECT COUNT(*) FROM Artist) as artist_count,
            (SELECT COUNT(*) FROM Release) as release_count,
            (SELECT COUNT(*) FROM Track) as track_count,
            (SELECT COUNT(*) FROM TrackSource) as track_source_count,
            (SELECT COUNT(*) FROM Playlist) as playlist_count,
            (SELECT COUNT(*) FROM Genre) as genre_count,
            (SELECT COUNT(*) FROM Dimage) as dimage_count,
            (SELECT COUNT(*) FROM Link) as link_count,
            (SELECT COUNT(*) FROM ZV_CHANGE) as change_count
            ",
        (),
        move |counts: Vec<EntityCounts>| {
            if let Some(count_data) = counts.first() {
                let count_data = count_data.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    let stats = vec![
                        format!("Artists: {}", count_data.artist_count),
                        format!("Releases: {}", count_data.release_count),
                        format!("Tracks: {}", count_data.track_count),
                        format!("TrackSources: {}", count_data.track_source_count),
                        format!("Playlists: {}", count_data.playlist_count),
                        format!("Genres: {}", count_data.genre_count),
                        format!("Dimages: {}", count_data.dimage_count),
                        format!("Links: {}", count_data.link_count),
                        format!("Changes: {}", count_data.change_count),
                    ];
                    let stats: Vec<SharedString> = stats.into_iter().map(Into::into).collect();
                    ui.global::<SettingsAdapter>().set_database_stats(ModelRc::from(stats.as_slice()));
                }).unwrap();
            }
        },
    )?;

        // Subscribe to config value updates
        let ui = app.ui.clone();
        let config_subscription = app.config.db.query_subscribe::<ConfigValue, _, _>(
        "SELECT id, key, value FROM ConfigValue",
        (),
        move |rows| {
            ui.upgrade_in_event_loop(move |ui| {
                for config_value in rows {
                    match config_value.key.as_str() {
                        "offline" => ui.global::<SettingsAdapter>().set_offline(config_value.value == Some("true".to_string())),
                        "debug" => ui.global::<SettingsAdapter>().set_debug(config_value.value == Some("true".to_string())),
                        "plugins_enabled" => ui.global::<SettingsAdapter>().set_plugins_enabled(config_value.value == Some("true".to_string())),
                        "sidebar_open" => ui.global::<SettingsAdapter>().set_sidebar_open(config_value.value == Some("true".to_string())),
                        "preferred_language" => {
                            let lang = config_value.value.unwrap_or_default();
                            // Find the index in the language options
                            let language_options = vec!["", "en", "es", "fr", "de", "it", "pt", "ru", "ja", "zh", "ko", "nl", "sv", "no", "da", "fi", "pl"];
                            let index = language_options.iter().position(|&l| l == lang).unwrap_or(0) as i32;
                            ui.global::<SettingsAdapter>().set_preferred_language_index(index);
                            ui.global::<SettingsAdapter>().set_preferred_language(lang.into());
                        },
                        "s3_endpoint" => ui.global::<SettingsAdapter>().set_s3_endpoint(config_value.value.unwrap_or_default().into()),
                        "s3_region" => ui.global::<SettingsAdapter>().set_s3_region(config_value.value.unwrap_or_default().into()),
                        "s3_bucket" => ui.global::<SettingsAdapter>().set_s3_bucket(config_value.value.unwrap_or_default().into()),
                        "s3_access_key" => ui.global::<SettingsAdapter>().set_s3_access_key(config_value.value.unwrap_or_default().into()),
                        "s3_secret_key" => ui.global::<SettingsAdapter>().set_s3_secret_key(config_value.value.unwrap_or_default().into()),
                        "s3_prefix" => ui.global::<SettingsAdapter>().set_s3_prefix(config_value.value.unwrap_or_default().into()),
                        _ => {}
                    }
                }
            }).unwrap();
        },
    )?;

        // Set up UI callbacks
        let app_ = app.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_.clone();
            ui.global::<SettingsAdapter>().on_set_sidebar_open(move |v| app.config.set_sidebar_open(v));

            let app = app_.clone();
            ui.global::<SettingsAdapter>().set_sidebar_open(app.config.sidebar_open());

            let app = app_.clone();
            ui.global::<SettingsAdapter>().on_set_offline(move |v| app.config.set_offline(v));

            let app = app_.clone();
            ui.global::<SettingsAdapter>().on_set_plugins_enabled(move |v| app.config.set_plugins_enabled(v));

            let app = app_.clone();
            ui.global::<SettingsAdapter>().on_set_debug(move |v| app.config.set_debug(v));

            let app = app_.clone();
            ui.global::<SettingsAdapter>().on_set_preferred_language(move |v| {
                // Extract ISO code from the dropdown value (e.g., "en - English" -> "en")
                let value = v.to_string();
                let iso_code = if value.is_empty() {
                    None
                } else if value.contains(" - ") {
                    // Extract the first 2 characters (ISO code)
                    Some(value.chars().take(2).collect())
                } else {
                    // Already an ISO code
                    Some(value)
                };
                app.config.set_preferred_language(iso_code);
            });

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
        
        Ok(Self {
            _stats_subscription: stats_subscription,
            _config_subscription: config_subscription,
        })
    }
}

pub fn settings_init(app: &App) -> Result<SettingsController> {
    SettingsController::new(app)
}

pub fn settings(app: &App) {
    // Navigate to settings page
    app.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::Settings)).unwrap();
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