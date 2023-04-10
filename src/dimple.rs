use std::sync::Arc;

use eframe::CreationContext;

use eframe::egui::{self};

use rodio::Sink;

use crate::librarian::Librarian;

use crate::player::Player;
use crate::player::PlayerHandle;
use crate::settings::Settings;
use crate::ui::main_screen::MainScreen;

pub struct Dimple {
    _librarian: Arc<Librarian>,
    _player: PlayerHandle,
    main_screen: MainScreen,
}

impl eframe::App for Dimple {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.main_screen.ui(ctx);
    }
}

impl Dimple {
    pub fn new(cc: &CreationContext, sink: Arc<Sink>) -> Self {
        let ctx = cc.egui_ctx.clone();
        catppuccin_egui::set_theme(&ctx, catppuccin_egui::FRAPPE);

        // Load settings
        let settings = Settings::default();

        // Configure music libraries
        let librarian = Arc::new(Librarian::from(settings.libraries));
        let librarian_1 = librarian.clone();
        std::thread::spawn(move || {
            librarian_1.refresh_all_libraries();
        });
        
        // Set up the music player
        let player = Player::new(sink, librarian.clone());

        Self {
            main_screen: MainScreen::new(player.clone(), librarian.clone()),
            _librarian: librarian,
            _player: player,
        }
    }
}

