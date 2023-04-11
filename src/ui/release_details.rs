use std::sync::Arc;

use eframe::{egui::{Context, Ui, TextEdit, Response}, epaint::{FontId, FontFamily}};

use crate::{player::PlayerHandle, music_library::Release};

use super::retained_images::RetainedImages;

pub struct ReleaseDetails {
}

impl ReleaseDetails {
    pub fn new() -> Self {
        Self {}
    }

    pub fn ui(&mut self, release: &Release, _ctx: &Context, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            ui.heading(release.title.as_str());
        });
    }
}

impl Default for ReleaseDetails {
    fn default() -> Self {
        Self::new()
    }
}

