use dimple_player::player::Player;

use std::{collections::VecDeque, sync::{Arc, Mutex}};


use slint::{ComponentHandle, SharedString, Weak};

use dimple_librarian::librarian::Librarian;

use directories::ProjectDirs;

use crate::ui::{*};

use self::images::ImageMangler;

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
        // TODO Probably this and librarian happens once the UI is up so that we
        // can show errors if needed. 
        let dirs = ProjectDirs::from("lol", "Dimple",  "dimple_ui_slint").unwrap();
        let dir = dirs.data_dir().to_str().unwrap();
        let librarian = Librarian::new(dir);
        let player = Player::new(Arc::new(librarian.clone()));
        let ui_weak = ui.as_weak();
        Self {
            ui,
            app: App {
                librarian: librarian.clone(),
                history: Arc::new(Mutex::new(VecDeque::new())),
                player,
                images: ImageMangler::new(librarian.clone()),
                ui: ui_weak,
            }
        }
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let app = self.app.clone();
        self.ui.global::<Navigator>().on_navigate(move |url| app.navigate(url));
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
            self.set_page(Page::Search);
        }
        else if url.starts_with("dimple://home") {
            self.set_page(Page::Home);
        } 
        else if url.starts_with("dimple://artists") {
            crate::ui::pages::artist_list::artist_list(self);
        }
        else if url.starts_with("dimple://artist/") {
            crate::ui::pages::artist_details::artist_details(&url, self);
        }
        else if url.starts_with("dimple://release-groups") {
            self.set_page(Page::ReleaseGroupList);
        }
        else if url.starts_with("dimple://release-group/") {
            self.set_page(Page::ReleaseGroupDetails);
        }
        else if url.starts_with("dimple://releases") {
            self.set_page(Page::ReleaseList);
        }
        else if url.starts_with("dimple://release/") {
            self.set_page(Page::ReleaseDetails);
        }
        else if url.starts_with("dimple://tracks") {
            crate::ui::pages::track_list::track_list(&self.librarian, self.ui.clone());
        }
        else if url.starts_with("dimple://track/") {
            self.set_page(Page::TrackDetails);
        }
        else if url == "dimple://settings" {
            self.set_page(Page::Settings);
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