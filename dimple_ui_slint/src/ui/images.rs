use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use dimple_core::model::{Model, Picture};
use dimple_librarian::librarian::Librarian;
use dimple_core::db::Db;
use image::DynamicImage;
use slint::Image;
use slint::{ModelRc, Rgba8Pixel, SharedPixelBuffer};
use crate::ui::AppWindow;
use crate::ui::CardAdapter;

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

// https://releases.slint.dev/1.5.1/docs/rust/slint/struct.image#sending-image-to-a-thread
// https://github.com/slint-ui/slint/discussions/4289
// https://github.com/slint-ui/slint/discussions/2527
/// Background loads images for the specified slice of Models. Every T = 250ms
/// the UI is notified of any newly loaded images. The images are batched by
/// time as this was shown in testing to keep the UI feeling smooth.
/// TODO This can be further improved with a par_iter sender/receiver setup.
/// And by adding caching back in.
pub fn lazy_load_images<F>(librarian: &Librarian, 
    models: &[dimple_core::model::Model], 
    ui: slint::Weak<AppWindow>, 
    get_model: F) 
    where F: Fn(AppWindow) -> ModelRc<CardAdapter> + Send + Copy + 'static {

    let models: Vec<_> = models.iter().cloned().collect();
    let librarian = librarian.clone();

    thread::spawn(move || {
        let mut queue: VecDeque<(usize, SharedPixelBuffer<Rgba8Pixel>)> = VecDeque::new();
        let mut last_send = Instant::now();
        for (i, model) in models.iter().enumerate() {
            let buffer = get_model_image(&librarian, model, 200, 200);
            queue.push_back((i, buffer));
            if last_send.elapsed() > Duration::from_millis(250) {
                last_send = Instant::now();
                // DRY 1
                let items: Vec<_> = queue.drain(..).collect();
                log::info!("updating batch of {}", items.len());
                ui.upgrade_in_event_loop(move |ui| {
                    let model = get_model(ui);
                    for (index, buffer) in items {
                        let mut card = slint::Model::row_data(&model, index).unwrap();
                        card.image.image = Image::from_rgba8_premultiplied(buffer);
                        slint::Model::set_row_data(&model, index, card);
                    }
                }).unwrap();
            }
        }
        // DRY 1
        let items: Vec<_> = queue.drain(..).collect();
        ui.upgrade_in_event_loop(move |ui| {
            let model = get_model(ui);
            for (index, buffer) in items {
                let mut card = slint::Model::row_data(&model, index).unwrap();
                card.image.image = Image::from_rgba8_premultiplied(buffer);
                slint::Model::set_row_data(&model, index, card);
            }
        }).unwrap();
    });
}

