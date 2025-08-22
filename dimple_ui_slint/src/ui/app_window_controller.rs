use dimple_core::{librarian::Librarian, library::Library, player::{PlayWhen, Player, PlayerEvent}, plugins::plugins::Plugins};
use dimple_db::Db;
use std::{collections::VecDeque, env, path::Path, sync::{Arc, Mutex, RwLock}};

use slint::{ComponentHandle, SharedString, Weak};

use directories::ProjectDirs;

use crate::{config::Config, ui::{components::lazy_image::init_lazy_image_loader, pages::queue_details::QueueDetailsController, *}};

use self::images::ImageMangler;

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};

use super::player_bar::PlayerBar;

#[derive(Clone)]
pub struct App {
    pub config: Config,
    pub library: Library,
    pub history: Arc<Mutex<VecDeque<String>>>,
    pub player: Player,
    pub images: ImageMangler,
    pub ui: Weak<AppWindow>,
    pub media_controls: Arc<Mutex<Option<MediaControls>>>,
    pub plugins: Plugins,
    pub librarian: Librarian,
    pub artist_details_controller: Arc<RwLock<Option<pages::artist_details::ArtistDetailsController>>>,
    pub release_details_controller: Arc<RwLock<Option<pages::release_details::ReleaseDetailsController>>>,
    pub track_details_controller: Arc<RwLock<Option<pages::track_details::TrackDetailsController>>>,
    pub queue_details_controller: Arc<RwLock<Option<pages::queue_details::QueueDetailsController>>>,
}

pub struct AppWindowController {
    ui: AppWindow,
    app: App,
    _settings_controller: pages::settings::SettingsController,
    _history_list_controller: pages::history_list::HistoryListController,
    _artist_list_controller: pages::artist_list::ArtistListController,
    _playlist_list_controller: pages::playlist_list::PlaylistListController,
    _genre_list_controller: pages::genre_list::GenreListController,
    _release_list_controller: pages::release_list::ReleaseListController,
    _track_list_controller: pages::track_list::TrackListController,
    _search_results_controller: pages::search_results::SearchResultsController,
}

impl AppWindowController {
    pub fn new() -> Self {
        let ui = AppWindow::new().unwrap();
        // TODO This and library should happen once the UI is up so that we
        // can show errors if needed. 
        // So, launch the UI, then launch a thread that loads the config
        // then that can load the library.
        let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
        let mut data_dir = dirs.data_dir().to_path_buf();
        let mut config_dir = dirs.config_dir().to_path_buf();
        let mut cache_dir = dirs.cache_dir().to_path_buf();
        if let Some(root) = env::var("DIMPLE_ROOT").ok() {
            let root_dir = Path::new(&root.to_string()).to_path_buf();
            data_dir = root_dir.join("data").to_path_buf();
            config_dir = root_dir.join("config").to_path_buf();
            cache_dir = root_dir.join("cache").to_path_buf();
        }
        let library_path = data_dir.join("library.db");
        let config_path = config_dir.join("config.db");
        let image_cache_dir = cache_dir.join("image_cache");
        dbg!(&data_dir, &cache_dir, &library_path, &image_cache_dir, &config_path);
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::create_dir_all(&image_cache_dir).unwrap();

        let library = Library::open(library_path.to_str().unwrap());
        let config = Config::new(Db::open(config_path.to_str().unwrap()).unwrap()).unwrap();
        let player = Player::new(Arc::new(library.clone()));
        let plugins = Plugins::new(cache_dir.to_str().unwrap());
        plugins.add_default_plugins();
        let librarian = Librarian::new(&library, &plugins);
        let images = ImageMangler::new(librarian.clone(), ui.as_weak().clone(), image_cache_dir.to_str().unwrap());        
        init_lazy_image_loader(&ui, &library);
        let ui_weak = ui.as_weak();
        // TODO look at this.
        // Create placeholders for detail controllers to break circular dependency
        let artist_details_controller = Arc::new(RwLock::new(None));
        let release_details_controller = Arc::new(RwLock::new(None));
        let track_details_controller = Arc::new(RwLock::new(None));
        let queue_details_controller = Arc::new(RwLock::new(None));
        
        let app = App {
            config,
            library,
            history: Arc::new(Mutex::new(VecDeque::new())),
            player,
            images,
            ui: ui_weak,
            media_controls: Arc::new(Mutex::new(None)),
            plugins,
            librarian,
            artist_details_controller: artist_details_controller.clone(),
            release_details_controller: release_details_controller.clone(),
            track_details_controller: track_details_controller.clone(),
            queue_details_controller: queue_details_controller.clone(),
        };
        
        // Now create the real controllers and replace the placeholders
        let real_artist_controller = pages::artist_details::ArtistDetailsController::new(&app).unwrap();
        *artist_details_controller.write().unwrap() = Some(real_artist_controller);
        
        let real_release_controller = pages::release_details::ReleaseDetailsController::new(&app).unwrap();
        *release_details_controller.write().unwrap() = Some(real_release_controller);
        
        let real_track_controller = pages::track_details::TrackDetailsController::new(&app).unwrap();
        *track_details_controller.write().unwrap() = Some(real_track_controller);
        
        let real_queue_controller = pages::queue_details::QueueDetailsController::new(&app).unwrap();
        *queue_details_controller.write().unwrap() = Some(real_queue_controller);

        // Initialize page controllers
        let settings_controller = pages::settings::SettingsController::new(&app).unwrap();
        let history_list_controller = pages::history_list::HistoryListController::new(&app).unwrap();
        let artist_list_controller = pages::artist_list::ArtistListController::new(&app).unwrap();
        let playlist_list_controller = pages::playlist_list::PlaylistListController::new(&app).unwrap();
        let genre_list_controller = pages::genre_list::GenreListController::new(&app).unwrap();
        let release_list_controller = pages::release_list::ReleaseListController::new(&app).unwrap();
        let track_list_controller = pages::track_list::TrackListController::new(&app).unwrap();
        let search_results_controller = pages::search_results::SearchResultsController::new(&app).unwrap();
        
        Self {
            ui,
            app,
            _settings_controller: settings_controller,
            _history_list_controller: history_list_controller,
            _artist_list_controller: artist_list_controller,
            _playlist_list_controller: playlist_list_controller,
            _genre_list_controller: genre_list_controller,
            _release_list_controller: release_list_controller,
            _track_list_controller: track_list_controller,
            _search_results_controller: search_results_controller,
        }
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let app = self.app.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| app.navigate(url));

