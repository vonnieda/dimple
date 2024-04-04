use dimple_player::player::Player;

use std::{collections::VecDeque, sync::{Arc, Mutex}, thread, time::Duration};

use dimple_core::source::AccessMode;

use slint::ComponentHandle;

use dimple_librarian::librarian::Librarian;

use directories::ProjectDirs;

use crate::ui::{*};

pub struct AppWindowController {
    ui: AppWindow,
    librarian: Arc<Librarian>,
    history: Arc<Mutex<VecDeque<String>>>,
    player: Player,
}

impl Default for AppWindowController {
    fn default() -> Self {
        let ui = AppWindow::new().unwrap();
        let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
        let dir = dirs.data_dir().to_str().unwrap();
        let librarian = Arc::new(Librarian::new(dir));
        let player = Player::new(librarian.clone());
        Self {
            ui,
            librarian,
            history: Arc::new(Mutex::new(VecDeque::new())),
            player,
        }
    }
}

impl AppWindowController {
    const THUMBNAIL_WIDTH: u32 = 200;
    const THUMBNAIL_HEIGHT: u32 = 200;

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        let history = self.history.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| 
            Self::navigate(url, history.clone(), &librarian.clone(), ui.clone()));

        // let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        self.ui.global::<AppState>().set_online(librarian.access_mode() == AccessMode::Online);
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

        let ui = self.ui.as_weak();
        let librarian = self.librarian.clone();
        // TODO moves to settings, or side bar, or wherever it's supposed to go.
        self.ui.global::<AppState>().on_set_online(move |online| {
            let librarian = librarian.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let librarian = librarian.clone();
                librarian.set_access_mode(if online { &AccessMode::Online } else { &AccessMode::Offline });
                ui.global::<AppState>().set_online(librarian.access_mode() == AccessMode::Online);
            }).unwrap();
        });

        // Updates player state
        // TODO moves to player_bar
        let ui = self.ui.as_weak();
        let player = self.player.clone();
        thread::spawn(move || {
            ui.upgrade_in_event_loop(move |ui| {
                fn length_to_string(length: u32) -> String {
                    format!("{}:{:02}", 
                        length / (60 * 1000), 
                        length % (60 * 1000) / 1000)
                }
                
                let adapter = PlayerBarAdapter {
                    duration_seconds: player.duration().as_secs() as i32,
                    duration_label: length_to_string(player.duration().as_secs() as u32).into(),
                    position_seconds: player.position().as_secs() as i32,
                    position_label: length_to_string(player.position().as_secs() as u32).into(),
                    ..Default::default()
                };
                ui.set_player_bar_adapter(adapter);
            }).unwrap();
            thread::sleep(Duration::from_millis(100));
        });
           
        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        self.ui.run()
    }

    fn navigate(url: slint::SharedString, history: Arc<Mutex<VecDeque<String>>>, 
        librarian: &Librarian, ui: slint::Weak<AppWindow>) {

        log::info!("{}", &url);
        if url.starts_with("http") {
            let _ = opener::open_browser(url.to_string());
        }
        else if url.starts_with("dimple://search") {
            Self::search(&url, librarian, ui);
        }
        else if url == "dimple://back" {
            Self::back(history.clone(), librarian, ui);
        }
        else if url == "dimple://refresh" {
            Self::refresh(history.clone(), librarian, ui);
        }
        // else if url.starts_with("dimple://home") {
        //     Self::home(librarian, ui);
        // } 
        // else if url == "dimple://artists" {
        //     Self::artists(librarian, ui);
        // }
        // else if url.starts_with("dimple://artist/") {
        //     Self::artist_details(&url, librarian, ui);
        // }
        // else if url.starts_with("dimple://release-group/") {
        //     Self::release_group_details(&url, librarian, ui);
        // }
        // else if url.starts_with("dimple://release/") {
        //     Self::release_details(&url, librarian, ui);
        // }
        // else if url.starts_with("dimple://recording/") {
        //     Self::recording_details(&url, librarian, ui);
        // }
        // else if url == "dimple://settings" {
        //     Self::settings(ui);
        // }

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

    fn search(url: &str, librarian: &Librarian, ui: slint::Weak<AppWindow>) {
        let url = url.to_owned();
        // std::thread::spawn(move || {
        //     ui.upgrade_in_event_loop(move |ui| {
        //         ui.global::<Navigator>().set_busy(true);
        //     }).unwrap();

        //     let url = Url::parse(&url).unwrap();
        //     let query = url.path_segments()
        //         // TODO is this pattern wrong? Shouldn't the or be an error?
        //         .ok_or("missing path").unwrap()
        //         .nth(0)
        //         .ok_or("missing query").unwrap();
        //     let search_results: Vec<_> = librarian.search(query).collect();
        //     // TODO woops, was sorting by name when they are returned by
        //     // relevance. Once more sources are merged I'll need to bring
        //     // rel to the front and sort on it.
        //     // search_results.sort_by_key(|e| e.name().to_lowercase());
        //     let cards = entity_cards(search_results, &librarian, 
        //         Self::THUMBNAIL_WIDTH, 
        //         Self::THUMBNAIL_WIDTH);
        //     ui.upgrade_in_event_loop(move |ui| {
        //         let adapter = CardGridAdapter {
        //             cards: card_adapters(cards),
        //         };
        //         ui.set_card_grid_adapter(adapter);
        //         ui.set_page(0);

        //         ui.global::<Navigator>().set_busy(false);
        //     }).unwrap();
        // });
    }
}

