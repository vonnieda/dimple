use crate::ui::AppWindow;
use crate::ui::FlowLayoutResults;
use crate::ui::FlowLayoutLogic;
use slint::ComponentHandle as _;
use slint::Model as _;

pub fn init(ui: &AppWindow) {
    ui.global::<FlowLayoutLogic>().on_calculate_positions(move |width, num_elements, spacing, preferred_widths, preferred_heights| {
        if num_elements < 1 {
            return FlowLayoutResults::default()
        }
        let mut ret_widths = vec![];
        let mut ret_heights = vec![];
        let mut ret_xs = vec![];
        let mut ret_ys = vec![];
        let mut x = 0.;
        let mut y = 0.;
        let row_height = preferred_heights.iter().fold(0., |a: f32, h| a.max(h));
        for i in 0..num_elements {
            let w = preferred_widths.row_data(i as usize).unwrap_or(1.0);
            let h = preferred_heights.row_data(i as usize).unwrap_or(1.0);
            if x + w >= width {
                x = 0.;
                // TODO this should actually just be the largest for the row,
                // but then we need to pre-calc.
                y += row_height + spacing;
            }
            ret_widths.push(w);
            ret_heights.push(h);
            ret_xs.push(x); 
            ret_ys.push(y);
            x += w + spacing;
        }
        FlowLayoutResults {
            xs: ret_xs.as_slice().into(),
            ys: ret_ys.as_slice().into(),
            widths: ret_widths.as_slice().into(),
            heights: ret_heights.as_slice().into(),
            height: y + row_height + row_height + spacing,
        }
    });
}

