use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::{Model, Picture};
use dimple_librarian::librarian;
use dimple_librarian::librarian::Librarian;
use dimple_core::db::Db;
use image::DynamicImage;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use slint::Image;
use slint::{ModelRc, Rgba8Pixel, SharedPixelBuffer};
use crate::ui::AppWindow;
use crate::ui::CardAdapter;

pub fn get_model_image(librarian: &Librarian, model: &Model, width: u32, height: u32) -> SharedPixelBuffer<Rgba8Pixel> {
    let picture = librarian.list(&Picture::default().into(), Some(&model))
        .unwrap()
        .next();
    let picture: Picture = match picture {
        Some(picture) => picture,
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
type LazyImage = (usize, SharedPixelBuffer<Rgba8Pixel>);

pub fn lazy_load_images<F>(librarian: &Librarian, 
    models: &[dimple_core::model::Model], 
    ui: slint::Weak<AppWindow>, 
    get_model: F) 
    where F: Fn(AppWindow) -> ModelRc<CardAdapter> + Send + Copy + 'static {

    let models: Vec<_> = models.iter().cloned().collect();
    let librarian = librarian.clone();
    let (sender, receiver) = channel::<LazyImage>();

    thread::spawn(move || {
        models.iter().enumerate().par_bridge().for_each(|(index, model)| {
            let buffer = get_model_image(&librarian, model, 200, 200);
            sender.send((index, buffer)).unwrap();
        });
    });

    thread::spawn(move || {        
        let mut queue: VecDeque<LazyImage> = VecDeque::new();
        let mut last_send = Instant::now();
        for lazy_image in receiver.iter() {
            let index = lazy_image.0;
            let buffer = lazy_image.1;
            queue.push_back((index, buffer));
            if last_send.elapsed() > Duration::from_millis(250) {
                last_send = Instant::now();
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

/// Handles image loading, placeholders, caching, scaling, generation, etc.
/// Primary job is to quickly return an image for a Model, and be able to
/// notify the view when a better one is available.
#[derive(Clone)]
pub struct ImageMangler {
    librarian: Librarian,
    cache: Arc<Mutex<HashMap<String, SharedPixelBuffer<Rgba8Pixel>>>>,
    // queue: Arc<Mutex<VecDeque<QueueItem>>>,
    default_artist: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    // default_release_group: SharedPixelBuffer<Rgba8Pixel>,
    default_other: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
}

impl ImageMangler {
    pub fn new(librarian: Librarian) -> Self {
        let images = Self {
            librarian: librarian.clone(),
            cache: Default::default(),
            // queue: Default::default(),
            default_artist: Arc::new(Mutex::new(dynamic_to_buffer(&fuzzy_circles(128, 128)))),
            // default_release_group: dynamic_to_buffer(&fuzzy_circles(1000, 1000)),
            default_other: Arc::new(Mutex::new(dynamic_to_buffer(&fuzzy_circles(128, 128)))),
        };
        // {
        //     let images = images.clone();
        //     let librarian = librarian.clone();
        //     thread::spawn(move || {
        //         log::info!("Preloading artist images");
        //         let artists: Vec<Model> = librarian.list(&Artist::default().model(), None).unwrap().collect();
        //         artists.par_iter().for_each(|artist| {
        //             let key = Self::model_key(artist);
        //             let image = images.get(&artist);
        //             images.cache.lock().unwrap().insert(key, image);
        //         });
        //         log::info!("Done preloading artist images");
        //     });
        // }
        images
    }

    pub fn get<F>(&self, model: dimple_core::model::Model, width: u32, height: u32, set_image: F) -> SharedPixelBuffer<Rgba8Pixel> 
        where F: Fn(AppWindow, Image) {
        // If an image is already cached at the correct size, return it. All
        // other methods are deferred and go into the queue.
        // let key = format!("{}:{}:{}", model.key(), width, height);
        let model_key = match &model {
            Model::Artist(v) => v.key.clone().unwrap(),
            Model::ReleaseGroup(v) => v.key.clone().unwrap(),
            _ => todo!(),
        };
        let cache_key = format!("{}:{}:{}", model_key, width, height);
        if let Some(buffer) = self.cache.lock().unwrap().get(&cache_key) {
            return buffer.clone()
        }
        // TODO queue the placeholder replacement
        match &model {
            Model::Artist(_v) => return self.default_artist.lock().unwrap().clone(),
            // Model::ReleaseGroup(_v) => return self.default_artist.clone(),
            _ => return self.default_other.lock().unwrap().clone(),
        }
    }

    fn model_key(model: &Model) -> String {
        match model {
            Model::Artist(a) => format!("artist:{}", a.key.clone().unwrap()),
            Model::Genre(_) => todo!(),
            Model::MediaFile(_) => todo!(),
            Model::Medium(_) => todo!(),
            Model::Recording(_) => todo!(),
            Model::RecordingSource(_) => todo!(),
            Model::ReleaseGroup(_) => todo!(),
            Model::Release(_) => todo!(),
            Model::Track(_) => todo!(),
            Model::Picture(_) => todo!(),
        }
    }
}

pub fn dynamic_to_buffer(dynamic_image: &DynamicImage) -> SharedPixelBuffer<Rgba8Pixel> {
    let rgba8_image = dynamic_image.clone().into_rgba8();
    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba8_image.as_raw(),
        rgba8_image.width(),
        rgba8_image.height(),
    )
}
