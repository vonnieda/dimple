use std::sync::Arc;

use eframe::egui::{Context, ImageButton, Ui, Layout, Frame, Margin};
use eframe::emath::Align;
use eframe::epaint::{Color32, ColorImage, FontId, Stroke, Rect};

use egui_extras::RetainedImage;

use crate::dimple::Theme;
use crate::player::{PlayerHandle};

use super::retained_images::RetainedImages;
use super::scrubber::{PlotScrubber, SliderScrubber};
use super::utils;

#[derive()]
pub struct PlayerBar {
    player: PlayerHandle,
    plot_scrubber: PlotScrubber,
    slider_scrubber: SliderScrubber,
    retained_images: Arc<RetainedImages>,

    artist_icon: RetainedImage,
    release_icon: RetainedImage,
    track_icon: RetainedImage,

    play_icon: RetainedImage,    
    pause_icon: RetainedImage,    
    previous_icon: RetainedImage,    
    next_icon: RetainedImage,    
}

impl PlayerBar {
    const now_playing_thumbnail_size: f32 = 130.0;
    const up_next_width: f32 = 100.0;
    const up_next_thumbnail_size: f32 = 80.0;

    pub fn new(player: PlayerHandle, retained_images: Arc<RetainedImages>) -> Self {
        Self {
            player,
            plot_scrubber: PlotScrubber::default(),
            slider_scrubber: SliderScrubber::default(),
            retained_images,

            artist_icon: Theme::svg_icon(include_bytes!("../icons/material/group_FILL0_wght400_GRAD0_opsz48.svg")),
            release_icon: Theme::svg_icon(include_bytes!("../icons/material/album_FILL0_wght400_GRAD0_opsz48.svg")),
            track_icon: Theme::svg_icon(include_bytes!("../icons/material/music_note_FILL0_wght400_GRAD0_opsz48.svg")),
            play_icon: Theme::svg_icon(include_bytes!("../icons/material/play_circle_FILL1_wght400_GRAD0_opsz48.svg")),
            pause_icon: Theme::svg_icon(include_bytes!("../icons/material/pause_FILL1_wght400_GRAD0_opsz48.svg")),
            next_icon: Theme::svg_icon(include_bytes!("../icons/material/skip_next_FILL1_wght400_GRAD0_opsz48.svg")),
            previous_icon: Theme::svg_icon(include_bytes!("../icons/material/skip_previous_FILL1_wght400_GRAD0_opsz48.svg")),
        }
    }

