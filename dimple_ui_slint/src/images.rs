use image::DynamicImage;
use slint::{SharedPixelBuffer, Rgb8Pixel};

pub fn generate_abstract_image() -> DynamicImage {
    let img = DynamicImage::new_rgb8(500, 500);
    img
}

pub fn dynamic_image_to_slint_image(dynamic_image: &DynamicImage) -> slint::Image {
    // TODO may be possible to limit the number of copies here
    let rgb8_image = dynamic_image.clone().into_rgb8();
    let shared_pixbuf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        rgb8_image.as_raw(),
        rgb8_image.width(),
        rgb8_image.height(),
    );
    slint::Image::from_rgb8(shared_pixbuf)
}
