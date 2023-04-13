use std::f64::consts::PI;

use eframe::{egui::{Context, Ui, Slider, plot::{PlotPoints, Line, Plot}}, epaint::{Color32, Stroke}};

use crate::{player::PlayerHandle, dimple::Theme};

#[derive(Default)]
pub struct PlotScrubber {
}

impl PlotScrubber {
    pub fn ui(&self, _ctx: &Context, ui: &mut Ui) {
        let points: PlotPoints = (0..1000).map(|x| {
            let x = x as f64;
            let y = f(x).powf(2.0);
            [x, y]
        }).collect();
        let line = Line::new(points)
            .fill(0.0)
            .color(Theme::background_top)
            // .stroke(Stroke::new(3.0, Theme::background_top))
        ;
        Plot::new("my_plot")
            .height(20.0)
            // .width(ui.available_width() * 0.8)
            .show_x(true)
            .show_y(false)
            .show_background(false)
            .show_axes([false; 2])
            .allow_boxed_zoom(false)
            .allow_double_click_reset(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .set_margin_fraction([0.0, 0.0].into())
            .show(ui, |plot_ui| plot_ui.line(line));
    }
}

fn f(x: f64) -> f64 {
    let m = 0.0; // slope of the linear function
    let a1 = 5.0; // amplitude of the first sinusoidal bump
    let w1 = 1.0 / 30.0; // frequency of the first sinusoidal bump
    let phi1 = std::f64::consts::FRAC_PI_4; // phase of the first sinusoidal bump
    let a2 = 3.0; // amplitude of the second sinusoidal bump
    let w2 = 1.0 / 20.0; // frequency of the second sinusoidal bump
    let phi2 = -std::f64::consts::FRAC_PI_3; // phase of the second sinusoidal bump
    let a3 = 2.0; // amplitude of the third sinusoidal bump
    let w3 = 1.0 / 10.0; // frequency of the third sinusoidal bump
    let phi3 = std::f64::consts::FRAC_PI_2; // phase of the third sinusoidal bump

    m * x + a1 * (w1 * x + phi1).sin() + a2 * (w2 * x + phi2).sin() + a3 * (w3 * x + phi3).sin()
}

#[derive(Default)]
pub struct SliderScrubber {
}

impl SliderScrubber {
    pub fn ui(&self, player: PlayerHandle, _ctx: &Context, ui: &mut Ui) {
        let position = player.read().unwrap().position();
        let duration = player.read().unwrap().duration();
        let mut mut_position: f32 = position;
        let slider = Slider::new(&mut mut_position, 0.0..=duration)
            .show_value(false)
            .trailing_fill(true);
        ui.spacing_mut().slider_width = ui.available_width();
        if ui.add(slider).changed() {
            player.read().unwrap().seek(mut_position);
        }
    }
}