        let app = self.app.clone();
        self.ui.global::<crate::ui::AppState>().on_play_next(move |key| {
            app.player.enqueue(&key, PlayWhen::Next);
        });
        let app = self.app.clone();
        self.ui.global::<crate::ui::AppState>().on_play_later(move |key| {
            app.player.enqueue(&key, PlayWhen::Last);
        });
        let app = self.app.clone();
        self.ui.global::<crate::ui::AppState>().on_play_now(move |key| {
            app.player.enqueue(&key, PlayWhen::Now);
        });

        let _player_bar = PlayerBar::new(&self.app);

        pages::home::home_init(&self.app);
        pages::genre_details::genre_details_init(&self.app);
        pages::playlist_details::playlist_details_init(&self.app);

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
        // TODO change this mess to use a registry that pages call during init
        // Or maybe get rid of the navigator altogether? Now that we have proper
        // callbacks it might be superfluous.
        // TODO ideally the switching of pages would be in the slint files to
        // make the ui editing experience a LOT better
        else if url.starts_with("dimple://home") {
            pages::home::home(self);
        } 
        else if url.starts_with("dimple://artists") {
            self.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::ArtistList)).unwrap();
        }
        else if url.starts_with("dimple://artist/") {
            if let Some(ref mut controller) = self.artist_details_controller.write().unwrap().as_mut() {
                crate::ui::pages::artist_details::artist_details(&url, self, controller);
            }
        }
        else if url.starts_with("dimple://releases") {
            self.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::ReleaseList)).unwrap();
        }
        else if url.starts_with("dimple://release/") {
            if let Some(ref mut controller) = self.release_details_controller.write().unwrap().as_mut() {
                crate::ui::pages::release_details::release_details(&url, self, controller);
            }
        }
        else if url.starts_with("dimple://tracks") {
            self.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::TrackList)).unwrap();
        }
        else if url.starts_with("dimple://track/") {
            if let Some(ref mut controller) = self.track_details_controller.write().unwrap().as_mut() {
                pages::track_details::track_details(&url, self, controller);
            }
        }
        else if url.starts_with("dimple://genres") {
            self.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::GenreList)).unwrap();
        }
        else if url.starts_with("dimple://genre/") {
            crate::ui::pages::genre_details::genre_details(&url, self);
        }
        else if url.starts_with("dimple://playlists") {
            self.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::PlaylistList)).unwrap();
        }
        else if url.starts_with("dimple://playlist/") {
            pages::playlist_details::playlist_details(&url, self);
        }
        else if url.starts_with("dimple://queue") {
            QueueDetailsController::show(self);
        }
        else if url.starts_with("dimple://history") {
            pages::history_list::history_list(self);
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
        app.player.notifier.observe(move |event| {
            let track_position = player.track_position();
            let track_duration = player.track_duration();
            let current_track = player.current_queue_track();
            let is_playing = player.is_playing();

            let playback = match is_playing {
                true => MediaPlayback::Playing { progress: Some(MediaPosition(track_position)) },
                false => MediaPlayback::Paused { progress: Some(MediaPosition(track_position)) },
            };
            let artist = current_track.clone().map(|t| t.artist_name(&app.library)).flatten();
            let album = current_track.clone().map(|t| t.album_name(&app.library)).flatten();
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
                    if let PlayerEvent::Position(p) = event {
                        controls.set_playback(playback).unwrap();
                    }
                    else {
                        controls.set_metadata(metadata).unwrap();
                        controls.set_playback(playback).unwrap();
                    }
                }
            }
        });
    }

    controls
}

