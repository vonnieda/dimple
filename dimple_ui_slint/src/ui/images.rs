use dimple_core::model::{Model, Picture};
use dimple_librarian::librarian::Librarian;
use dimple_core::db::Db;
use image::DynamicImage;
use slint::{Rgba8Pixel, SharedPixelBuffer};

pub fn get_model_image(librarian: &Librarian, model: &Model, width: u32, height: u32) -> SharedPixelBuffer<Rgba8Pixel> {
    let picture = librarian.list(&Picture::default().into(), Some(&model))
        .unwrap()
        .next();
    let picture: Picture = match picture {
        Some(art) => art,
        None => {
            let image = create_model_image(librarian, model, width, height);
            let mut picture = Picture::default();
            picture.set_image(&image);
            let picture = librarian.insert(&picture.into()).unwrap();
            librarian.link(&picture.clone().into(), &model).unwrap();
            picture
        }
    }.into();
    let dyn_image = picture.get_image();
    let image_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        dyn_image.as_bytes(),
        dyn_image.width(),
        dyn_image.height(),
    );
    image_buf
}

pub fn create_model_image(librarian: &Librarian, model: &Model, width: u32, height: u32) -> DynamicImage {
    match model {
        Model::Artist(value) => {
            fuzzy_circles(width, height)
        },
        _ => fuzzy_circles(width, height)
    }
}

pub fn fuzzy_circles(width: u32, height: u32) -> DynamicImage {
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
    for i in 0..50 {
        let mut paint = tiny_skia::Paint::default();
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

    DynamicImage::ImageRgba8(image)
}

