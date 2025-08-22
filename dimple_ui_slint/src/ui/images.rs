use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;

use dimple_core::librarian::Librarian;
use dimple_core::model::DimpleEntity;
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
    librarian: Librarian,
    artist_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    release_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    genre_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    track_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    playlist_placeholder: Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>,
    threadpool: ThreadPool,
    cache_path: String,
    ui: Weak<AppWindow>,
}

impl ImageMangler {
    pub fn new(librarian: Librarian, ui: Weak<AppWindow>, cache_path: &str) -> Self {
        let images = Self {
            librarian: librarian.clone(),
            artist_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/users-three.png")),
            release_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/vinyl-record.png")),
            track_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/music-notes.png")),
            genre_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/globe-simple.png")),
            playlist_placeholder: Self::load_default_image(include_bytes!("../../icons/phosphor/PNGs/regular/playlist.png")),
            threadpool: ThreadPool::default(),
            cache_path: cache_path.to_string(),
            ui,
        };

        images
    }

    // TODO tomorrow, chopping block
    pub fn lazy_get<F>(&self, model: &DimpleEntity, width: u32, height: u32, set_image: F) -> slint::Image
            where F: Fn(AppWindow, Image) + Send + Copy + 'static {
        let cache_key = format!("{}:{}:{}", model.id(), width, height);
        if let Some(dyn_image) = self.cache_get(&cache_key) {
            let buffer = dynamic_to_buffer(&dyn_image);
            return Image::from_rgba8_premultiplied(buffer.clone())
        }
        let images_clone = self.clone();
        let model_clone = model.clone();
        let ui_clone = self.ui.clone();
        self.threadpool.execute(move || {
            if let Some(dyn_image) = images_clone.librarian.library.image(&model_clone) {
                let dyn_image = resize(dyn_image, width, height);
                images_clone.cache_set(&cache_key, &dyn_image);
                let buffer = dynamic_to_buffer(&dyn_image);
                ui_clone.upgrade_in_event_loop(move |ui| {
                    let image = Image::from_rgba8_premultiplied(buffer);
                    set_image(ui, image);
                }).unwrap();                    
            }
        });
        Image::from_rgba8_premultiplied(self.get_model_placeholder(model))
    }

    pub fn get_model_placeholder(&self, model: &DimpleEntity) -> SharedPixelBuffer<Rgba8Pixel> {
        match model {
            DimpleEntity::Artist(_artist) => self.artist_placeholder.lock().unwrap().clone(),
            DimpleEntity::Track(_track) => self.track_placeholder.lock().unwrap().clone(),
            DimpleEntity::Genre(_genre) => self.genre_placeholder.lock().unwrap().clone(),
            DimpleEntity::Release(_release) => self.release_placeholder.lock().unwrap().clone(),
            DimpleEntity::Playlist(_playlist) => self.playlist_placeholder.lock().unwrap().clone(),
        }
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

