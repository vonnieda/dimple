use std::{sync::{Arc, RwLock}, collections::HashMap};

use eframe::{epaint::{Color32}, egui::{RichText, TextStyle, Context, FontData, Id}};
use egui_extras::{RetainedImage};

use eframe::egui::{FontDefinitions, Visuals, Style, Ui, Response, ImageButton};

use eframe::epaint::{FontFamily, FontId, Stroke};
use resvg::{usvg::{TreeParsing, NodeKind, Group, Node, NodeExt, Text}, FitTo};

use crate::{librarian::Librarian, music_library::{Image}};

use super::{retained_images::RetainedImages, utils};

pub struct Theme {
    pub retained_images: Arc<RwLock<RetainedImages>>,

    pub background_top: Color32,
    pub background_middle: Color32,
    pub background_bottom: Color32,
    // TODO see if any of these can be done with existing egui theme colors
    // instead.
    pub player_background: Color32,
    pub image_placeholder: Color32,
    pub text: Color32,
    pub detail_panel: Color32,

    pub add_icon: SvgIcon,
    pub artist_icon: SvgIcon,
    pub back_icon: SvgIcon,
    pub favorite_icon: SvgIcon,
    pub genre_icon: SvgIcon,
    pub home_icon: SvgIcon,
    pub next_track_icon: SvgIcon,
    pub pause_icon: SvgIcon,
    pub play_icon: SvgIcon,
    pub previous_track_icon: SvgIcon,
    pub release_icon: SvgIcon,
    pub track_icon: SvgIcon,
}

impl Theme {
    pub fn new(librarian: Arc<Librarian>) -> Self {
        Self {
            retained_images: Arc::new(RwLock::new(RetainedImages::new(librarian))),
            background_top: Color32::from_rgb(0x54, 0x3b, 0x67),
            background_middle: Color32::from_rgb(0x21, 0x21, 0x21),
            background_bottom: Color32::from_rgb(0x21, 0x21, 0x21),
            player_background: Color32::from_gray(0x17),
            image_placeholder: Color32::from_gray(0x33),
            detail_panel: Color32::from_gray(0xcc),
            text: Color32::from_gray(220),

            artist_icon: SvgIcon::new(include_bytes!("../icons/feather/users.svg")),
            genre_icon: SvgIcon::new(include_bytes!("../icons/feather/compass.svg")),
            release_icon: SvgIcon::new(include_bytes!("../icons/feather/disc.svg")),
            track_icon: SvgIcon::new(include_bytes!("../icons/feather/music.svg")),
            favorite_icon: SvgIcon::new(include_bytes!("../icons/feather/heart.svg")),

            add_icon: SvgIcon::new(include_bytes!("../icons/feather/plus.svg")),
            play_icon: SvgIcon::new(include_bytes!("../icons/feather/play-circle.svg")),
            pause_icon: SvgIcon::new(include_bytes!("../icons/feather/pause-circle.svg")),
            next_track_icon: SvgIcon::new(include_bytes!("../icons/feather/skip-forward.svg")),
            previous_track_icon: SvgIcon::new(include_bytes!("../icons/feather/skip-back.svg")),

            home_icon: SvgIcon::new(include_bytes!("../icons/feather/home.svg")),
            back_icon: SvgIcon::new(include_bytes!("../icons/feather/arrow-left.svg")),
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
        let bold: FontFamily = FontFamily::Name("Bold".into());
        let style = Style {
            // https://stackoverflow.com/questions/5410066/what-are-the-default-font-sizes-in-pixels-for-the-html-heading-tags-h1-h2/70720104#70720104
            text_styles: [
                (TextStyle::Heading, FontId::new(32.0, Proportional)),
                (TextStyle::Name("Heading 1".into()), FontId::new(32.0, Proportional)),
                (TextStyle::Name("Heading 2".into()), FontId::new(24.0, Proportional)),
                (TextStyle::Name("Heading 3".into()), FontId::new(18.72, Proportional)),
                (TextStyle::Button, FontId::new(16.0, Proportional)),
                (TextStyle::Body, FontId::new(16.0, Proportional)),
                (TextStyle::Small, FontId::new(13.28, Proportional)),

                (TextStyle::Name("Heading Bold".into()), FontId::new(32.0, bold.clone())),
                (TextStyle::Name("Heading 1 Bold".into()), FontId::new(32.0, bold.clone())),
                (TextStyle::Name("Heading 2 Bold".into()), FontId::new(24.0, bold.clone())),
                (TextStyle::Name("Heading 3 Bold".into()), FontId::new(18.72, bold.clone())),
                (TextStyle::Name("Button Bold".into()), FontId::new(16.0, bold.clone())),
                (TextStyle::Name("Body Bold".into()), FontId::new(16.0, bold.clone())),
                (TextStyle::Name("Small Bold".into()), FontId::new(12.0, bold)),

                (TextStyle::Monospace, FontId::new(16.0, Monospace)),
            ].into(),
            ..Default::default()
        };
        ctx.set_style(style);

        let mut visuals = Visuals::dark();
        // Set default text color
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(0., self.text);
        // Set hyperlink color same as text color.
        visuals.hyperlink_color = visuals.widgets.noninteractive.fg_stroke.color;
        // TODO move this into the scrubber.
        visuals.selection.bg_fill = self.background_top;
        ctx.set_visuals(visuals);
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

    pub fn image_button(retained: &RetainedImage, width: usize, height: usize, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;            
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.active.weak_bg_fill = Color32::TRANSPARENT;
            let button = ImageButton::new(retained.texture_id(ui.ctx()), 
                [width as f32, height as f32]);
            ui.add(button)
        }).inner
    }

