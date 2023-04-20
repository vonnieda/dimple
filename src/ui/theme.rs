use std::sync::{Arc, RwLock};

use eframe::{epaint::{Color32, Rounding}, egui::{RichText, TextStyle, Context, FontData, Id}};
use egui_extras::RetainedImage;

use eframe::egui::{FontDefinitions, Visuals, Style, Ui, Response, ImageButton};

use eframe::epaint::{FontFamily, FontId, Stroke};

use crate::{librarian::Librarian, music_library::{Image}};

use super::{retained_images::RetainedImages, utils};

pub struct Theme {
    pub retained_images: Arc<RwLock<RetainedImages>>,

    pub background_top: Color32,
    pub background_middle: Color32,
    pub background_bottom: Color32,
    // TODO see if any of these can be done with existing egui theme colors
    // instead.
    // pub player_background: Color32,
    // pub image_placeholder: Color32,
    // pub text: Color32,
    // pub detail_panel: Color32,

    pub add_icon: RetainedImage,
    pub artist_icon: RetainedImage,
    pub back_icon: RetainedImage,
    pub favorite_icon: RetainedImage,
    pub genre_icon: RetainedImage,
    pub home_icon: RetainedImage,
    pub next_track_icon: RetainedImage,
    pub pause_icon: RetainedImage,
    pub play_icon: RetainedImage,
    pub previous_track_icon: RetainedImage,
    pub release_icon: RetainedImage,
    pub track_icon: RetainedImage,
}

impl Theme {
    pub fn new(librarian: Arc<Librarian>) -> Self {
        Self {
            retained_images: Arc::new(RwLock::new(RetainedImages::new(librarian))),
            // background_top: Color32::from_rgb(186, 136, 255),
            // background_middle: Color32::from_gray(240),
            // background_bottom: Color32::from_gray(240),
            background_top: Color32::from_rgb(0x54, 0x3b, 0x67),
            background_middle: Color32::from_rgb(0x21, 0x21, 0x21),
            background_bottom: Color32::from_rgb(0x21, 0x21, 0x21),
            // player_background: Color32::from_gray(0x17),
            // image_placeholder: Color32::from_gray(0x33),
            // detail_panel: Color32::from_gray(0xcc),
            // text: Color32::from_gray(206),

            add_icon: Self::svg_icon(include_bytes!("../icons/material/add_circle_FILL0_wght400_GRAD0_opsz48.svg")),
            artist_icon: Self::svg_icon(include_bytes!("../icons/material/group_FILL0_wght400_GRAD0_opsz48.svg")),
            genre_icon: Self::svg_icon(include_bytes!("../icons/material/theater_comedy_FILL0_wght400_GRAD0_opsz48.svg")),
            release_icon: Self::svg_icon(include_bytes!("../icons/material/album_FILL0_wght400_GRAD0_opsz48.svg")),
            track_icon: Self::svg_icon(include_bytes!("../icons/material/music_note_FILL0_wght400_GRAD0_opsz48.svg")),
            favorite_icon: Self::svg_icon(include_bytes!("../icons/material/favorite_FILL0_wght400_GRAD0_opsz48.svg")),

            play_icon: Self::svg_icon(include_bytes!("../icons/material/play_circle_FILL0_wght400_GRAD0_opsz48.svg")),
            // play_icon: Theme::svg_icon(include_bytes!("../icons/material/play_circle_FILL1_wght400_GRAD0_opsz48.svg")),

            pause_icon: Theme::svg_icon(include_bytes!("../icons/material/pause_circle_FILL0_wght400_GRAD0_opsz48.svg")),
            next_track_icon: Theme::svg_icon(include_bytes!("../icons/material/skip_next_FILL1_wght400_GRAD0_opsz48.svg")),
            previous_track_icon: Theme::svg_icon(include_bytes!("../icons/material/skip_previous_FILL1_wght400_GRAD0_opsz48.svg")),

            home_icon: Theme::svg_icon(include_bytes!("../icons/material/home_FILL0_wght400_GRAD0_opsz48.svg")),
            back_icon: Theme::svg_icon(include_bytes!("../icons/material/arrow_back_FILL0_wght400_GRAD0_opsz48.svg")),
        }
    }

    pub fn get(ctx: &Context) -> Arc<Theme> {
        ctx.data_mut(|writer| {
            writer.get_temp::<Arc<Theme>>(Id::null()).unwrap()
        })
    }

