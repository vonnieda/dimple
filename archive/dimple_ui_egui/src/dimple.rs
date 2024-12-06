use std::sync::Arc;

use dimple_core::library::{LibraryHandle, Library};
use dimple_navidrome_library::navidrome_library::NavidromeLibrary;
use dimple_player::player::{Player, PlayerHandle};
use dimple_sled_library::sled_library::SledLibrary;
use dimple_librarian::librarian::Librarian;
use eframe::CreationContext;

use eframe::egui::{self, Id};

use crate::settings::{Settings, LibraryConfig};
use crate::ui::main_screen::MainScreen;

use crate::ui::theme::Theme;

pub struct Dimple {
    _library: LibraryHandle,
    _player: PlayerHandle,
    _theme: Arc<Theme>,
    main_screen: MainScreen,
}

impl eframe::App for Dimple {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.main_screen.ui(ctx);
    }
}

impl Dimple {
    pub fn new(cc: &CreationContext) -> Self {
        // Load settings
        let settings = Settings::default();

        // Load the Library
        let librarian: Arc<Librarian> = Arc::new(Librarian::from(settings.libraries));
        let library: LibraryHandle = librarian.clone();
        std::thread::spawn(move || {
            librarian.refresh_all_libraries();
        });

        // Set theme
        let ctx = cc.egui_ctx.clone();
        let theme = Arc::new(Theme::new(library.clone()));
        theme.set(&ctx);
        // TODO Move this into Theme::set, and maybe move that into new.
        ctx.data_mut(|wr| {
            wr.insert_temp::<Arc<Theme>>(Id::null(), theme.clone());
        });

        // Set up the music player
        let player = Player::new(library.clone());

        Self {
            main_screen: MainScreen::new(player.clone(), library.clone()),
            _library: library,
            _player: player,
            _theme: theme,
        }
    }
}

impl From<Vec<LibraryConfig>> for Librarian {
    fn from(configs: Vec<LibraryConfig>) -> Self {
        let mut librarian = Self::default();
        for config in configs {
            let library: LibraryHandle = match config {
                LibraryConfig::Navidrome(config) => Arc::new(NavidromeLibrary::from(config)),
                LibraryConfig::Sled(config) => Arc::new(SledLibrary::from(config)),
            };
            librarian.add_library(library);
        }
        librarian
    }
}

impl From<Settings> for Librarian {
    fn from(settings: Settings) -> Self {
        Librarian::from(settings.libraries)
    }
}
