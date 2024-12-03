use dimple_core::{library::Library, model::{Artist, Model}, player::Player};
use pages::{playlist_details, track_list};
use player_bar;
use std::{collections::VecDeque, sync::{Arc, Mutex}, time::Duration};

use slint::{SharedString, Weak};

use directories::ProjectDirs;

use crate::ui::{*};

use self::{images::ImageMangler, pages::settings};

#[derive(Clone)]
pub struct App {
    pub library: Library,
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
        let data_dir = dirs.data_dir();
        let cache_dir = dirs.cache_dir();
        let config_dir = dirs.config_dir();
        let image_cache_dir = cache_dir.join("image_cache");
        let library_path = data_dir.join("library.db");
        dbg!(&data_dir, &cache_dir, &config_dir, &library_path, &image_cache_dir);
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&image_cache_dir).unwrap();

        let librarian = Library::open(library_path.to_str().unwrap());
        let images = ImageMangler::new(librarian.clone(), ui.as_weak().clone(), image_cache_dir.to_str().unwrap());        
        let player = Player::new(Arc::new(librarian.clone()));
        let ui_weak = ui.as_weak();
        Self {
            ui,
            app: App {
                library: librarian.clone(),
                history: Arc::new(Mutex::new(VecDeque::new())),
                player,
                images,
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
        self.ui.global::<AppState>().on_playlist_details_track_selected(
            move |row| playlist_details::playlist_details_track_selected(&app, row));
    
        let app = self.app.clone();
        self.ui.global::<AppState>().on_track_list_track_selected(
            move |row| track_list::track_list_track_selected(&app, row));
        
        player_bar::player_bar_init(&self.app);

        self.ui.global::<Navigator>().invoke_navigate("dimple://queue".into());

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
        // else if url.starts_with("dimple://search") {
        //     crate::ui::pages::search::search(&url, self);
        // }
        else if url.starts_with("dimple://home") {
            // TODO
            self.set_page(Page::Home);
        } 
        // else if url.starts_with("dimple://artists") {
        //     crate::ui::pages::artist_list::artist_list(self);
        // }
        // else if url.starts_with("dimple://artist/") {
        //     crate::ui::pages::artist_details::artist_details(&url, self);
        // }
        // else if url.starts_with("dimple://release-groups") {
        //     crate::ui::pages::release_group_list::release_group_list(self);
        // }
        // else if url.starts_with("dimple://release-group/") {
        //     crate::ui::pages::release_group_details::release_group_details(&url, self);
        // }
        // else if url.starts_with("dimple://releases") {
        //     crate::ui::pages::release_list::release_list(self);
        // }
        // else if url.starts_with("dimple://release/") {
        //     crate::ui::pages::release_details::release_details(&url, self);
        // }
        // else if url.starts_with("dimple://recording/") {
        //     crate::ui::pages::recording_details::recording_details(&url, self);
        // }
        else if url.starts_with("dimple://tracks") {
            crate::ui::pages::track_list::track_list(self);
        }
        else if url.starts_with("dimple://track/") {
            crate::ui::pages::track_details::track_details(&url, self);
        }
        // else if url.starts_with("dimple://genres") {
        //     crate::ui::pages::genre_list::genre_list(self);
        // }
        // else if url.starts_with("dimple://genre/") {
        //     crate::ui::pages::genre_details::genre_details(&url, self);
        // }
        // else if url.starts_with("dimple://playlists") {
        //     crate::ui::pages::playlist_list::playlist_list(self);
        // }
        else if url.starts_with("dimple://playlist/") {
            crate::ui::pages::playlist_details::playlist_details(&url, self);
        }
        else if url.starts_with("dimple://queue") {
            let play_queue = self.player.queue();
            self.navigate(format!("dimple://playlist/{}", &play_queue.key.unwrap()).into());
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

// fn model_card(model: impl Model) -> CardAdapter {
//     match model {
//         // Model::Artist(artist) => artist_card(artist),
//         // Model::ReleaseGroup(release_group) => release_group_card(release_group),
//         // Model::Genre(genre) => genre_card(genre),
//         // Model::Recording(recording) => recording_card(recording),
//         _ => todo!(),
//     }
// }

// pub fn artist_card(artist: &Artist) -> CardAdapter {
//     CardAdapter {
//         image: ImageLinkAdapter {
//             image: Default::default(),
//             name: artist.name.clone().unwrap_or_default().into(),
//             url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
//         },
//         title: LinkAdapter {
//             name: artist.name.clone().unwrap_or_default().into(),
//             url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
//         },
//         sub_title: LinkAdapter {
//             name: "Artist".to_string().into(),
//             url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
//         },
//     }
// }




     
        // let app = self.app.clone();
        // self.ui.global::<AppState>().on_release_group_details_release_selected(
        //     move |s| release_group_details::release_group_details_release_selected(&app, s.to_string()));
        
        // // Load the sidebar
        // let app = self.app.clone();
        // std::thread::spawn(move || {
        //     let mut pinned_items: Vec<Model> = vec![];
        //     pinned_items.push(app.librarian.get2(Artist {
        //         known_ids: KnownIds {
        //             musicbrainz_id: Some("73084492-3e59-4b7f-aa65-572a9d7691d5".to_string()),
        //             ..Default::default()
        //         },
        //         ..Default::default()
        //     }).unwrap().model());
        //     pinned_items.push(app.librarian.get2(Artist {
        //         known_ids: KnownIds {
        //             musicbrainz_id: Some("65f4f0c5-ef9e-490c-aee3-909e7ae6b2ab".to_string()),
        //             ..Default::default()
        //         },
        //         ..Default::default()
        //     }).unwrap().model());
        //     pinned_items.push(app.librarian.get2(Artist {
        //         known_ids: KnownIds {
        //             musicbrainz_id: Some("c14b4180-dc87-481e-b17a-64e4150f90f6".to_string()),
        //             ..Default::default()
        //         },
        //         ..Default::default()
        //     }).unwrap().model());
        //     pinned_items.push(app.librarian.get2(Artist {
        //         known_ids: KnownIds {
        //             musicbrainz_id: Some("69158f97-4c07-4c4e-baf8-4e4ab1ed666e".to_string()),
        //             ..Default::default()
        //         },
        //         ..Default::default()
        //     }).unwrap().model());
        //     pinned_items.push(app.librarian.get2(Artist {
        //         known_ids: KnownIds {
        //             musicbrainz_id: Some("f1686ac4-3f28-4789-88eb-083ccb3a213a".to_string()),
        //             ..Default::default()
        //         },
        //         ..Default::default()
        //     }).unwrap().model());
        //     let images = app.images.clone();
        //     app.ui.upgrade_in_event_loop(move |ui| {
        //         let cards: Vec<CardAdapter> = pinned_items.iter().cloned().enumerate()
        //             .map(|(index, model)| {
        //                 let mut card: CardAdapter = model_card(&model);
        //                 card.image.image = images.lazy_get(model, 48, 48, move |ui, image| {
        //                     let mut card = ui.get_sidebar().pinned_items.row_data(index).unwrap();
        //                     card.image.image = image;
        //                     ui.get_sidebar().pinned_items.set_row_data(index, card);
        //                 });
        //                 card
        //             })
        //             .collect();
        //         let adapter = SideBarAdapter {
        //             pinned_items: ModelRc::from(cards.as_slice()),
        //         };
        //         ui.set_sidebar(adapter);
        //     }).unwrap();
        // });



        // use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
        // desktop_integration();

        // // TODO desktop integration using souvlaki. currently broken on Windows.
        // fn desktop_integration() {
        //     #[cfg(not(target_os = "windows"))]
        //     let hwnd = None;
        
        //     #[cfg(target_os = "windows")]
        //     let hwnd = {
        //         use raw_window_handle::windows::WindowsHandle;
        
        //         let handle: WindowsHandle = unimplemented!();
        //         Some(handle.hwnd)
        //     };
        
        //     let config = PlatformConfig {
        //         dbus_name: "dimple",
        //         display_name: "Dimple",
        //         hwnd,
        //     };
        
        //     let mut controls = MediaControls::new(config).unwrap();
        
        //     // The closure must be Send and have a static lifetime.
        //     controls
        //         .attach(|event: MediaControlEvent| println!("Event received: {:?}", event))
        //         .unwrap();
        
        //     // Update the media metadata.
        //     controls
        //         .set_metadata(MediaMetadata {
        //             title: Some("Time to get Dimply"),
        //             artist: Some("Dimple"),
        //             album: Some("Dimple Time"),
        //             ..Default::default()
        //         })
        //         .unwrap();
        // }
        
        