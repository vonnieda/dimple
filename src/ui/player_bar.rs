use eframe::egui::{self, Context, ImageButton, Ui};
use eframe::epaint::{ColorImage};
use egui_extras::RetainedImage;

use crate::player::PlayerHandle;

use super::scrubber::{PlotScrubber, SliderScrubber};

#[derive()]
pub struct PlayerBar {
    player: PlayerHandle,
    _plot_scrubber: PlotScrubber,
    slider_scrubber: SliderScrubber,
}

impl PlayerBar {
    pub fn new(player: PlayerHandle) -> Self {
        Self {
            player,
            _plot_scrubber: PlotScrubber::default(),
            slider_scrubber: SliderScrubber::default(),
        }
    }

    pub fn ui(&self, ctx: &Context, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.horizontal(|ui| {
                let track = self.player.read().unwrap().current_track();
                let image = RetainedImage::from_color_image("default", ColorImage::example());
                let title = track.map_or("".to_string(), |track| track.title);
                // TODO till we know what album it's from
                let subtitle = title.clone();

                ui.add(ImageButton::new(
                    image.texture_id(ctx),
                    egui::vec2(120.0, 120.0),
                ));
                ui.vertical(|ui| {
                    ui.link(&title).clicked();
                    ui.link(&subtitle).clicked();
                    // self.plot_scrubber.ui(ctx, ui);
                    self.slider_scrubber.ui(ctx, ui);
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
                    ui.horizontal(|ui| {
                        if ui.button("List Queue").clicked() {
                            for (i, track) in self.player.read().unwrap().tracks().iter().enumerate() {
                                log::info!("{}. {}", 
                                    i + 1, 
                                    track.title);
                            }
                        }
                        if ui.button("Clear Queue").clicked() {
                            self.player.write().unwrap().clear();
                        }
                    });
                });
                // self.card(&self.up_next, 60.0, 60.0, ctx, ui);
            });
        });        
    }
}

