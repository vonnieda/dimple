use std::sync::Arc;

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

    first_frame: bool,
}

impl eframe::App for Dimple {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO gross hack, see:
        // TODO info on how to do something on first frame: https://github.com/emilk/eframe_template/blob/master/src/app.rs#L24
        if !self.first_frame {
            self.first_frame = true;
            catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
        }

        self.main_screen.ui(ctx);
    }
}


impl Dimple {
    pub fn new(sink: Arc<Sink>) -> Self {
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
            first_frame: false,
        }
    }
}
