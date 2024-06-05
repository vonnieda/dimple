use dimple_core::model::{Artist, Entity, KnownIds};
use dimple_fanart_tv_plugin::FanartTvPlugin;
use dimple_mediafiles_plugin::MediaFilesPlugin;
use dimple_musicbrainz_plugin::MusicBrainzPlugin;
use dimple_player::player::Player;
use dimple_wikidata_plugin::WikidataPlugin;

use std::{borrow::BorrowMut, collections::VecDeque, path::PathBuf, sync::{Arc, Mutex}};

use dimple_core::db::Db;

use slint::{ComponentHandle, SharedString, Weak};

use dimple_librarian::librarian::Librarian;

use directories::ProjectDirs;

use crate::ui::{*};

use self::{images::ImageMangler, pages::settings};

#[derive(Clone)]
pub struct App {
    pub librarian: Librarian,
    pub history: Arc<Mutex<VecDeque<String>>>,
    pub player: Player,
    pub images: ImageMangler,
    pub ui: Weak<AppWindow>,
}

pub struct AppWindowController {
    ui: AppWindow,
    app: App,
}

impl AppWindowController {
    pub fn new() -> Self {
        let ui = AppWindow::new().unwrap();
        // TODO This and librarian should happen once the UI is up so that we
        // can show errors if needed. 
        let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
        let dir = dirs.data_dir().to_str().unwrap();
        let librarian = Librarian::new(dir);
        
        // let mediafiles_plugin = MediaFilesPlugin::new();
        // mediafiles_plugin.monitor_directory(&PathBuf::from("/Users/jason/Music"));
        // librarian.add_plugin(Box::new(mediafiles_plugin));

        librarian.add_plugin(Box::new(MusicBrainzPlugin::default()));
        librarian.add_plugin(Box::new(WikidataPlugin::default()));
        librarian.add_plugin(Box::new(FanartTvPlugin::default()));

        // librarian.get(&Artist {
        //     known_ids: KnownIds {
        //         musicbrainz_id: Some("65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab".to_string()),
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // }.model()).unwrap();
        // librarian.get(&Artist {
        //     known_ids: KnownIds {
        //         musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // }.model()).unwrap();

        let player = Player::new(Arc::new(librarian.clone()));
        let ui_weak = ui.as_weak();
        Self {
            ui,
            app: App {
                librarian: librarian.clone(),
                history: Arc::new(Mutex::new(VecDeque::new())),
                player,
                images: ImageMangler::new(librarian.clone(), ui_weak.clone()),
                ui: ui_weak,
            }
        }
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let app = self.app.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| app.navigate(url));

        // TODO I think this stuff moves into init on settings or something,
        // and we use that pattern for each page?
        let app = self.app.clone();
        self.ui.global::<AppState>().on_settings_generate_artists(
            move || settings::settings_generate_artists(&app));

        let app = self.app.clone();
        self.ui.global::<AppState>().on_settings_reset_database(
            move || settings::settings_reset_database(&app));
    
        let app = self.app.clone();
        self.ui.global::<AppState>().on_settings_set_online(
            move |online| settings::settings_set_online(&app, online));

        let app = self.app.clone();
        self.ui.global::<AppState>().on_settings_set_debug(
            move |debug| settings::settings_set_debug(&app, debug));

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }
}

impl App {
    pub fn navigate(&self, url: SharedString) {
        log::info!("{}", &url);
        if url.starts_with("http") {
            let _ = opener::open_browser(url.to_string());
        }
        else if url == "dimple://back" {
            self.back();
        }
        else if url == "dimple://refresh" {
            self.refresh();
        }
        else if url.starts_with("dimple://search") {
            crate::ui::pages::search::search(&url, self);
        }
        else if url.starts_with("dimple://home") {
            // TODO
            self.set_page(Page::Home);
        } 
        else if url.starts_with("dimple://artists") {
            crate::ui::pages::artist_list::artist_list(self);
        }
        else if url.starts_with("dimple://artist/") {
            crate::ui::pages::artist_details::artist_details(&url, self);
        }
        else if url.starts_with("dimple://release-groups") {
            crate::ui::pages::release_group_list::release_group_list(self);
        }
        else if url.starts_with("dimple://release-group/") {
            crate::ui::pages::release_group_details::release_group_details(&url, self);
        }
        else if url.starts_with("dimple://releases") {
            crate::ui::pages::release_list::release_list(self);
        }
        else if url.starts_with("dimple://release/") {
            crate::ui::pages::release_details::release_details(&url, self);
        }
        else if url.starts_with("dimple://tracks") {
            crate::ui::pages::track_list::track_list(self);
        }
        else if url.starts_with("dimple://track/") {
            // TODO
            self.set_page(Page::TrackDetails);
        }
        else if url.starts_with("dimple://genres") {
            crate::ui::pages::genre_list::genre_list(self);
        }
        else if url.starts_with("dimple://genre/") {
            crate::ui::pages::genre_details::genre_details(&url, self);
        }
        else if url.starts_with("dimple://playlists") {
            crate::ui::pages::playlist_list::playlist_list(self);
        }
        else if url == "dimple://settings" {
            crate::ui::pages::settings::settings(self);
        }

        // Store history.
        if url != "dimple://back" && url != "dimple://refresh" {
            self.history.lock().unwrap().push_back(url.into());
        }
    }

    pub fn back(&self) {
        let app = self.clone();
        self.ui.upgrade_in_event_loop(move |ui| {
            let url: Option<String> = app.history.lock().ok()
                .and_then(|mut history| {
                    let _ = history.pop_back()?;
                    history.pop_back()
                });
            if let Some(url) = url {
                app.navigate(url.into());
            }
        }).unwrap();
    }

    pub fn refresh(&self) {
        let app = self.clone();
        self.ui.upgrade_in_event_loop(move |ui| {
            let url: Option<String> = app.history.lock().ok()
                .and_then(|mut history| {
                    history.pop_back()
                });
            if let Some(url) = url {
                app.navigate(url.into());
            }
        }).unwrap();
    }    

    pub fn set_page(&self, page: Page) {
        self.ui.upgrade_in_event_loop(move |ui| {
            ui.set_page(page);
        }).unwrap();
    }
}