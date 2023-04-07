use std::sync::Arc;

use eframe::egui::{self, Context};

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
            self.refresh(ctx);
        }

        self.main_screen.ui(ctx);
    }
}

impl Dimple {
    pub fn new(sink: Arc<Sink>) -> Self {
        // Load settings
        let settings = Settings::default();

        // Create libraries from configs
        let librarian = Arc::new(Librarian::from(settings.libraries));

        let player = Player::new(sink, librarian.clone());

        Self {
            main_screen: MainScreen::new(player.clone(), librarian.clone()),
            _librarian: librarian,
            _player: player,
            first_frame: false,
        }
    }

    pub fn refresh(&self, _ctx: &Context) {
        // // Launch a thread that refreshes libraries and updates cards.
        // // TODO temporary, just needs a place to live for a moment
        // // TODO currently just runs once, eventually will handle merging
        // // cards and will refresh.
        // let librarian = self.librarian.clone();
        // let cards = self.cards.clone();
        // let ctx = ctx.clone();
        // thread::spawn(move || {
        //     // For each release in the Librarian, create a ReleaseCard and
        //     // push it into the cards Vec. Done in parallel for performance.
        //     // TODO cards go into a hash or cache
        //     let pool = ThreadPool::default();
        //     let librarian = librarian.clone();
        //     let cards = cards.clone();

        //     for release in librarian.releases().iter() {
        //         let librarian = librarian.clone();
        //         let cards = cards.clone();
        //         let ctx = ctx.clone();
        //         pool.execute(move || {
        //             let card = Self::card_from_release(&librarian, &release);
        //             cards.write().unwrap().push(card);
        //             ctx.request_repaint();
        //         });
        //     }
        //     pool.join();

        //     // TODO pausing here, time to work on merging cards.
        //     librarian.refresh();

        //     for release in librarian.releases().iter() {
        //         let librarian = librarian.clone();
        //         let cards = cards.clone();
        //         let ctx = ctx.clone();
        //         pool.execute(move || {
        //             let card = Self::card_from_release(&librarian, &release);
        //             cards.write().unwrap().push(card);
        //             ctx.request_repaint();
        //         });
        //     }
        //     pool.join();
        // });
    }
}
