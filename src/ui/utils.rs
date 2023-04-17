use eframe::epaint::{ColorImage, Color32};
use egui_extras::RetainedImage;
use image::DynamicImage;

// TODO take a look at how RetainedImage does it's loading and see if I can
// optimize or remove this.
pub fn dynamic_to_retained(debug_name: &str, image: &DynamicImage) -> RetainedImage {
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    RetainedImage::from_color_image(debug_name, color)
}

pub fn sample_image(color: Color32, width: usize, height: usize) -> RetainedImage {
    RetainedImage::from_color_image("", ColorImage::new([width, height], color))
}

