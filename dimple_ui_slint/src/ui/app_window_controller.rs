use dimple_core::{library::Library, player::Player};
use pages::{event_list, playlist_details, queue_details, track_details, track_list};
use player_bar;
use std::{collections::VecDeque, sync::{Arc, Mutex}};

use slint::{SharedString, Weak};

use directories::ProjectDirs;

use crate::ui::{*};

use self::{images::ImageMangler, pages::settings};

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};

#[derive(Clone)]
pub struct App {
    pub library: Library,
    pub history: Arc<Mutex<VecDeque<String>>>,
    pub player: Player,
    pub images: ImageMangler,
    pub ui: Weak<AppWindow>,
    pub media_controls: Arc<Mutex<Option<MediaControls>>>,
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
                media_controls: Arc::new(Mutex::new(None)),
            },
        }
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let app = self.app.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| app.navigate(url));

        settings::settings_init(&self.app);        
        player_bar::player_bar_init(&self.app);
        track_list::track_list_init(&self.app);
        track_details::track_details_init(&self.app);
        event_list::event_list_init(&self.app);
        playlist_details::playlist_details_init(&self.app);
        queue_details::queue_details_init(&self.app);

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        let app = self.app.clone();
        self.ui.window().on_close_requested(move || {
            app.ui.upgrade_in_event_loop(|ui| ui.window().set_minimized(true)).unwrap();
            slint::CloseRequestResponse::KeepWindowShown
        });

        let app = self.app.clone();
        self.app.ui.upgrade_in_event_loop(move |ui| {
            let controls = desktop_integration(&app);
            *app.media_controls.lock().unwrap() = Some(controls);
        }).unwrap();

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
        // TODO change this mess to use a registry that pages call during init
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
        // TODO select index
        else if url.starts_with("dimple://queue") {
            crate::ui::pages::queue_details::queue_details(&url, self);
        }
        else if url.starts_with("dimple://history") {
            crate::ui::pages::event_list::event_list(self);
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

// TODO desktop integration using souvlaki. currently broken on Windows.
fn desktop_integration(app: &App) -> MediaControls {
    let hwnd = None;

    let config = PlatformConfig {
        dbus_name: "dimple",
        display_name: "Dimple",
        hwnd,
    };

    let mut controls = MediaControls::new(config).unwrap();
    {
        let app = app.clone();
        controls.attach(move |event: MediaControlEvent| {
            println!("Event received: {:?}", event);
            match event {
                MediaControlEvent::Play => app.player.play(),
                MediaControlEvent::Pause => app.player.pause(),
                MediaControlEvent::Toggle => {
                    if app.player.is_playing() {
                        app.player.pause();
                    }
                    else {
                        app.player.play();
                    }
                },
                MediaControlEvent::Next => app.player.next(),
                MediaControlEvent::Previous => app.player.previous(),
                MediaControlEvent::Stop => app.player.pause(),
                MediaControlEvent::Seek(seek_direction) => todo!(),
                MediaControlEvent::SeekBy(seek_direction, duration) => todo!(),
                MediaControlEvent::SetPosition(media_position) => app.player.seek(media_position.0),
                MediaControlEvent::SetVolume(_) => todo!(),
                MediaControlEvent::OpenUri(_) => todo!(),
                MediaControlEvent::Raise => {
                    app.ui.upgrade_in_event_loop(|ui| ui.window().set_minimized(false)).unwrap();
                },
                MediaControlEvent::Quit => todo!(),
            }
        })
        .unwrap();
    }

    {
        let app = app.clone();
        app.player.on_change(move |player, event| {
            let track_position = player.track_position();
            let track_duration = player.track_duration();
            let current_track = player.current_queue_track();
            let is_playing = player.is_playing();

            let playback = match is_playing {
                true => MediaPlayback::Playing { progress: Some(MediaPosition(track_position)) },
                false => MediaPlayback::Paused { progress: Some(MediaPosition(track_position)) },
            };
            let artist = current_track.clone().map(|t| t.artist).flatten();
            let album = current_track.clone().map(|t| t.album).flatten();
            let title = current_track.clone().map(|t| t.title).flatten();
            let metadata = MediaMetadata {
                duration: Some(track_duration),
                artist: artist.as_deref(),
                album: album.as_deref(),
                title: title.as_deref(),
                ..Default::default()
            };
            if let Ok(mut controls) = app.media_controls.lock() {
                if let Some(controls) = controls.as_mut() {
                    controls.set_playback(playback).unwrap();
                    controls.set_metadata(metadata).unwrap();
                }
            }
        });
    }

    controls
}

