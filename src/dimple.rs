use std::sync::Arc;

use catppuccin_egui::set_theme;
use eframe::CreationContext;

use eframe::egui::style::{Widgets, WidgetVisuals, Spacing};
use eframe::egui::{self, Context, SidePanel, FontDefinitions, FontData, Visuals, Style, TextStyle, RichText, Id, Ui, Response, ImageButton};

use eframe::epaint::{FontFamily, Color32, FontId, Stroke, TextureId};
use egui_extras::RetainedImage;
use rodio::Sink;

use crate::librarian::Librarian;

use crate::player::Player;
use crate::player::PlayerHandle;
use crate::settings::Settings;
use crate::ui::main_screen::MainScreen;
use crate::ui::retained_images::RetainedImages;

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
        Theme::set(&ctx);

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

pub struct Theme {
}

/// Supplies colors and sizes for the app's theme.
impl Theme {
    // pub const background_top: Color32 = Color32::from_rgb(0x02, 0x55, 0x70); // Blue
    pub const background_top: Color32 = Color32::from_rgb(0x54, 0x3b, 0x67); // Purple
    pub const background_middle: Color32 = Color32::from_rgb(0x21, 0x21, 0x21);
    pub const background_bottom: Color32 = Color32::from_rgb(0x21, 0x21, 0x21);
    pub const text: Color32 = Color32::from_gray(206);
    pub const player_background: Color32 = Color32::from_gray(0x17);

    // TODO do a builder style thing for RichText. It almost already is there
    // but basically just needs defaults for font(), or I need to learn the
    // idiomatic way to get a font reference. I think the main issue is that
    // the library doesn't really support font weights yet. If it did...
    // I like the idea of a set of named styles, 

    // pub fn body(str: &str, size: f32) -> RichText {
    //     RichText::new(str).font(TextStyle::Body)
    // }

    // pub fn bold(str: &str, size: f32) -> RichText {

    // }

    pub fn svg_icon(bytes: &[u8]) -> RetainedImage {
        RetainedImage::from_svg_bytes("", bytes).unwrap()
    }

    // Alias for button
    pub fn bigger(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Button)
    }

    pub fn bold(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Name("Body Bold".into()))
    }

    pub fn big_n_bold(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Name("Button Bold".into()))
    }

    pub fn small(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Small)
    }

    pub fn small_n_bold(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Name("Small Bold".into()))
    }

    pub fn set(ctx: &Context) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("Commissioner-Regular".to_owned(),
           FontData::from_static(include_bytes!("fonts/Commissioner/static/Commissioner/Commissioner-Regular.ttf")));
        fonts.font_data.insert("Commissioner-Bold".to_owned(),
           FontData::from_static(include_bytes!("fonts/Commissioner/static/Commissioner/Commissioner-Bold.ttf")));
        fonts.font_data.insert("Roboto-Regular".to_owned(),
           FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Regular.ttf")));
        fonts.font_data.insert("Roboto-Bold".to_owned(),
           FontData::from_static(include_bytes!("fonts/Roboto/Roboto-Bold.ttf")));
           
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "Commissioner-Regular".to_owned());
        fonts.families.insert(FontFamily::Name("Bold".into()), vec!["Commissioner-Bold".into()]);
        // fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "Roboto-Regular".to_owned());
        // fonts.families.insert(FontFamily::Name("Bold".into()), vec!["Roboto-Bold".into()]);
        ctx.set_fonts(fonts);
    
        use FontFamily::{Monospace, Proportional};
        let style = Style {
            text_styles: [
                (TextStyle::Heading, FontId::new(26.0, Proportional)),
                (TextStyle::Button, FontId::new(16.0, Proportional)),
                (TextStyle::Name("Button Bold".into()), FontId::new(16.0, FontFamily::Name("Bold".into()))),
                (TextStyle::Body, FontId::new(14.0, Proportional)),
                (TextStyle::Name("Body Bold".into()), FontId::new(14.0, FontFamily::Name("Bold".into()))),
                (TextStyle::Small, FontId::new(12.0, Proportional)),
                (TextStyle::Name("Small Bold".into()), FontId::new(12.0, FontFamily::Name("Bold".into()))),

                (TextStyle::Monospace, FontId::new(14.0, Monospace)),
            ].into(),
            ..Default::default()
        };
        ctx.set_style(style);

        // TODO buttons backgrounds transparent
        let mut visuals = Visuals {
            hyperlink_color: Self::text,
            panel_fill: Color32::TRANSPARENT, // So the background is visible
            text_cursor_preview: true,
            slider_trailing_fill: true,
            widgets: Widgets {
                ..Default::default()
            },
            ..Default::default()
        };
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(0., Self::text);
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE; // Hide lines between panels
        visuals.selection.bg_fill = Self::background_top;
        ctx.set_visuals(visuals);
    }

    pub fn icon_button(retained: &RetainedImage, width: usize, height: usize, 
        ctx: &Context, ui: &mut Ui) -> Response {

        ui.add(ImageButton::new(retained.texture_id(ctx), [width as f32, height as f32]))
    }
}
