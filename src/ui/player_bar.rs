use eframe::egui::{Frame, ImageButton, Layout, Margin, Response, Ui};
use eframe::emath::Align;


use egui_extras::RetainedImage;

use crate::player::PlayerHandle;

use super::card_grid::LibraryItem;

use super::scrubber::{PlotScrubber, SliderScrubber};
use super::theme::{Theme, SvgIcon};
use super::utils;

#[derive()]
pub struct PlayerBar {
    player: PlayerHandle,
    plot_scrubber: PlotScrubber,
    slider_scrubber: SliderScrubber,
}

impl PlayerBar {
    const NOW_PLAYING_THUMBNAIL_SIZE: f32 = 150.0;
    const UP_NEXT_WIDTH: f32 = 120.0;
    const UP_NEXT_THUMBNAIL_SIZE: f32 = 80.0;

    pub fn new(player: PlayerHandle) -> Self {
        Self {
            player,
            plot_scrubber: PlotScrubber::default(),
            slider_scrubber: SliderScrubber::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Option<LibraryItem> {
        let mut action = None;
        ui.horizontal_top(|ui| {
            Frame::none()
                .inner_margin(Margin {
                    top: 8.0,
                    left: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                })
                .show(ui, |ui| {
                    if let Some(item) = self.now_playing(ui) {
                        action = Some(item);
                    }
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - Self::UP_NEXT_WIDTH);
                        ui.horizontal_top(|ui| {
                            if let Some(item) = self.track_info(ui) {
                                action = Some(item);
                            }
                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                self.play_controls(ui);
                            });
                        });
                        ui.add_space(8.0);
                        ui.scope(|ui| {
                            // The negative Y spacing slides the plot behind the
                            // handle of the slider and makes it look awesome.
                            ui.spacing_mut().item_spacing = [0.0, -3.0].into();
                            self.plot_scrubber.ui(ui);
                            self.slider_scrubber.ui(self.player.clone(), ui);
                        });
                        self.timers(ui);
                    });
                });
            if let Some(_item) = self.up_next(ui) {
                // TODO for now just overriding all clicks from up next to
                // go to the play queue view.
                action = Some(LibraryItem::Player(self.player.clone()));
                // action = Some(item);
            }
        });

        action
    }

    pub fn track_info(&self, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let queue_item = self.player.read().unwrap().current_queue_item();
        let track_title = queue_item
            .as_ref()
            .map_or(String::new(), |f| f.track.title.clone());
        let release_title = queue_item
            .as_ref()
            .map_or(String::new(), |f| f.release.title.clone());
        let artist_name = queue_item
            .as_ref()
            .map_or(String::new(), |f| f.release.artist());
        let mut action = None;
        ui.vertical(|ui| {
            if self
                .icon_label(&theme.track_icon, &track_title, ui)
                .clicked()
            {
                if let Some(queue_item) = &queue_item {
                    action = Some(LibraryItem::Track(queue_item.track.clone()));
                }
            }

            if self
                .icon_label(&theme.release_icon, &release_title, ui)
                .clicked()
            {
                if let Some(queue_item) = &queue_item {
                    action = Some(LibraryItem::Release(queue_item.release.clone()));
                }
            }

            if self
                .icon_label(&theme.artist_icon, &artist_name, ui)
                .clicked()
            {
                if let Some(queue_item) = &queue_item {
                    action = Some(LibraryItem::Artist(
                        queue_item.release.artists.first().unwrap().clone(),
                    ));
                }
            }
        });
        action
    }

    pub fn now_playing(&self, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let thumbnail_size: usize = Self::NOW_PLAYING_THUMBNAIL_SIZE as usize;
        if let Some(item) = self.player.read().unwrap().current_queue_item() {
            // TODO track art.
            if theme
                .carousel(&item.release.art, thumbnail_size, ui)
                .clicked()
            {
                return Some(LibraryItem::Release(item.release));
            }
        } 
        else {
            let image =
                utils::sample_image(theme.image_placeholder, thumbnail_size, thumbnail_size);
            ui.add(ImageButton::new(
                image.texture_id(ui.ctx()),
                [thumbnail_size as f32, thumbnail_size as f32],
            ));
        }

        None
    }

    pub fn up_next(&self, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let thumbnail_size: usize = Self::UP_NEXT_THUMBNAIL_SIZE as usize;
        let mut action = None;

        let queue_item = self.player.read().unwrap().next_queue_item();
        let release_art = queue_item
            .as_ref()
            .map_or(vec![], |f| f.release.art.clone());
        let track_title = queue_item
            .as_ref()
            .map_or(String::new(), |f| f.track.title.clone());
        let artist_name = queue_item
            .as_ref()
            .map_or(String::new(), |f| f.release.artist());
        ui.vertical_centered(|ui| {
            ui.set_width(Self::UP_NEXT_WIDTH);
            ui.label(Theme::small("Up Next").weak());
            if theme.carousel(&release_art, thumbnail_size, ui).clicked() {
                if let Some(queue_item) = &queue_item {
                    action = Some(LibraryItem::Release(queue_item.release.clone()));
                }
            }
            if ui.link(Theme::small_bold(&track_title)).clicked() {
                if let Some(queue_item) = &queue_item {
                    action = Some(LibraryItem::Track(queue_item.track.clone()));
                }
            }
            if ui.link(Theme::small(&artist_name)).clicked() {
                if let Some(queue_item) = &queue_item {
                    action = Some(LibraryItem::Artist(
                        queue_item.release.artists.first().unwrap().clone(),
                    ));
                }
            }
        });
        action
    }

    pub fn icon_label(&self, icon: &SvgIcon, label: &str, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            Theme::svg_image(icon, 24, 24, ui);
            ui.link(label)
        })
        .inner
    }

    pub fn play_controls(&self, ui: &mut Ui) {
        let theme = Theme::get(ui.ctx());
        ui.horizontal_top(|ui| {
            // The button order is inverted because the parent UI is right to
            // left so that the player controls are right justified. Don't @ me.
            if Theme::svg_button(&theme.next_track_icon, 48, 48, ui).clicked() {
                self.player.write().unwrap().next();
            }
            if self.player.read().unwrap().is_playing() {
                if Theme::svg_button(&theme.pause_icon, 48, 48, ui).clicked() {
                    self.player.write().unwrap().pause();
                }
            }
            else if Theme::svg_button(&theme.play_icon, 48, 48, ui).clicked() {
                self.player.write().unwrap().play();
            }
            if Theme::svg_button(&theme.previous_track_icon, 48, 48, ui).clicked() {
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

    pub fn timers(&self, ui: &mut Ui) {
        let position = Self::split_seconds(self.player.read().unwrap().position());
        let duration = Self::split_seconds(self.player.read().unwrap().duration());
        ui.horizontal(|ui| {
            ui.small(format!("{:02}:{:02}", position.0, position.1,));
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                ui.small(format!("{:02}:{:02}", duration.0, duration.1,));
            });
        });
    }
}