    pub fn set(&self, ctx: &Context) {
        let mut fonts = FontDefinitions::default();
        // fonts.font_data.insert("Commissioner-Regular".to_owned(),
        //    FontData::from_static(include_bytes!("../fonts/Commissioner/static/Commissioner/Commissioner-Regular.ttf")));
        // fonts.font_data.insert("Commissioner-Bold".to_owned(),
        //    FontData::from_static(include_bytes!("../fonts/Commissioner/static/Commissioner/Commissioner-Bold.ttf")));
        fonts.font_data.insert("Roboto-Regular".to_owned(),
           FontData::from_static(include_bytes!("../fonts/Roboto/Roboto-Regular.ttf")));
        fonts.font_data.insert("Roboto-Bold".to_owned(),
           FontData::from_static(include_bytes!("../fonts/Roboto/Roboto-Bold.ttf")));
           
        // fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "Commissioner-Regular".to_owned());
        // fonts.families.insert(FontFamily::Name("Bold".into()), vec!["Commissioner-Bold".into()]);
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "Roboto-Regular".to_owned());
        fonts.families.insert(FontFamily::Name("Bold".into()), vec!["Roboto-Bold".into()]);
        ctx.set_fonts(fonts);
    
        use FontFamily::{Monospace, Proportional};
        let Bold: FontFamily = FontFamily::Name("Bold".into());
        let style = Style {
            // https://stackoverflow.com/questions/5410066/what-are-the-default-font-sizes-in-pixels-for-the-html-heading-tags-h1-h2/70720104#70720104
            text_styles: [
                (TextStyle::Heading, FontId::new(32.0, Bold.clone())),
                (TextStyle::Name("Heading 1".into()), FontId::new(32.0, Bold.clone())),
                (TextStyle::Name("Heading 2".into()), FontId::new(24.0, Bold.clone())),
                (TextStyle::Name("Heading 3".into()), FontId::new(18.72, Bold.clone())),
                (TextStyle::Button, FontId::new(16.0, Proportional)),
                (TextStyle::Body, FontId::new(16.0, Proportional)),
                (TextStyle::Small, FontId::new(13.28, Proportional)),

                (TextStyle::Name("Button Bold".into()), FontId::new(16.0, Bold.clone())),
                (TextStyle::Name("Body Bold".into()), FontId::new(16.0, Bold.clone())),
                (TextStyle::Name("Small Bold".into()), FontId::new(12.0, Bold.clone())),

                (TextStyle::Monospace, FontId::new(16.0, Monospace)),
            ].into(),
            ..Default::default()
        };
        ctx.set_style(style);

        let mut visuals = Visuals::dark();
        // Set default text color
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(0., Color32::from_gray(206));
        // Set hyperlink color same as text color.
        visuals.hyperlink_color = visuals.widgets.noninteractive.fg_stroke.color;
        // TODO move this into the scrubber.
        visuals.selection.bg_fill = self.background_top;
        ctx.set_visuals(visuals);
    }

    pub fn svg_icon(bytes: &[u8]) -> RetainedImage {
        RetainedImage::from_svg_bytes("", bytes).unwrap()
    }

    pub fn heading3(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Name("Heading 3".into()))
    }

    pub fn heading2(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Name("Heading 2".into()))
    }

    pub fn heading1(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Heading)
    }

    pub fn heading(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Heading)
    }

    pub fn bold(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Name("Body Bold".into()))
    }

    pub fn small(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Small)
    }

    pub fn small_bold(str: &str) -> RichText {
        RichText::new(str).text_style(TextStyle::Name("Small Bold".into()))
    }

    // TODO work on frame
    pub fn icon_button(retained: &RetainedImage, width: usize, height: usize, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;            
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.active.weak_bg_fill = Color32::TRANSPARENT;
            let button = ImageButton::new(retained.texture_id(ui.ctx()), 
                [width as f32, height as f32]);
            ui.add(button)
        }).inner
    }

    pub fn carousel(&self, images: &[Image], width: usize, ui: &mut Ui) -> Response {
        let theme = Theme::get(ui.ctx());
        let texture_id = match images.first() {
            Some(image) => self.retained_images.read().unwrap().get(image, width, width)
                .read()
                .unwrap()
                .texture_id(ui.ctx()),
            None => utils::sample_image(Color32::BLACK, width, width).texture_id(ui.ctx()),
        };
        ui.add(ImageButton::new(texture_id, [width as f32, width as f32]))
    }

    // TODO links functions like artists, genres used in itemdetails
}
