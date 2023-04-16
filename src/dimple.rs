use std::sync::Arc;

use eframe::CreationContext;


use eframe::egui::{self, Id};



use rodio::Sink;

use crate::librarian::Librarian;

use crate::player::Player;
use crate::player::PlayerHandle;
use crate::settings::Settings;
use crate::ui::main_screen::MainScreen;

use crate::ui::theme::Theme;

pub struct Dimple {
    _librarian: Arc<Librarian>,
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
    pub fn new(cc: &CreationContext, sink: Arc<Sink>) -> Self {
        // Load settings
        let settings = Settings::default();

        // Setup The Librarian
        let librarian = Arc::new(Librarian::from(settings.libraries));
        let librarian_1 = librarian.clone();
        std::thread::spawn(move || {
            librarian_1.refresh_all_libraries();
        });

        // Set theme
        let ctx = cc.egui_ctx.clone();
        let theme = Arc::new(Theme::new(librarian.clone()));
        theme.set(&ctx);
        // TODO Move this into Theme::set, and maybe move that into new.
        ctx.data_mut(|wr| {
            wr.insert_temp::<Arc<Theme>>(Id::null(), theme.clone());
        });

        // Set up the music player
        let player = Player::new(sink, librarian.clone());

        Self {
            main_screen: MainScreen::new(player.clone(), librarian.clone()),
            _librarian: librarian,
            _player: player,
            _theme: theme,
        }
    }
}

