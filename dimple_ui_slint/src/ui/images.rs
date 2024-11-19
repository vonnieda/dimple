use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;

use dimple_core::library::Library;
use dimple_core::model::Model;
use fast_image_resize::Resizer;
use image::DynamicImage;
use image::ImageFormat;
use slint::Image;
use slint::Weak;
use slint::{Rgba8Pixel, SharedPixelBuffer};
use threadpool::ThreadPool;
use crate::ui::AppWindow;

use super::image_gen::gen_fuzzy_circles;
use super::image_gen::gen_fuzzy_rects;

/// Handles image loading, placeholders, caching, scaling, generation, etc.
/// Primary job is to quickly return an image for a Model, and be able to
/// notify the view when a better one is available.
#[derive(Clone)]
pub struct ImageMangler {
    librarian: Library,
    ui: Weak<AppWindow>,
    default_artist: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    default_release_group: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    default_release: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    default_genre: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    default_other: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    threadpool: ThreadPool,
    cache_path: String,
}

impl ImageMangler {
    pub fn new(librarian: Library, ui: Weak<AppWindow>, cache_path: &str) -> Self {
        let images = Self {
            ui,
            librarian: librarian.clone(),
            default_artist: Self::load_default_image("images/artist_placeholder.png"),
            default_release_group: Self::load_default_image("images/release_group_placeholder.png"),
            default_release: Self::load_default_image("images/release_placeholder.png"),
            default_genre: Arc::new(Mutex::new(dynamic_to_buffer(&gen_fuzzy_circles(128, 128)))),
            default_other: Arc::new(Mutex::new(dynamic_to_buffer(&gen_fuzzy_rects(128, 128)))),
            threadpool: ThreadPool::new(1),
            cache_path: cache_path.to_string(),
        };

        images
    }

    pub fn lazy_get<F>(&self, model: impl Model + 'static, width: u32, height: u32, set_image: F) -> slint::Image
            where F: Fn(AppWindow, Image) + Send + Copy + 'static {
        let cache_key = format!("{}:{}:{}:{}", 
            model.table_name(), model.key().unwrap(), width, height);
        if let Some(dyn_image) = self.cache_get(&cache_key) {
            let buffer = dynamic_to_buffer(&dyn_image);
            return Image::from_rgba8_premultiplied(buffer.clone())
        }
        {
            let images = self.clone();
            let model = model.clone();
            let ui = self.ui.clone();
            self.threadpool.execute(move || {
                if let Some(dyn_image) = images.librarian.image(&model) {
                    let dyn_image = resize(dyn_image, width, height);
                    images.cache_set(&cache_key, &dyn_image);
                    let buffer = dynamic_to_buffer(&dyn_image);
                    ui.upgrade_in_event_loop(move |ui| {
                        let image = Image::from_rgba8_premultiplied(buffer);
                        set_image(ui, image);
                    }).unwrap();                    
                }
            });
        }
        Image::from_rgba8_premultiplied(self.default_model_image(model))
    }

    pub fn default_model_image(&self, model: impl Model) -> SharedPixelBuffer<Rgba8Pixel> {
        match model {
            // Model::Artist(_) => return self.default_artist.lock().unwrap().clone(),
            // Model::ReleaseGroup(_) => return self.default_release_group.lock().unwrap().clone(),
            // Model::Release(_) => return self.default_release.lock().unwrap().clone(),
            // Model::Genre(_) => return self.default_genre.lock().unwrap().clone(),
            _ => return self.default_other.lock().unwrap().clone(),
        }
    }

    pub fn cancel_all_pending(&self) {
        todo!()
    }

    fn cache_get(&self, key: &str) -> Option<DynamicImage> {
        if let Ok(bytes) = cacache::read_sync(self.cache_path.clone(), key) {
            if let Ok(dyn_image) = image::load_from_memory(&bytes) {
                return Some(dyn_image)
            }
        }
        None
    }

    fn cache_set(&self, key: &str, image: &DynamicImage) {
        let mut bytes = vec![];
        let mut cursor = Cursor::new(&mut bytes);
        image.write_to(&mut cursor, ImageFormat::Png).unwrap();
        cacache::write_sync(self.cache_path.clone(), key, bytes).unwrap();
    }

    pub fn cache_len(&self) -> usize {
        let mut len = 0;
        // TODO explodes when directory not created yet
        // for entry in cacache::list_sync(self.cache_path.clone()) {
        //     len += entry.unwrap().size;
        // }
        len
    }

    fn load_default_image(path: &str) -> Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>> {
        let image = image::open(path).ok().unwrap();
        Arc::new(Mutex::new(dynamic_to_buffer(&image)))
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