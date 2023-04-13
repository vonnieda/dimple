use std::sync::Arc;

use eframe::egui::{Context, ImageButton, Ui, Layout};
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

    up_next_width: f32,
}

impl PlayerBar {
    pub fn new(player: PlayerHandle, retained_images: Arc<RetainedImages>) -> Self {
        Self {
            player,
            plot_scrubber: PlotScrubber::default(),
            slider_scrubber: SliderScrubber::default(),
            retained_images,

            artist_icon: Self::svg_icon(include_bytes!("../icons/material/group_FILL0_wght400_GRAD0_opsz48.svg")),
            release_icon: Self::svg_icon(include_bytes!("../icons/material/album_FILL0_wght400_GRAD0_opsz48.svg")),
            track_icon: Self::svg_icon(include_bytes!("../icons/material/music_note_FILL0_wght400_GRAD0_opsz48.svg")),
            play_icon: Self::svg_icon(include_bytes!("../icons/material/play_circle_FILL0_wght400_GRAD0_opsz48.svg")),
            pause_icon: Self::svg_icon(include_bytes!("../icons/material/pause_circle_FILL0_wght400_GRAD0_opsz48.svg")),
            next_icon: Self::svg_icon(include_bytes!("../icons/material/skip_next_FILL0_wght400_GRAD0_opsz48.svg")),
            previous_icon: Self::svg_icon(include_bytes!("../icons/material/skip_previous_FILL0_wght400_GRAD0_opsz48.svg")),

            up_next_width: 88.0,
        }
    }

    pub fn svg_icon(bytes: &[u8]) -> RetainedImage {
        RetainedImage::from_svg_bytes("", bytes).unwrap()
    }

    pub fn ui(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            ui.add_space(16.0);
            ui.vertical(|ui| {
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    self.now_playing(ctx, ui);
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.set_max_width(ui.available_width() 
                            - self.up_next_width 
                            - ui.spacing().item_spacing.x 
                            - 16.0);
                        self.track_info(ctx, ui);
                        ui.scope(|ui| {
                            ui.spacing_mut().item_spacing = [0.0, 0.0].into();
                            self.plot_scrubber.ui(ctx, ui);
                            self.slider_scrubber.ui(self.player.clone(), ctx, ui);
                        });
                        self.timers(ctx, ui);
                    });
                    ui.add_space(4.0);
                    self.up_next_width = ui.scope(|ui| {
                        self.up_next(ctx, ui);
                    }).response.rect.width();
                });
            });
            ui.add_space(16.0);
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
                .map_or("Track Title".to_string(), |qi| qi.track.title.clone());
            let release_title = queue_item
                .as_ref()
                .map_or("Release Title".to_string(), |qi| qi.release.title.clone());
            let artist_name = queue_item
                .as_ref()
                .map_or("Artist Name".to_string(), |qi| qi.release.artist());

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
        let thumbnail_size = 120;
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
        let thumbnail_size = 58;
        let queue_item = self
            .player
            .read()
            .map(|player| player.next_queue_item())
            .unwrap_or(None);
        let track_title = queue_item
            .as_ref()
            .map_or("Track Title".to_string(), |qi| qi.track.title.clone());
        // let release_title = queue_item.as_ref()
        //     .map_or("".to_string(), |qi| qi.release.title.clone());
        let artist_name = queue_item
            .as_ref()
            .map_or("Artist Name".to_string(), |qi| qi.release.artist());
        let texture_id = queue_item.as_ref().map_or(
            utils::sample_image(Color32::TRANSPARENT, thumbnail_size, thumbnail_size).texture_id(&ctx),
            |qi| {
                let image = self.retained_images.get(qi.release.art.first().unwrap(), thumbnail_size, thumbnail_size);
                let texture_id = image.read().unwrap().texture_id(ctx);
                texture_id
            },
        );

        ui.vertical(|ui| {
            ui.label("Up Next");
            ui.add(ImageButton::new(texture_id, [thumbnail_size as f32, thumbnail_size as f32]));
            ui.label(track_title);
            ui.label(artist_name);
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
            if ui.button("Previous").clicked() {
                self.player.write().unwrap().previous();
            }
            if ui.button("Play").clicked() {
                self.player.write().unwrap().play();
            }
            if ui.button("Pause").clicked() {
                self.player.read().unwrap().pause();
            }
            if ui.button("Next").clicked() {
                self.player.write().unwrap().next();
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
