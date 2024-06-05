use image::DynamicImage;
use tiny_skia::Rect;

pub fn gen_fuzzy_circles(width: u32, height: u32) -> DynamicImage {
    let output_width = width;
    let output_height = height;
    let width = 128;
    let height = 128;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
    let mut paint = tiny_skia::Paint::default();
    for i in 0..50 {
        paint.set_color_rgba8(
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
        );
        let circle = tiny_skia::PathBuilder::from_circle(
            fakeit::misc::random(0., width as f32), 
            fakeit::misc::random(0., height as f32), 
            fakeit::misc::random(2., width as f32 / 3.), 
        ).unwrap();
        pixmap.fill_path(&circle, &paint, tiny_skia::FillRule::Winding, Default::default(), None);
    }

    let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_raw(width, height, pixmap.data().to_vec()).unwrap();
    let image = image::imageops::blur(&image, 9.0);

    let dyn_image = DynamicImage::ImageRgba8(image);
    dyn_image.resize(output_width, output_height, image::imageops::FilterType::Nearest)
}

pub fn gen_fuzzy_rects(width: u32, height: u32) -> DynamicImage {
    let output_width = width;
    let output_height = height;
    let width = 128;
    let height = 128;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
    let mut paint = tiny_skia::Paint::default();
    for i in 0..50 {
        paint.set_color_rgba8(
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
        );
        let rect = tiny_skia::PathBuilder::from_rect(Rect::from_xywh(
            fakeit::misc::random(0., width as f32), 
            fakeit::misc::random(0., height as f32), 
            fakeit::misc::random(2., width as f32 / 3.), 
            fakeit::misc::random(2., height as f32 / 3.), 
        ).unwrap());
        pixmap.fill_path(&rect, &paint, tiny_skia::FillRule::Winding, Default::default(), None);
    }

    let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_raw(width, height, pixmap.data().to_vec()).unwrap();
    let image = image::imageops::blur(&image, 7.0);
    
    let dyn_image = DynamicImage::ImageRgba8(image);
    dyn_image.resize(output_width, output_height, image::imageops::FilterType::Nearest)
}

