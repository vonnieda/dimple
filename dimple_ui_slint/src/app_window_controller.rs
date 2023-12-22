use std::sync::{Arc, RwLock};

use dimple_librarian::librarian::Librarian;
use dimple_s3_compatible_storage_library::s3_compatible_storage_library::{S3CompatibleStorageLibraryConfig, S3CompatibleStorageLibrary};

slint::include_modules!();

pub struct AppWindowController {
    ui: AppWindow,
    librarian: Arc<RwLock<Librarian>>,
}

/**
 * Okay, so we need to get a library in but we want to get the UI up ASAP.
 * 
 * Okay, and now I finally see that I was trying to create a navigate method
 * to clean this up and that's where I got stuck with the UI reference.
 */

impl AppWindowController {
    pub fn run(&self) -> Result<(), slint::PlatformError> {
        let ui_handle = self.ui.as_weak();
        // TODO I think the Navigator should be a Rust object? that we
        // set? in the global? Cause all this trash belong somewhere else.
        self.ui.global::<Navigator>().on_navigate(move |url| {
            dbg!(&url);
            let ui = ui_handle.unwrap();
            // TODO I think this mess can change to an enum, or possibly
            // this all just happens inside the appwindow view based on
            // the current URL.
            if url.starts_with("dimple://home") {
                ui.set_page(0);
            }
            else if url.starts_with("dimple://artists") {
                ui.set_page(1);
            }
            else if url.starts_with("dimple://releases") {
                ui.set_page(2);
            }
            else if url.starts_with("dimple://genres") {
                ui.set_page(3);
            }
            else if url.starts_with("dimple://search") {
                ui.set_page(4);
            }
        });

        let librarian = self.librarian.clone();
        std::thread::spawn(move || {
            let s3_config = S3CompatibleStorageLibraryConfig {
                ulid: "0198408312740278340128374".to_string(),
                name: "Backblaze B2 - jason@vonnieda.org".to_string(),
                endpoint: "https://s3.us-west-004.backblazeb2.com".to_string(),
                region_name: "us-west-004".to_string(),
                access_key: "004b18e577e234a0000000002".to_string(),
                secret_key: "K004EsSEVqEP+fQF6uQaiP40YsJ7PNs".to_string(),
                bucket_name: "dimple-music".to_string(),
                prefix: "".to_string(),
            };
            let s3 = S3CompatibleStorageLibrary::new(&s3_config);
            librarian.write().unwrap().add_library(Arc::new(s3));
            librarian.write().unwrap().refresh_all_libraries();
        });

        self.ui.run()
    }
}

impl Default for AppWindowController {
    fn default() -> Self {
        Self { 
            ui: AppWindow::new().unwrap(),
            librarian: Arc::new(RwLock::new(Librarian::default())),
        }
    }
}

        // // Load settings
        // let settings = Settings::default();

        // // Load the Library
        // let librarian: Arc<Librarian> = Arc::new(Librarian::from(settings.libraries));
        // let library: LibraryHandle = librarian.clone();
        // std::thread::spawn(move || {
        //     librarian.refresh_all_libraries();
        // });

        // // Set theme
        // let ctx = cc.egui_ctx.clone();
        // let theme = Arc::new(Theme::new(library.clone()));
        // theme.set(&ctx);
        // // TODO Move this into Theme::set, and maybe move that into new.
        // ctx.data_mut(|wr| {
        //     wr.insert_temp::<Arc<Theme>>(Id::null(), theme.clone());
        // });

        // // Set up the music player
        // let player = Player::new(library.clone());

        // Self {
        //     main_screen: MainScreen::new(player.clone(), library.clone()),
        //     _library: library,
        //     _player: player,
        //     _theme: theme,
        // }