    pub fn ui(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            Frame::none().inner_margin(Margin {
                top: 8.0,
                left: 0.0,
                bottom: 0.0,
                right: 0.0,
            }).show(ui, |ui| {
                self.now_playing(ctx, ui);
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - Self::up_next_width);
                    ui.horizontal_top(|ui| {
                        self.track_info(ctx, ui);
                        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                            self.play_controls(ctx, ui);
                        });
                    });
                    ui.scope(|ui| {
                        // The negative Y spacing slides the plot behind the
                        // handle of the slider and makes it look awesome.
                        ui.spacing_mut().item_spacing = [0.0, -3.0].into();
                        self.plot_scrubber.ui(ctx, ui);
                        self.slider_scrubber.ui(self.player.clone(), ctx, ui);
                    });
                    self.timers(ctx, ui);
                });
            });
            self.up_next(ctx, ui);
        });
    }

    pub fn track_info(&self, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            let queue_item = self
                .player
                .read()
                .map(|player| player.current_queue_item())
                .unwrap_or(None);
            let track_title = queue_item
                .as_ref()
                .map_or("".to_string(), |qi| qi.track.title.clone());
            let release_title = queue_item
                .as_ref()
                .map_or("".to_string(), |qi| qi.release.title.clone());
            let artist_name = queue_item
                .as_ref()
                .map_or("".to_string(), |qi| qi.release.artist());

            self.fav_icon_label(
                &self.track_icon,
                &track_title,
                false,
                ctx,
                ui,
            );
            self.fav_icon_label(
                &self.release_icon,
                &release_title,
                false,
                ctx,
                ui,
            );
            self.fav_icon_label(
                &self.artist_icon,
                &artist_name,
                false,
                ctx,
                ui,
            );
        });
    }

    pub fn now_playing(&self, ctx: &Context, ui: &mut Ui) {
        let thumbnail_size: usize = Self::now_playing_thumbnail_size as usize;
        if let Some(item) = self.player.read().unwrap().current_queue_item() {
            let image =
                self.retained_images
                    .get(item.release.art.first().unwrap(), thumbnail_size, thumbnail_size);
            ui.add(ImageButton::new(
                image.read().unwrap().texture_id(ctx),
                [thumbnail_size as f32, thumbnail_size as f32],
            ));
        } else {
            let image = utils::sample_image(Color32::TRANSPARENT, thumbnail_size, thumbnail_size);
            ui.add(ImageButton::new(image.texture_id(ctx), [thumbnail_size as f32, thumbnail_size as f32]));
        }
    }

    pub fn up_next(&self, ctx: &Context, ui: &mut Ui) {
        let thumbnail_size: usize = Self::up_next_thumbnail_size as usize;
        let queue_item = self
            .player
            .read()
            .map(|player| player.next_queue_item())
            .unwrap_or(None);
        let track_title = queue_item
            .as_ref()
            .map_or("".to_string(), |qi| qi.track.title.clone());
        // let release_title = queue_item.as_ref()
        //     .map_or("".to_string(), |qi| qi.release.title.clone());
        let artist_name = queue_item
            .as_ref()
            .map_or("".to_string(), |qi| qi.release.artist());
        let texture_id = queue_item.as_ref().map_or(
            utils::sample_image(Color32::TRANSPARENT, thumbnail_size, thumbnail_size).texture_id(&ctx),
            |qi| {
                let image = self.retained_images.get(qi.release.art.first().unwrap(), thumbnail_size, thumbnail_size);
                let texture_id = image.read().unwrap().texture_id(ctx);
                texture_id
            },
        );

        ui.vertical_centered(|ui| {
            ui.set_width(Self::up_next_width);
            ui.label(Theme::small("Up Next").weak());
            ui.add(ImageButton::new(texture_id, [thumbnail_size as f32, thumbnail_size as f32]));
            ui.label(Theme::small_n_bold(&track_title));
            ui.small(artist_name);
        });
    }

    pub fn fav_icon_label(
        &self,
        icon: &RetainedImage,
        label: &str,
        is_fav: bool,
        ctx: &Context,
        ui: &mut Ui,
    ) {
        ui.horizontal(|ui| {
            ui.image(icon.texture_id(ctx), [21.0, 21.0]);
            ui.label(Theme::bigger(label));
        });
    }

    pub fn play_controls(&self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let previous_button = ImageButton::new(self.previous_icon.texture_id(ctx), [42.0, 42.0]).frame(false);
            let play_pause_button = ImageButton::new(self.play_icon.texture_id(ctx), [48.0, 48.0]).frame(false);
            let next_button = ImageButton::new(self.next_icon.texture_id(ctx), [42.0, 42.0]).frame(false);
            // The button order is inverted because the parent UI is right to 
            // left so that the player controls are right justified. Don't @ me.
            if ui.add(next_button).clicked() {
                self.player.write().unwrap().next();
            }
            if ui.add(play_pause_button).clicked() {
                self.player.write().unwrap().play();
            }
            if ui.add(previous_button).clicked() {
                self.player.write().unwrap().previous();
            }
        });
    }

    fn split_seconds(seconds: f32) -> (u32, u32, u32) {
        let total_seconds = seconds as u32;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        let tenths_of_second = ((seconds as f32 - total_seconds as f32) * 10.0).round() as u32;
        (minutes, seconds, tenths_of_second)
    }

    pub fn timers(&self, _ctx: &Context, ui: &mut Ui) {
        let position = Self::split_seconds(self.player.read().unwrap().position());
        let duration = Self::split_seconds(self.player.read().unwrap().duration());
        ui.horizontal(|ui| {
            ui.small(format!(
                "{:02}:{:02}",
                position.0, position.1,
            ));
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                ui.small(format!(
                    "{:02}:{:02}",
                    duration.0, duration.1,
                ));
            });
        });
    }
}