    pub fn svg_button(svg_icon: &SvgIcon, width: usize, height: usize, ui: &mut Ui) -> Response {
        let fit_to = FitTo::Size(width as u32, height as u32);
        let retained = svg_icon.retained(fit_to);
        Self::image_button(&retained, width, height, ui)
    }

    pub fn svg_image(svg_icon: &SvgIcon, width: usize, height: usize, ui: &mut Ui) -> Response {
        let fit_to = FitTo::Size(width as u32, height as u32);
        let retained = svg_icon.retained(fit_to);
        ui.image(retained.texture_id(ui.ctx()), [width as f32, height as f32])
    }

    pub fn carousel(&self, images: &[Image], width: usize, ui: &mut Ui) -> Response {
        let theme = Theme::get(ui.ctx());
        let texture_id = match images.first() {
            Some(image) => self.retained_images.read().unwrap().get(image, width, width)
                .read()
                .unwrap()
                .texture_id(ui.ctx()),
            None => utils::sample_image(theme.image_placeholder, width, width).texture_id(ui.ctx()),
        };
        ui.add(ImageButton::new(texture_id, [width as f32, width as f32]))
    }
}

/// Stores the bytes of an SVG and renders RetainedImages from the SVG
/// on demand. Caches rendered images internally.
pub struct SvgIcon {
    svg_bytes: Vec<u8>,
    renders: RwLock<HashMap<String, Arc<RetainedImage>>>,
}

impl SvgIcon {
    pub fn new(svg_bytes: &[u8]) -> Self {
        Self {
            svg_bytes: svg_bytes.into(),
            renders: RwLock::new(HashMap::new()),
        }
    }

    pub fn retained(&self, fit_to: FitTo) -> Arc<RetainedImage> {
        // Check the cache for an existing render at this size.
        let key = match fit_to {
            FitTo::Original => "original".to_string(),
            FitTo::Width(w) => format!("width({})", w),
            FitTo::Height(h) => format!("height({})", h),
            FitTo::Size(w, h) => format!("size({}, {})", w, h),
            FitTo::Zoom(f) => format!("zoom{}", f),
        };
        if let Some(retained) = self.renders.read().unwrap().get(&key) {
            return retained.clone()
        }

        // If not cached, render
        let retained = self.render(fit_to);

        // Store it in the cache and return
        let result = Arc::new(retained);
        self.renders.write().unwrap().insert(key, result.clone());
        result
    }

    fn render(&self, fit_to: FitTo) -> RetainedImage {
        let opt = resvg::usvg::Options::default();
        let mut wrapped: Vec<u8> = Vec::new();
        wrapped.extend(r#"<svg xmlns="http://www.w3.org/2000/svg" color="white">"#.as_bytes());
        wrapped.extend(&self.svg_bytes);
        wrapped.extend(r#"</svg>"#.as_bytes());
        let rtree = resvg::usvg::Tree::from_data(&wrapped, &opt).unwrap();

        let pixmap_size = rtree.size.to_screen_size();
        let [w, h] = match fit_to {
            FitTo::Original => [pixmap_size.width(), pixmap_size.height()],
            FitTo::Size(w, h) => [w, h],
            FitTo::Height(h) => [
                (pixmap_size.width() as f32 * (h as f32 / pixmap_size.height() as f32)) as u32,
                h,
            ],
            FitTo::Width(w) => [
                w,
                (pixmap_size.height() as f32 * (w as f32 / pixmap_size.width() as f32)) as u32,
            ],
            FitTo::Zoom(z) => [
                (pixmap_size.width() as f32 * z) as u32,
                (pixmap_size.height() as f32 * z) as u32,
            ],
        };
        let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).unwrap();
        resvg::render(&rtree, 
            fit_to, 
            Default::default(), 
            pixmap.as_mut())
            .unwrap();
        let image = eframe::egui::ColorImage::from_rgba_unmultiplied([w as _, h as _], pixmap.data());
        RetainedImage::from_color_image("", image)
    }
}
