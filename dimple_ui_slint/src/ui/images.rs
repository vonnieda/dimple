use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;

use dimple_core::librarian::Librarian;
use dimple_core::model::DimpleEntity;
use fast_image_resize::Resizer;
use image::DynamicImage;
use image::ImageFormat;
use slint::Weak;
use slint::{Rgba8Pixel, SharedPixelBuffer};
use crate::ui::AppWindow;

pub fn dynamic_to_buffer(dynamic_image: &DynamicImage) -> SharedPixelBuffer<Rgba8Pixel> {
    // TODO this might be cloning twice.
    let rgba8_image = dynamic_image.clone().into_rgba8();
    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba8_image.as_raw(),
        rgba8_image.width(),
        rgba8_image.height(),
    )
}

pub fn resize(image: DynamicImage, width: u32, height: u32) -> DynamicImage {
    let src_image = image;

    let mut dst_image = DynamicImage::new(width, height, 
        src_image.color());

    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None).unwrap();

    dst_image
}

pub fn dynamic_to_slint(dyn_image: &DynamicImage) -> slint::Image {
    slint::Image::from_rgba8(dynamic_to_buffer(dyn_image))
}

