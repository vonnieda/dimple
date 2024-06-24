use dimple_core::model::{Artist, Entity, KnownIds, Model};
use dimple_coverartarchive_plugin::CoverArtArchivePlugin;
use dimple_fanart_tv_plugin::FanartTvPlugin;
use dimple_mediafiles_plugin::MediaFilesPlugin;
use dimple_musicbrainz_plugin::MusicBrainzPlugin;
use dimple_player::player::Player;
use dimple_theaudiodb_plugin::TheAudioDbPlugin;
use dimple_wikidata_plugin::WikidataPlugin;
use pages::release_group_details::{self, release_group_details};
use serde::de;

use std::{borrow::BorrowMut, collections::VecDeque, path::PathBuf, sync::{Arc, Mutex}};

use dimple_core::db::Db;

use slint::{ComponentHandle, Model as _, ModelRc, SharedString, Weak};

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
        // let librarian = Librarian::new_in_memory();
        let librarian = Librarian::new(dir);
        
        // let mediafiles_plugin = MediaFilesPlugin::new();
        // mediafiles_plugin.monitor_directory(&PathBuf::from("/Users/jason/Music"));
        // librarian.add_plugin(Box::new(mediafiles_plugin));

        librarian.add_plugin(Box::new(MusicBrainzPlugin::default()));
        librarian.add_plugin(Box::new(WikidataPlugin::default()));
        librarian.add_plugin(Box::new(FanartTvPlugin::default()));
        librarian.add_plugin(Box::new(TheAudioDbPlugin::default()));
        librarian.add_plugin(Box::new(CoverArtArchivePlugin::default()));

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
        self.ui.global::<AppState>().on_settings_reset_database(
            move || settings::settings_reset_database(&app));
    
        let app = self.app.clone();
        self.ui.global::<AppState>().on_settings_set_online(
            move |online| settings::settings_set_online(&app, online));

        let app = self.app.clone();
        self.ui.global::<AppState>().on_settings_set_debug(
            move |debug| settings::settings_set_debug(&app, debug));
    
        let app = self.app.clone();
        self.ui.global::<AppState>().on_release_group_details_release_selected(
            move |s| release_group_details::release_group_details_release_selected(&app, s.to_string()));
        
                    // Load the sidebar
        let app = self.app.clone();
        std::thread::spawn(move || {
            let mut pinned_items: Vec<Model> = vec![];
            pinned_items.push(app.librarian.get2(Artist {
                known_ids: KnownIds {
                    musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }).unwrap().model());
            pinned_items.push(app.librarian.get2(Artist {
                known_ids: KnownIds {
                    musicbrainz_id: Some("65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }).unwrap().model());
            pinned_items.push(app.librarian.get2(Artist {
                known_ids: KnownIds {
                    musicbrainz_id: Some("c14b4180-dc87-481e-b17a-64e4150f90f6".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }).unwrap().model());
            pinned_items.push(app.librarian.get2(Artist {
                known_ids: KnownIds {
                    musicbrainz_id: Some("69158f97-4c07-4c4e-baf8-4e4ab1ed666e".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }).unwrap().model());
            pinned_items.push(app.librarian.get2(Artist {
                known_ids: KnownIds {
                    musicbrainz_id: Some("f1686ac4-3f28-4789-88eb-083ccb3a213a".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }).unwrap().model());
            let images = app.images.clone();
            app.ui.upgrade_in_event_loop(move |ui| {
                let cards: Vec<CardAdapter> = pinned_items.iter().cloned().enumerate()
                    .map(|(index, model)| {
                        let mut card: CardAdapter = model_card(&model);
                        card.image.image = images.lazy_get(model, 48, 48, move |ui, image| {
                            let mut card = ui.get_sidebar().pinned_items.row_data(index).unwrap();
                            card.image.image = image;
                            ui.get_sidebar().pinned_items.set_row_data(index, card);
                        });
                        card
                    })
                    .collect();
                let adapter = SideBarAdapter {
                    pinned_items: ModelRc::from(cards.as_slice()),
                };
                ui.set_sidebar(adapter);
            }).unwrap();
        });

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }
}

fn model_card(model: &Model) -> CardAdapter {
    match model {
        Model::Artist(artist) => artist_card(artist),
        // Model::ReleaseGroup(release_group) => release_group_card(release_group),
        // Model::Genre(genre) => genre_card(genre),
        // Model::Recording(recording) => recording_card(recording),
        _ => todo!(),
    }
}

fn artist_card(artist: &Artist) -> CardAdapter {
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: "Artist".to_string().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
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
        else if url.starts_with("dimple://recording/") {
            crate::ui::pages::recording_details::recording_details(&url, self);
        }
        else if url.starts_with("dimple://tracks") {
            crate::ui::pages::track_list::track_list(self);
        }
        else if url.starts_with("dimple://track/") {
            crate::ui::pages::track_details::track_details(&url, self);
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
        // TODO magic
        if url != "dimple://back" && url != "dimple://refresh" && !url.starts_with("http") {
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