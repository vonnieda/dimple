use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use dimple_core::model::{Model, Dimage};
use dimple_librarian::librarian::Librarian;
use dimple_core::db::Db;
use fast_image_resize::ImageView;
use fast_image_resize::Resizer;
use image::DynamicImage;
use slint::Image;
use slint::Weak;
use slint::{Rgba8Pixel, SharedPixelBuffer};
use threadpool::ThreadPool;
use tiny_skia::Rect;
use crate::ui::AppWindow;

use super::image_gen::gen_fuzzy_circles;
use super::image_gen::gen_fuzzy_rects;

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
    default_release: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    default_genre: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    default_other: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    threadpool: ThreadPool,
}

impl ImageMangler {
    pub fn new(librarian: Librarian, ui: Weak<AppWindow>) -> Self {
        let images = Self {
            ui,
            librarian: librarian.clone(),
            cache: Default::default(),
            default_artist: Self::load_default_image("images/artist_placeholder.png"),
            default_release_group: Self::load_default_image("images/release_group_placeholder.png"),
            default_release: Self::load_default_image("images/release_placeholder.png"),
            default_genre: Arc::new(Mutex::new(dynamic_to_buffer(&gen_fuzzy_circles(128, 128)))),
            default_other: Arc::new(Mutex::new(dynamic_to_buffer(&gen_fuzzy_rects(128, 128)))),
            threadpool: ThreadPool::default(),
        };

        images
    }

    fn load_default_image(path: &str) -> Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>> {
        let image = image::open(path).ok().unwrap();
        Arc::new(Mutex::new(dynamic_to_buffer(&image)))
    }

    pub fn cache_len(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    pub fn get(&self, model: Model, width: u32, height: u32) -> slint::Image {
        let entity = model.entity();
        let cache_key = format!("{}:{}:{}:{}", 
            entity.type_name(), entity.key().unwrap(), width, height);
        if let Some(buffer) = self.cache.lock().unwrap().get(&cache_key) {
            return Image::from_rgba8_premultiplied(buffer.clone())
        }
        // TODO DRY(osdfhaosdhfoiuaysdiofuhaoisfd)
        if let Some(dyn_image) = self.librarian.image(&model) {
            let dyn_image = resize(dyn_image, width, height);
            let buffer = dynamic_to_buffer(&dyn_image);
            self.cache.lock().unwrap().insert(cache_key, buffer.clone());
            return Image::from_rgba8_premultiplied(buffer)
        }
        Image::from_rgba8_premultiplied(self.default_model_image(&model))
    }

    pub fn lazy_get<F>(&self, model: Model, width: u32, height: u32, set_image: F) -> slint::Image
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
                if let Some(dyn_image) = images.librarian.image(&model) {
                    let dyn_image = resize(dyn_image, width, height);
                    let buffer = dynamic_to_buffer(&dyn_image);
                    images.cache.lock().unwrap().insert(cache_key, buffer.clone());
                    ui.upgrade_in_event_loop(move |ui| {
                        let image = Image::from_rgba8_premultiplied(buffer);
                        set_image(ui, image);
                    }).unwrap();                    
                }
            });
        }
        Image::from_rgba8_premultiplied(self.default_model_image(&model))
    }

    pub fn default_model_image(&self, model: &Model) -> SharedPixelBuffer<Rgba8Pixel> {
        match model {
            Model::Artist(_) => return self.default_artist.lock().unwrap().clone(),
            Model::ReleaseGroup(_) => return self.default_release_group.lock().unwrap().clone(),
            Model::Release(_) => return self.default_release.lock().unwrap().clone(),
            Model::Genre(_) => return self.default_genre.lock().unwrap().clone(),
            _ => return self.default_other.lock().unwrap().clone(),
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

pub fn resize(image: DynamicImage, width: u32, height: u32) -> DynamicImage {
    let src_image = image;

    let mut dst_image = DynamicImage::new(width, height, 
        src_image.color());

    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None).unwrap();

    dst_image
}