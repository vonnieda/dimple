use std::sync::Arc;

use eframe::egui::{self, Context, ImageButton, Ui, Image};
use eframe::epaint::{ColorImage, Color32};
use eframe::glow::Texture;
use egui_extras::RetainedImage;

use crate::music_library::{Release, Track};
use crate::player::{PlayerHandle, QueueItem};

use super::retained_images::RetainedImages;
use super::scrubber::{PlotScrubber, SliderScrubber};

#[derive()]
pub struct PlayerBar {
    player: PlayerHandle,
    plot_scrubber: PlotScrubber,
    slider_scrubber: SliderScrubber,
    favorite_icon: RetainedImage,
    favorite_icon_filled: RetainedImage,
    retained_images: Arc<RetainedImages>,
}

impl PlayerBar {
    pub fn new(player: PlayerHandle, retained_images: Arc<RetainedImages>) -> Self {
        Self {
            player,
            plot_scrubber: PlotScrubber::default(),
            slider_scrubber: SliderScrubber::default(),
            retained_images: retained_images.clone(),

            favorite_icon: RetainedImage::from_svg_bytes("", 
                include_bytes!("../icons/material/favorite_FILL0_wght700_GRAD0_opsz48.svg")).unwrap(),
            favorite_icon_filled: RetainedImage::from_svg_bytes("", 
                include_bytes!("../icons/material/favorite_FILL1_wght700_GRAD0_opsz48.svg")).unwrap(),
        }
    }

    pub fn ui(&self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.now_playing(ctx, ui);
            ui.vertical(|ui| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        let queue_item = self.player.read()
                            .map(|player| player.current_queue_item())
                            .unwrap_or(None);
                        let track_title = queue_item.as_ref()
                            .map_or("".to_string(), |qi| qi.track.title.clone());
                        let release_title = queue_item.as_ref()
                            .map_or("".to_string(), |qi| qi.release.title.clone());
                        let artist_name = queue_item.as_ref()
                            .map_or("".to_string(), |qi| qi.release.artist());
                
                        self.fav_icon_label(&Self::sample_image(Color32::LIGHT_BLUE, 120, 120), &track_title, false, ctx, ui);
                        self.fav_icon_label(&Self::sample_image(Color32::LIGHT_GREEN, 120, 120), &release_title, false, ctx, ui);
                        self.fav_icon_label(&Self::sample_image(Color32::LIGHT_YELLOW, 120, 120), &artist_name, false, ctx, ui);
                    });
                    self.play_controls(ctx, ui);
                });
                self.plot_scrubber.ui(ctx, ui);
                self.slider_scrubber.ui(ctx, ui);
                self.timers("07:45", "13:16", ctx, ui);
            });
            self.up_next(ctx, ui);
        });
    }

    pub fn now_playing(&self, ctx: &Context, ui: &mut Ui) {
        if let Some(item) = self.player.read().unwrap().current_queue_item() {
            let image = self.retained_images.retained_image(item.release.art.first().unwrap(), 120, 120);
            ui.add(ImageButton::new(image.read().unwrap().texture_id(ctx), [120.0, 120.0]));
        }
        else {
            let image = Self::sample_image(Color32::RED, 120, 120);
            ui.add(ImageButton::new(
                image.texture_id(ctx), [120.0, 120.0]));    
        }
    }
    
    pub fn up_next(&self, ctx: &Context, ui: &mut Ui) {
        let queue_item = self.player.read()
            .map(|player| player.next_queue_item())
            .unwrap_or(None);
        let track_title = queue_item.as_ref()
            .map_or("".to_string(), |qi| qi.track.title.clone());
        // let release_title = queue_item.as_ref()
        //     .map_or("".to_string(), |qi| qi.release.title.clone());
        let artist_name = queue_item.as_ref()
            .map_or("".to_string(), |qi| qi.release.artist());
        let texture_id = queue_item.as_ref()
            .map_or(Self::sample_image(Color32::RED, 60, 60).texture_id(&ctx), |qi| {
                let image = self.retained_images.retained_image(qi.release.art.first().unwrap(), 60, 60);
                let texture_id = image.read().unwrap().texture_id(ctx);
                texture_id
            });
               

        ui.vertical_centered(|ui| {
            ui.label("Up Next");
            ui.add(ImageButton::new(texture_id, [60.0, 60.0]));
            ui.label(track_title);
            ui.label(artist_name);
        });
    }    

    pub fn fav_icon_label(&self, icon: &RetainedImage, label: &str,
        is_fav: bool,ctx: &Context, ui: &mut Ui) {
        
        ui.horizontal(|ui| {
            let fav_icon = if is_fav { &self.favorite_icon_filled } else { &self.favorite_icon };
            ui.image(fav_icon.texture_id(ctx), [20.0, 20.0]);
            ui.image(icon.texture_id(ctx), [20.0, 20.0]);
            ui.label(label);
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

    pub fn timers(&self, left: &str, right: &str, ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(left);
            // TODO magic
            ui.label(right);
        });
    }    

    pub fn sample_image(color: Color32, width: usize, height: usize) -> RetainedImage {
        RetainedImage::from_color_image("", ColorImage::new([width, height], color))
    }

    // ui.horizontal(|ui| {
    //     if ui.button("List Queue").clicked() {
    //         for (i, track) in self.player.read().unwrap().tracks().iter().enumerate() {
    //             log::info!("{}. {}", 
    //                 i + 1, 
    //                 track.title);
    //         }
    //     }
    //     if ui.button("Clear Queue").clicked() {
    //         self.player.write().unwrap().clear();
    //     }
    // });

}

