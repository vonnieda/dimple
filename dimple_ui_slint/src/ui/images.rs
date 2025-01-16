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

/// Handles image loading, placeholders, caching, scaling, generation, etc.
/// Primary job is to quickly return an image for a Model, and be able to
/// notify the view when a better one is available.
#[derive(Clone)]
pub struct ImageMangler {
    library: Library,
    artist_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    release_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    genre_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    track_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    playlist_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    other_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    threadpool: ThreadPool,
    cache_path: String,
    ui: Weak<AppWindow>,
}

impl ImageMangler {
    pub fn new(library: Library, ui: Weak<AppWindow>, cache_path: &str) -> Self {
        let images = Self {
            library: library.clone(),
            artist_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/users-three.png")),
            release_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/vinyl-record.png")),
            track_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/music-notes.png")),
            genre_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/globe-simple.png")),
            playlist_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/playlist.png")),
            other_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/question.png")),
            threadpool: ThreadPool::new(8),
            cache_path: cache_path.to_string(),
            ui,
        };

        images
    }

    pub fn lazy_get<F>(&self, model: impl Model + 'static, width: u32, height: u32, set_image: F) -> slint::Image
            where F: Fn(AppWindow, Image) + Send + Copy + 'static {
        let cache_key = format!("{}:{}:{}:{}", 
            model.type_name(), model.key().unwrap(), width, height);
        if let Some(dyn_image) = self.cache_get(&cache_key) {
            let buffer = dynamic_to_buffer(&dyn_image);
            return Image::from_rgba8_premultiplied(buffer.clone())
        }
        {
            let images = self.clone();
            let model1 = model.clone();
            let ui = self.ui.clone();
            self.threadpool.execute(move || {
                if let Some(dyn_image) = images.library.image(&model1) {
                    let dyn_image = resize(dyn_image, width, height);
                    images.cache_set(&cache_key, &dyn_image);
                    let buffer = dynamic_to_buffer(&dyn_image);
                    ui.upgrade_in_event_loop(move |ui| {
                        let image = Image::from_rgba8_premultiplied(buffer);
                        set_image(ui, image);
                    }).unwrap();                    
                }
            });
            Image::from_rgba8_premultiplied(self.get_model_placeholder(model))
        }
    }

    pub fn get_model_placeholder(&self, model: impl Model) -> SharedPixelBuffer<Rgba8Pixel> {
        match model.type_name().as_str() {
            "Artist" => return self.artist_placeholder.lock().unwrap().clone(),
            "Release" => return self.release_placeholder.lock().unwrap().clone(),
            "Track" => return self.track_placeholder.lock().unwrap().clone(),
            "Genre" => return self.genre_placeholder.lock().unwrap().clone(),
            "Playlist" => return self.playlist_placeholder.lock().unwrap().clone(),
            _ => return self.other_placeholder.lock().unwrap().clone(),
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
        for entry in cacache::list_sync(self.cache_path.clone()) {
            len += entry.unwrap().size;
        }
        len
    }

    fn load_default_image(buffer: &[u8]) -> Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>> {
        let image = image::load_from_memory(buffer).unwrap();
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

pub fn dynamic_to_slint(dyn_image: &DynamicImage) -> slint::Image {
    slint::Image::from_rgba8(dynamic_to_buffer(dyn_image))
}