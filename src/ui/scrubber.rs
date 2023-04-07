use eframe::egui::{Context, Ui, Slider, plot::{PlotPoints, Line, Plot}};

#[derive(Default)]
pub struct PlotScrubber {
}

impl PlotScrubber {
    pub fn ui(&self, _ctx: &Context, ui: &mut Ui) {
        let sin: PlotPoints = (0..1000).map(|i| {
            let x = i as f64 * 0.01;
            [x, x.sin()]
        }).collect();
        let line = Line::new(sin);
        Plot::new("my_plot")
            .view_aspect(1.0)
            .show(ui, |plot_ui| plot_ui.line(line));
    }
}

#[derive(Default)]
pub struct SliderScrubber {
}

impl SliderScrubber {
    pub fn ui(&self, _ctx: &Context, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            let mut my_f32: f32 = 0.33;
            ui.add(
                Slider::new(&mut my_f32, 0.0..=1.0)
                    .show_value(false)
                    .trailing_fill(true),
            );
        });
    }
}
