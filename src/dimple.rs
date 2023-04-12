use std::sync::Arc;

use catppuccin_egui::set_theme;
use eframe::CreationContext;

use eframe::egui::{self, Context, SidePanel, FontDefinitions, FontData, Visuals, Style};

use eframe::epaint::{FontFamily, Color32, FontId};
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
        Self::set_theme(&ctx);

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

    pub fn set_theme(ctx: &Context) {
        // catppuccin_egui::set_theme(&ctx, catppuccin_egui::FRAPPE);

        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("Roboto Black".to_owned(),
           FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Black.ttf")));
        fonts.font_data.insert("Roboto Bold".to_owned(),
           FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Bold.ttf")));
        fonts.font_data.insert("Roboto Italic".to_owned(),
            FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Italic.ttf")));
        fonts.font_data.insert("Roboto Light".to_owned(),
           FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Light.ttf")));
        fonts.font_data.insert("Roboto Medium".to_owned(),
           FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Medium.ttf")));
        fonts.font_data.insert("Roboto Regular".to_owned(),
           FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Regular.ttf")));
        fonts.font_data.insert("Roboto Thin".to_owned(),
           FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Thin.ttf")));
        
        // Put my font first (highest priority):
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "Roboto Regular".to_owned());
        
        ctx.set_fonts(fonts);      

        ctx.set_style(Style {
            override_font_id: Some(FontId::proportional(14.0)),
            ..Default::default()
        });

        ctx.set_visuals(Visuals {
            override_text_color: Some(Color32::from_gray(0xAF)),
            hyperlink_color: Color32::from_gray(0xAF),
            ..Default::default()
        });
    }
}

