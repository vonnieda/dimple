use dimple_player::player::Player;

use std::{collections::VecDeque, sync::{Arc, Mutex}};

use dimple_core::{db::Db, model::{Artist, Medium, Recording, RecordingSource, Release, ReleaseGroup, Track}};

use slint::{ComponentHandle, SharedString, Weak};

use dimple_librarian::librarian::Librarian;

use directories::ProjectDirs;

use crate::ui::{*};

pub struct AppWindowController {
    ui: AppWindow,
    librarian: Librarian,
    history: Arc<Mutex<VecDeque<String>>>,
    player: Player,
}

impl Default for AppWindowController {
    fn default() -> Self {
        let ui = AppWindow::new().unwrap();
        // TODO Probably this and librarian happens once the UI is up so that we
        // can show errors if needed. 
        let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
        let dir = dirs.data_dir().to_str().unwrap();
        let librarian = Librarian::new(dir);
        let player = Player::new(Arc::new(librarian.clone()));
        Self {
            ui,
            librarian,
            history: Arc::new(Mutex::new(VecDeque::new())),
            player,
        }
    }
}

impl AppWindowController {
    pub fn run(&self) -> Result<(), slint::PlatformError> {
        {
            let ui = self.ui.as_weak();
            let librarian = self.librarian.clone();
            let history = self.history.clone();
            self.ui.global::<Navigator>().on_navigate(move |url| 
                Self::navigate(url, history.clone(), &librarian.clone(), ui.clone()));
        }

        if self.librarian.list(&Artist::default().into(), None).unwrap().count() == 0 {
            log::info!("Creating some random data.");
            Self::create_random_data(&self.librarian, 1000);
            log::info!("Done.");
        }

        // let ui = self.ui.as_weak();
        // let librarian = self.librarian.clone();
        // self.ui.global::<AppState>().set_online(librarian.access_mode() == AccessMode::Online);
        // let paths = vec![
        //     "/Users/jason/Music/My Music".to_string(),
        // ];
        // self.librarian.add_library(Box::new(FileLibrary::new(&paths)));
        // self.librarian.add_library(Box::<MusicBrainzLibrary>::default());
        // self.librarian.add_library(Box::<TheAudioDbLibrary>::default());
        // self.librarian.add_library(Box::<FanartTvLibrary>::default());
        // self.librarian.add_library(Box::<DeezerLibrary>::default());
        // self.librarian.add_library(Box::<WikidataLibrary>::default());
        // self.librarian.add_library(Box::<LastFmLibrary>::default());
        // self.librarian.add_library(Box::<CoverArtArchiveLibrary>::default());

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }

    pub fn set_page(ui: Weak<AppWindow>, page: Page) {
        ui.upgrade_in_event_loop(move |ui| {
            ui.set_page(page);
        }).unwrap();
    }

    fn navigate(url: SharedString, history: Arc<Mutex<VecDeque<String>>>, 
        librarian: &Librarian, ui: Weak<AppWindow>) {

        log::info!("{}", &url);
        if url.starts_with("http") {
            let _ = opener::open_browser(url.to_string());
        }
        else if url == "dimple://back" {
            Self::back(history.clone(), librarian, ui);
        }
        else if url == "dimple://refresh" {
            Self::refresh(history.clone(), librarian, ui);
        }
        else if url.starts_with("dimple://search") {
            Self::set_page(ui, Page::Search);
        }
        else if url.starts_with("dimple://home") {
            Self::set_page(ui, Page::Home);
        } 
        else if url.starts_with("dimple://artists") {
            crate::ui::pages::artist_list::artist_list(librarian, ui);
        }
        else if url.starts_with("dimple://artist/") {
            crate::ui::pages::artist_details::artist_details(&url, librarian, ui);
        }
        else if url.starts_with("dimple://release-groups") {
            Self::set_page(ui, Page::ReleaseGroupList);
        }
        else if url.starts_with("dimple://release-group/") {
            Self::set_page(ui, Page::ReleaseGroupDetails);
        }
        else if url.starts_with("dimple://releases") {
            Self::set_page(ui, Page::ReleaseList);
        }
        else if url.starts_with("dimple://release/") {
            Self::set_page(ui, Page::ReleaseDetails);
        }
        else if url.starts_with("dimple://tracks") {
            crate::ui::pages::track_list::track_list(librarian, ui);
        }
        else if url.starts_with("dimple://track/") {
            Self::set_page(ui, Page::TrackDetails);
        }
        else if url == "dimple://settings" {
            Self::set_page(ui, Page::Settings);
        }

        // Store history.
        if url != "dimple://back" && url != "dimple://refresh" {
            history.lock().unwrap().push_back(url.into());
        }
    }

    fn back(history: Arc<Mutex<VecDeque<String>>>, 
        librarian: &Librarian, ui: slint::Weak<AppWindow>) {
            let librarian = librarian.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let url: Option<String> = history.lock().ok()
                    .and_then(|mut history| {
                        let _ = history.pop_back()?;
                        history.pop_back()
                    });
                if let Some(url) = url {
                    Self::navigate(url.into(), history.clone(), &librarian, ui.as_weak());
                }
            }).unwrap();
    }

    fn refresh(history: Arc<Mutex<VecDeque<String>>>, 
        librarian: &Librarian, ui: slint::Weak<AppWindow>) {
            let librarian = librarian.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let url: Option<String> = history.lock().ok()
                    .and_then(|mut history| {
                        history.pop_back()
                    });
                if let Some(url) = url {
                    Self::navigate(url.into(), history.clone(), &librarian, ui.as_weak());
                }
            }).unwrap();
    }

    fn create_random_data(db: &dyn Db, num_artists: u32) {
        for i in 0..num_artists {
            let artist = Artist {
                name: Some(fakeit::name::full()),
                summary: Some(fakeit::hipster::paragraph(1, 4, 40, " ".to_string())),
                country: Some(fakeit::address::country_abr()),
                disambiguation: Some(fakeit::address::country()),
                ..Default::default()
            };
            let release_group = ReleaseGroup {
                title: Some(fakeit::hipster::sentence(2)),
                ..Default::default()
            };
            let release = Release {
                title: release_group.title.clone(),
                ..Default::default()
            };
            let medium = Medium {
                disc_count: Some(1),
                track_count: Some(1),
                ..Default::default()
            };
            let track = Track {
                title: Some(fakeit::hipster::sentence(3)),
                ..Default::default()
            };
            let recording = Recording {
                title: track.title.clone(),
                ..Default::default()
            };
            let recording_source = RecordingSource {
                ..Default::default()
            };
            let artist = db.insert(&artist.clone().into()).unwrap();
            let release_group = db.insert(&release_group.clone().into()).unwrap();
            let release = db.insert(&release.clone().into()).unwrap();
            let medium = db.insert(&medium.clone().into()).unwrap();
            let track = db.insert(&track.clone().into()).unwrap();
            let recording = db.insert(&recording.clone().into()).unwrap();
            let recording_source = db.insert(&recording_source.clone().into()).unwrap();
            db.link(&release_group.clone().into(), &artist.clone().into()).unwrap();
            db.link(&release.clone().into(), &release_group.clone().into()).unwrap();
            db.link(&medium.clone().into(), &release.clone().into()).unwrap();
            db.link(&track.clone().into(), &medium.clone().into()).unwrap();
            db.link(&recording.clone().into(), &track.clone().into()).unwrap();
            db.link(&recording_source.clone().into(), &recording.clone().into()).unwrap();
        }
    }
}

