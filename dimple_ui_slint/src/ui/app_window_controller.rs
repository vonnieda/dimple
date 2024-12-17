use dimple_core::{library::Library, player::Player};
use player_bar;
use std::{collections::VecDeque, sync::{Arc, Mutex}};

use slint::{ComponentHandle, SharedString, Weak};

use directories::ProjectDirs;

use crate::{config::Config, ui::*};

use self::images::ImageMangler;

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};

#[derive(Clone)]
pub struct App {
    pub config: Config,
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
        // TODO This and library should happen once the UI is up so that we
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
                config: Config::default(),
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

        player_bar::player_bar_init(&self.app);

        pages::artist_list::artist_list_init(&self.app);
        pages::event_list::event_list_init(&self.app);
        pages::genre_list::genre_list_init(&self.app);
        pages::home::home_init(&self.app);
        pages::playlist_list::playlist_list_init(&self.app);
        pages::playlist_details::playlist_details_init(&self.app);
        pages::queue_details::queue_details_init(&self.app);
        pages::release_list::release_list_init(&self.app);
        pages::search_results::search_results_init(&self.app);        
        pages::settings::settings_init(&self.app);        
        pages::track_list::track_list_init(&self.app);
        pages::track_details::track_details_init(&self.app);

        self.ui.global::<Navigator>().invoke_navigate("dimple://home".into());

        let app = self.app.clone();
        self.ui.window().on_close_requested(move || {
            app.ui.upgrade_in_event_loop(|ui| ui.window().set_minimized(true)).unwrap();
            slint::CloseRequestResponse::KeepWindowShown
        });

        let app = self.app.clone();
        self.app.ui.upgrade_in_event_loop(move |ui| {
            let controls = desktop_integration(&app, &ui);
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
            pages::search_results::search_results(&url, self);
        }
        // TODO change this mess to use a registry that pages call during init
        // Or maybe get rid of the navigator altogether? Now that we have proper
        // callbacks it might be superfluous.
        else if url.starts_with("dimple://home") {
            pages::home::home(self);
        } 
        else if url.starts_with("dimple://artists") {
            pages::artist_list::artist_list(self);
        }
        // else if url.starts_with("dimple://artist/") {
        //     crate::ui::pages::artist_details::artist_details(&url, self);
        // }
        // else if url.starts_with("dimple://release-groups") {
        //     crate::ui::pages::release_group_list::release_group_list(self);
        // }
        // else if url.starts_with("dimple://release-group/") {
        //     crate::ui::pages::release_group_details::release_group_details(&url, self);
        // }
        else if url.starts_with("dimple://releases") {
            pages::release_list::release_list(self);
        }
        // else if url.starts_with("dimple://release/") {
        //     crate::ui::pages::release_details::release_details(&url, self);
        // }
        // else if url.starts_with("dimple://recording/") {
        //     crate::ui::pages::recording_details::recording_details(&url, self);
        // }
        else if url.starts_with("dimple://tracks") {
            pages::track_list::track_list(self);
        }
        else if url.starts_with("dimple://track/") {
            pages::track_details::track_details(&url, self);
        }
        else if url.starts_with("dimple://genres") {
            pages::genre_list::genre_list(self);
        }
        // else if url.starts_with("dimple://genre/") {
        //     crate::ui::pages::genre_details::genre_details(&url, self);
        // }
        else if url.starts_with("dimple://playlists") {
            pages::playlist_list::playlist_list(self);
        }
        else if url.starts_with("dimple://playlist/") {
            pages::playlist_details::playlist_details(&url, self);
        }
        else if url.starts_with("dimple://queue") {
            pages::queue_details::queue_details(&url, self);
        }
        else if url.starts_with("dimple://history") {
            pages::event_list::event_list(self);
        }
        else if url == "dimple://settings" {
            pages::settings::settings(self);
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

fn desktop_integration(app: &App, ui: &AppWindow) -> MediaControls {

    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    use {
        std::os::raw::c_void,
        raw_window_handle::HasWindowHandle,
        raw_window_handle::HasRawWindowHandle,
        raw_window_handle::RawWindowHandle,
    };
    #[cfg(target_os = "windows")]
    let hwnd: Option<*mut c_void> = {
        let window_handle = ui.window().window_handle();
        let raw_window_handle = window_handle.raw_window_handle().unwrap();
        let handle: raw_window_handle::Win32WindowHandle = match raw_window_handle {
            RawWindowHandle::Win32(h) => h,
            _ => unreachable!(),
        };
        Some(handle.hwnd.get() as *mut c_void)
    };

    let config = PlatformConfig {
        dbus_name: "dimple",
        display_name: "Dimple",
        hwnd,
    };

    let mut controls = MediaControls::new(config).unwrap();
    {
        let app = app.clone();
        controls.attach(move |event: MediaControlEvent| {
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
        let player = app.player.clone();
        app.player.on_change(Box::new(move |event| {
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
                    if event == "position" {
                        controls.set_playback(playback).unwrap();
                    }
                    else {
                        controls.set_metadata(metadata).unwrap();
                        controls.set_playback(playback).unwrap();
                    }
                }
            }
        }));
    }

    controls
}

