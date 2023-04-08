use eframe::egui::{self, Context, Ui, ScrollArea, Grid};

pub trait Card {
    fn ui(&self, image_width: f32, image_height: f32, 
        ctx: &Context, ui: &mut Ui);
}

#[derive(Default)]
pub struct CardGrid {
}

// TODO use ScrollArea::show_rows to improve performance. I
// tried previously and I couldn't get the rendering right.
// Oh, a hint, might also need Grid::show_rows

impl CardGrid {
    pub fn ui(&self, cards: &[Box<dyn Card>], image_width: f32, 
        image_height: f32, ctx: &Context, ui: &mut Ui) {
        
        let max_width = ui.available_width();
        let padding = 16.0;
        let cards_max_width = max_width - (padding * 2.0);
        let min_card_spacing = 16.0;
        let card_frame_width = 4.0;
        let card_width = card_frame_width + image_width + card_frame_width + min_card_spacing;
        let num_columns: usize = (cards_max_width / card_width).floor() as usize;
        let cards_width = card_width * num_columns as f32;
        let addl_card_spacing = (cards_max_width - cards_width) / num_columns as f32;
        let card_spacing = min_card_spacing + addl_card_spacing;
        let row_spacing = 16.0;
        // log::info!("{} {} {}",s max_width, num_columns, cards_width);

        ui.spacing_mut().scroll_bar_width = 16.0;
        ui.spacing_mut().scroll_handle_min_length = 24.0;
        // ui.spacing_mut().scroll_bar_outer_margin = 10.0;
        // ui.spacing_mut().scroll_bar_inner_margin = 10.0;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, move |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(padding);
                    Grid::new("CardGrid")
                        .spacing(egui::vec2(card_spacing, row_spacing))
                        .show(ui, |ui| {
                            for (i, card) in cards.iter().enumerate() {
                                card.ui(image_width, image_height, ctx, ui);
                                if i % num_columns == num_columns - 1 {
                                    ui.end_row();
                                }
                            }
                    });
                });
            });
        }
}

