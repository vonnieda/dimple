use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
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
use slint::Weak;
use slint::{ModelRc, Rgba8Pixel, SharedPixelBuffer};
use threadpool::ThreadPool;
use tiny_skia::Rect;
use crate::ui::AppWindow;
use crate::ui::CardAdapter;


/// Handles image loading, placeholders, caching, scaling, generation, etc.
/// Primary job is to quickly return an image for a Model, and be able to
/// notify the view when a better one is available.
#[derive(Clone)]
pub struct ImageMangler {
    librarian: Librarian,
    cache: Arc<Mutex<HashMap<String, SharedPixelBuffer<Rgba8Pixel>>>>,
    ui: Weak<AppWindow>,
    default_artist: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    default_release_group: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    default_other: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    threadpool: ThreadPool,
}

impl ImageMangler {
    pub fn new(librarian: Librarian, ui: Weak<AppWindow>) -> Self {
        let images = Self {
            ui,
            librarian: librarian.clone(),
            cache: Default::default(),
            default_artist: Arc::new(Mutex::new(dynamic_to_buffer(&gen_fuzzy_circles(128, 128)))),
            default_release_group: Arc::new(Mutex::new(dynamic_to_buffer(&gen_fuzzy_circles(128, 128)))),
            default_other: Arc::new(Mutex::new(dynamic_to_buffer(&gen_fuzzy_rects(128, 128)))),
            threadpool: ThreadPool::new(8),
        };

        images
    }

    pub fn get(&self, model: dimple_core::model::Model, width: u32, height: u32) -> slint::Image {
        let entity = model.entity();
        let cache_key = format!("{}:{}:{}:{}", 
            entity.type_name(), entity.key().unwrap(), width, height);
        if let Some(buffer) = self.cache.lock().unwrap().get(&cache_key) {
            return Image::from_rgba8_premultiplied(buffer.clone())
        }
        // TODO DRY(osdfhaosdhfoiuaysdiofuhaoisfd)
        if let Some(dyn_image) = self.load_model_image(&model) {
            let dyn_image = dyn_image.resize(width, height, 
                image::imageops::FilterType::Nearest);
            let buffer = dynamic_to_buffer(&dyn_image);
            self.cache.lock().unwrap().insert(cache_key, buffer.clone());
            // TODO go to network, or more likely we'll just use a different
            // library request that also goes to the network if needed?
            return Image::from_rgba8_premultiplied(buffer)
        }
        Image::from_rgba8_premultiplied(self.default_model_image(&model))
    }

    pub fn lazy_get<F>(&self, model: dimple_core::model::Model, width: u32, height: u32, set_image: F) -> slint::Image
            where F: Fn(AppWindow, Image) + Send + Copy + 'static {
        let entity = model.entity();
        let cache_key = format!("{}:{}:{}:{}", 
            entity.type_name(), entity.key().unwrap(), width, height);
        if let Some(buffer) = self.cache.lock().unwrap().get(&cache_key) {
            return Image::from_rgba8_premultiplied(buffer.clone())
        }
        {
            let images = self.clone();
            let model = model.clone();
            let ui = self.ui.clone();
            self.threadpool.execute(move || {
                // TODO DRY(osdfhaosdhfoiuaysdiofuhaoisfd)
                if let Some(dyn_image) = images.load_model_image(&model) {
                    // TODO note, this resize is so slow in some cases that it
                    // might be worth doing a way scaled down version first, setting it,
                    // and then doing the real one.
                    let dyn_image = dyn_image.resize(width, height, 
                        image::imageops::FilterType::Nearest);
                    let buffer = dynamic_to_buffer(&dyn_image);
                    images.cache.lock().unwrap().insert(cache_key, buffer.clone());
                    // TODO go to network?
                    ui.upgrade_in_event_loop(move |ui| {
                        let image = Image::from_rgba8_premultiplied(buffer);
                        set_image(ui, image);
                    }).unwrap();                    
                }
            });
        }
        Image::from_rgba8_premultiplied(self.default_model_image(&model))
    }

    pub fn load_model_image(&self, model: &Model) -> Option<DynamicImage> {
        let picture = self.librarian.list(&Picture::default().into(), Some(&model))
            .unwrap()
            .map(Into::<Picture>::into)
            .next();
        match picture {
            Some(picture) => Some(picture.get_image()),
            None => None,
        }
    }

    pub fn default_model_image(&self, model: &Model) -> SharedPixelBuffer<Rgba8Pixel> {
        // let image = image::open("images/light.png").expect("Error loading demo image").into_rgba8();

        match model {
            Model::Artist(_) => return self.default_artist.lock().unwrap().clone(),
            Model::ReleaseGroup(_) => return self.default_release_group.lock().unwrap().clone(),
            _ => return self.default_other.lock().unwrap().clone(),
        }
    }
}

//             let mut demo_image = image::open("images/light.png").expect("Error loading demo image").into_rgba8();

//             image::imageops::colorops::brighten_in_place(&mut demo_image, 20);
            
//             let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
//                 demo_image.as_raw(),
//                 demo_image.width(),
//                 demo_image.height(),
//             );
//             let image = Image::from_rgba8(buffer);

//             for (i, card) in adapter.cards.iter().enumerate() {
//                 let mut card = card.clone();
//                 card.image.image = image.clone();
//                 card.title.name = "Wow".to_string().into();
//                 adapter.cards.set_row_data(i, card);
//             }


pub fn dynamic_to_buffer(dynamic_image: &DynamicImage) -> SharedPixelBuffer<Rgba8Pixel> {
    let rgba8_image = dynamic_image.clone().into_rgba8();
    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba8_image.as_raw(),
        rgba8_image.width(),
        rgba8_image.height(),
    )
}

pub fn gen_fuzzy_circles(width: u32, height: u32) -> DynamicImage {
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

    DynamicImage::ImageRgba8(image)
}

pub fn gen_fuzzy_rects(width: u32, height: u32) -> DynamicImage {
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
    let image = image::imageops::blur(&image, 9.0);

    DynamicImage::ImageRgba8(image)
}
